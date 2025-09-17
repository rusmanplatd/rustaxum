use async_trait::async_trait;
use anyhow::Result;
use chrono::{DateTime, Utc};
use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::{
    delete::DeleteObjectRequest,
    get::GetObjectRequest,
    insert::{InsertObjectRequest, Media},
    list::ListObjectsRequest,
};
use google_cloud_storage::http::buckets::get::GetBucketRequest;
use std::collections::HashMap;

use crate::storage::filesystem::{FileInfo, Filesystem, FilesystemError};

pub struct GcsFilesystem {
    client: Client,
    bucket: String,
    prefix: Option<String>,
    url_prefix: Option<String>,
    visibility: String,
}

impl GcsFilesystem {
    pub async fn new(
        project_id: String,
        bucket: String,
        service_account_key: Option<String>,
        prefix: Option<String>,
        url_prefix: Option<String>,
    ) -> Result<Self> {
        let config = if let Some(key_json) = service_account_key {
            ClientConfig::default()
                .with_service_account_key_json(&key_json)
                .map_err(|e| FilesystemError::Config {
                    message: format!("Failed to parse service account key: {}", e),
                })?
        } else {
            // Use default credentials (ADC)
            ClientConfig::default()
        };

        let client = Client::new(config);

        Ok(Self {
            client,
            bucket,
            prefix,
            url_prefix,
            visibility: "private".to_string(),
        })
    }

    pub fn with_visibility(mut self, visibility: String) -> Self {
        self.visibility = visibility;
        self
    }

    fn resolve_key(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        if let Some(ref prefix) = self.prefix {
            format!("{}/{}", prefix.trim_end_matches('/'), path)
        } else {
            path.to_string()
        }
    }

    fn strip_prefix(&self, key: &str) -> String {
        if let Some(ref prefix) = self.prefix {
            let prefix_with_slash = format!("{}/", prefix.trim_end_matches('/'));
            if key.starts_with(&prefix_with_slash) {
                key[prefix_with_slash.len()..].to_string()
            } else {
                key.to_string()
            }
        } else {
            key.to_string()
        }
    }

    async fn object_exists(&self, key: &str) -> Result<bool> {
        let request = GetObjectRequest {
            bucket: self.bucket.clone(),
            object: key.to_string(),
            ..Default::default()
        };

        match self.client.get_object(&request).await {
            Ok(_) => Ok(true),
            Err(google_cloud_storage::http::Error::HttpClient(ref http_err)) => {
                if http_err.to_string().contains("404") {
                    Ok(false)
                } else {
                    Err(FilesystemError::Network {
                        message: format!("Failed to check object existence: {}", http_err),
                    }.into())
                }
            }
            Err(e) => Err(FilesystemError::Network {
                message: format!("Failed to check object existence: {}", e),
            }.into()),
        }
    }

    fn generate_public_url(&self, path: &str) -> Option<String> {
        if let Some(ref url_prefix) = self.url_prefix {
            Some(format!("{}/{}", url_prefix.trim_end_matches('/'), path.trim_start_matches('/')))
        } else {
            // Generate default GCS public URL
            let key = self.resolve_key(path);
            Some(format!("https://storage.googleapis.com/{}/{}", self.bucket, key))
        }
    }
}

#[async_trait]
impl Filesystem for GcsFilesystem {
    async fn exists(&self, path: &str) -> Result<bool> {
        let key = self.resolve_key(path);
        self.object_exists(&key).await
    }

    async fn get(&self, path: &str) -> Result<Vec<u8>> {
        let key = self.resolve_key(path);

        let request = GetObjectRequest {
            bucket: self.bucket.clone(),
            object: key.clone(),
            ..Default::default()
        };

        let data = self.client.download_object(&request, &std::ops::Range { start: 0, end: usize::MAX }).await
            .map_err(|e| {
                if e.to_string().contains("404") {
                    FilesystemError::FileNotFound { path: path.to_string() }
                } else {
                    FilesystemError::Network {
                        message: format!("Failed to get object: {}", e),
                    }
                }
            })?;

        Ok(data)
    }

    async fn put(&self, path: &str, contents: &[u8]) -> Result<()> {
        let key = self.resolve_key(path);

        let mut metadata = HashMap::new();
        if let Some(mime_type) = mime_guess::from_path(path).first() {
            metadata.insert("content-type".to_string(), mime_type.to_string());
        }

        let media = Media {
            name: key.clone(),
            content_type: mime_guess::from_path(path).first().map(|m| m.to_string()),
            content_length: Some(contents.len()),
        };

        let request = InsertObjectRequest {
            bucket: self.bucket.clone(),
            ..Default::default()
        };

        self.client.upload_object(&request, contents.to_vec(), &media).await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to put object: {}", e),
            })?;

        Ok(())
    }

    async fn put_file_as(&self, path: &str, file: &str, name: Option<String>) -> Result<String> {
        let source_data = tokio::fs::read(file).await
            .map_err(|e| FilesystemError::Io(e))?;

        let filename = name.unwrap_or_else(|| {
            std::path::Path::new(file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
                .to_string()
        });

        let dest_path = format!("{}/{}", path.trim_end_matches('/'), filename);
        self.put(&dest_path, &source_data).await?;

        Ok(dest_path)
    }

    async fn prepend(&self, path: &str, data: &[u8]) -> Result<()> {
        let existing_data = match self.get(path).await {
            Ok(data) => data,
            Err(_) => Vec::new(),
        };

        let mut new_data = data.to_vec();
        new_data.extend_from_slice(&existing_data);

        self.put(path, &new_data).await
    }

    async fn append(&self, path: &str, data: &[u8]) -> Result<()> {
        let existing_data = match self.get(path).await {
            Ok(data) => data,
            Err(_) => Vec::new(),
        };

        let mut new_data = existing_data;
        new_data.extend_from_slice(data);

        self.put(path, &new_data).await
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let key = self.resolve_key(path);

        if !self.object_exists(&key).await? {
            return Err(FilesystemError::FileNotFound {
                path: path.to_string(),
            }.into());
        }

        let request = DeleteObjectRequest {
            bucket: self.bucket.clone(),
            object: key,
            ..Default::default()
        };

        self.client.delete_object(&request).await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to delete object: {}", e),
            })?;

        Ok(())
    }

    async fn copy(&self, from: &str, to: &str) -> Result<()> {
        // GCS copy requires getting the object and putting it again
        let data = self.get(from).await?;
        self.put(to, &data).await
    }

    async fn move_file(&self, from: &str, to: &str) -> Result<()> {
        self.copy(from, to).await?;
        self.delete(from).await?;
        Ok(())
    }

    async fn size(&self, path: &str) -> Result<u64> {
        let key = self.resolve_key(path);

        let request = GetObjectRequest {
            bucket: self.bucket.clone(),
            object: key.clone(),
            ..Default::default()
        };

        let object = self.client.get_object(&request).await
            .map_err(|e| {
                if e.to_string().contains("404") {
                    FilesystemError::FileNotFound { path: path.to_string() }
                } else {
                    FilesystemError::Network {
                        message: format!("Failed to get object metadata: {}", e),
                    }
                }
            })?;

        Ok(object.size as u64)
    }

    async fn last_modified(&self, path: &str) -> Result<DateTime<Utc>> {
        let key = self.resolve_key(path);

        let request = GetObjectRequest {
            bucket: self.bucket.clone(),
            object: key.clone(),
            ..Default::default()
        };

        let object = self.client.get_object(&request).await
            .map_err(|e| {
                if e.to_string().contains("404") {
                    FilesystemError::FileNotFound { path: path.to_string() }
                } else {
                    FilesystemError::Network {
                        message: format!("Failed to get object metadata: {}", e),
                    }
                }
            })?;

        Ok(object.updated)
    }

    async fn files(&self, directory: &str) -> Result<Vec<String>> {
        let prefix = if directory.is_empty() {
            self.prefix.clone().unwrap_or_default()
        } else {
            self.resolve_key(&format!("{}/", directory.trim_end_matches('/')))
        };

        let request = ListObjectsRequest {
            bucket: self.bucket.clone(),
            prefix: Some(prefix.clone()),
            delimiter: Some("/".to_string()),
            ..Default::default()
        };

        let response = self.client.list_objects(&request).await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to list objects: {}", e),
            })?;

        let mut files = Vec::new();
        if let Some(items) = response.items {
            for item in items {
                if !item.name.ends_with('/') && item.name != prefix {
                    files.push(self.strip_prefix(&item.name));
                }
            }
        }

        Ok(files)
    }

    async fn all_files(&self, directory: &str) -> Result<Vec<String>> {
        let prefix = if directory.is_empty() {
            self.prefix.clone().unwrap_or_default()
        } else {
            self.resolve_key(&format!("{}/", directory.trim_end_matches('/')))
        };

        let request = ListObjectsRequest {
            bucket: self.bucket.clone(),
            prefix: Some(prefix.clone()),
            ..Default::default()
        };

        let response = self.client.list_objects(&request).await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to list objects: {}", e),
            })?;

        let mut files = Vec::new();
        if let Some(items) = response.items {
            for item in items {
                if !item.name.ends_with('/') && item.name != prefix {
                    files.push(self.strip_prefix(&item.name));
                }
            }
        }

        Ok(files)
    }

    async fn directories(&self, directory: &str) -> Result<Vec<String>> {
        let prefix = if directory.is_empty() {
            self.prefix.clone().unwrap_or_default()
        } else {
            self.resolve_key(&format!("{}/", directory.trim_end_matches('/')))
        };

        let request = ListObjectsRequest {
            bucket: self.bucket.clone(),
            prefix: Some(prefix.clone()),
            delimiter: Some("/".to_string()),
            ..Default::default()
        };

        let response = self.client.list_objects(&request).await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to list objects: {}", e),
            })?;

        let mut directories = Vec::new();
        if let Some(prefixes) = response.prefixes {
            for prefix_str in prefixes {
                let dir_name = self.strip_prefix(&prefix_str.trim_end_matches('/'));
                if !dir_name.is_empty() {
                    directories.push(dir_name);
                }
            }
        }

        Ok(directories)
    }

    async fn all_directories(&self, directory: &str) -> Result<Vec<String>> {
        let all_files = self.all_files(directory).await?;
        let mut directories = std::collections::HashSet::new();

        for file in all_files {
            let path = std::path::Path::new(&file);
            if let Some(parent) = path.parent() {
                if parent != std::path::Path::new("") {
                    directories.insert(parent.to_string_lossy().to_string());
                }
            }
        }

        Ok(directories.into_iter().collect())
    }

    async fn make_directory(&self, path: &str) -> Result<()> {
        // GCS doesn't have real directories, create a placeholder object
        let key = self.resolve_key(&format!("{}/", path.trim_end_matches('/')));
        self.put(&key, &[]).await
    }

    async fn delete_directory(&self, directory: &str) -> Result<()> {
        let prefix = self.resolve_key(&format!("{}/", directory.trim_end_matches('/')));

        // List all objects with this prefix
        let request = ListObjectsRequest {
            bucket: self.bucket.clone(),
            prefix: Some(prefix.clone()),
            ..Default::default()
        };

        let response = self.client.list_objects(&request).await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to list objects: {}", e),
            })?;

        if let Some(items) = response.items {
            for item in items {
                let delete_request = DeleteObjectRequest {
                    bucket: self.bucket.clone(),
                    object: item.name,
                    ..Default::default()
                };

                self.client.delete_object(&delete_request).await
                    .map_err(|e| FilesystemError::Network {
                        message: format!("Failed to delete object: {}", e),
                    })?;
            }
        }

        Ok(())
    }

    async fn url(&self, path: &str) -> Result<Option<String>> {
        if self.visibility == "public" {
            Ok(self.generate_public_url(path))
        } else {
            Ok(None)
        }
    }

    async fn temporary_url(&self, path: &str, expires_at: DateTime<Utc>) -> Result<String> {
        // GCS signed URLs require additional implementation
        // For now, return a basic URL or implement using service account keys
        Err(FilesystemError::Config {
            message: "Temporary URLs not yet implemented for GCS".to_string(),
        }.into())
    }

    async fn get_info(&self, path: &str) -> Result<FileInfo> {
        let key = self.resolve_key(path);

        let request = GetObjectRequest {
            bucket: self.bucket.clone(),
            object: key.clone(),
            ..Default::default()
        };

        let object = self.client.get_object(&request).await
            .map_err(|e| {
                if e.to_string().contains("404") {
                    FilesystemError::FileNotFound { path: path.to_string() }
                } else {
                    FilesystemError::Network {
                        message: format!("Failed to get object metadata: {}", e),
                    }
                }
            })?;

        let is_directory = path.ends_with('/') || key.ends_with('/');

        if is_directory {
            Ok(FileInfo::directory(path.to_string(), object.updated))
        } else {
            Ok(FileInfo::new(path.to_string(), object.size as u64, object.updated))
        }
    }

    async fn set_visibility(&self, _path: &str, _visibility: &str) -> Result<()> {
        // GCS ACL management would require additional implementation
        Ok(())
    }

    async fn get_visibility(&self, _path: &str) -> Result<String> {
        Ok(self.visibility.clone())
    }

    fn name(&self) -> &str {
        "gcs"
    }

    fn is_local(&self) -> bool {
        false
    }
}