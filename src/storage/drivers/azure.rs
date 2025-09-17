use async_trait::async_trait;
use anyhow::Result;
use azure_core::auth::TokenCredential;
use azure_storage::StorageCredentials;
use azure_storage_blobs::prelude::*;
use azure_storage_blobs::blob::operations::*;
use azure_storage_blobs::container::operations::*;
use chrono::{DateTime, Utc};
use futures::stream::StreamExt;

use crate::storage::filesystem::{FileInfo, Filesystem, FilesystemError};

pub struct AzureFilesystem {
    client: BlobServiceClient,
    container: String,
    prefix: Option<String>,
    url_prefix: Option<String>,
    visibility: String,
}

impl AzureFilesystem {
    pub async fn new(
        account_name: String,
        access_key: Option<String>,
        container: String,
        prefix: Option<String>,
        url_prefix: Option<String>,
    ) -> Result<Self> {
        let credentials = if let Some(key) = access_key {
            StorageCredentials::access_key(account_name.clone(), key)
        } else {
            // Use default Azure credentials (managed identity, etc.)
            return Err(FilesystemError::Config {
                message: "Azure managed identity authentication not yet implemented".to_string(),
            }.into());
        };

        let client = BlobServiceClient::new(account_name, credentials);

        Ok(Self {
            client,
            container,
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

    async fn blob_exists(&self, key: &str) -> Result<bool> {
        let blob_client = self.client.container_client(&self.container).blob_client(key);

        match blob_client.get_properties().await {
            Ok(_) => Ok(true),
            Err(azure_core::Error::HttpResponse { status, .. }) if status.as_u16() == 404 => Ok(false),
            Err(e) => Err(FilesystemError::Network {
                message: format!("Failed to check blob existence: {}", e),
            }.into()),
        }
    }

    fn generate_public_url(&self, path: &str) -> Option<String> {
        if let Some(ref url_prefix) = self.url_prefix {
            Some(format!("{}/{}", url_prefix.trim_end_matches('/'), path.trim_start_matches('/')))
        } else {
            // Generate default Azure Blob Storage URL
            let key = self.resolve_key(path);
            let account_name = self.client.account_name();
            Some(format!("https://{}.blob.core.windows.net/{}/{}", account_name, self.container, key))
        }
    }
}

#[async_trait]
impl Filesystem for AzureFilesystem {
    async fn exists(&self, path: &str) -> Result<bool> {
        let key = self.resolve_key(path);
        self.blob_exists(&key).await
    }

    async fn get(&self, path: &str) -> Result<Vec<u8>> {
        let key = self.resolve_key(path);
        let blob_client = self.client.container_client(&self.container).blob_client(&key);

        let response = blob_client.get().await
            .map_err(|e| {
                if let azure_core::Error::HttpResponse { status, .. } = &e {
                    if status.as_u16() == 404 {
                        return FilesystemError::FileNotFound { path: path.to_string() };
                    }
                }
                FilesystemError::Network {
                    message: format!("Failed to get blob: {}", e),
                }
            })?;

        let data = response.data.collect().await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to read blob data: {}", e),
            })?;

        Ok(data.to_vec())
    }

    async fn put(&self, path: &str, contents: &[u8]) -> Result<()> {
        let key = self.resolve_key(path);
        let blob_client = self.client.container_client(&self.container).blob_client(&key);

        let mut request = blob_client.put_block_blob(contents.to_vec());

        // Set content type based on file extension
        if let Some(mime_type) = mime_guess::from_path(path).first() {
            request = request.content_type(mime_type.to_string());
        }

        request.await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to put blob: {}", e),
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

        if !self.blob_exists(&key).await? {
            return Err(FilesystemError::FileNotFound {
                path: path.to_string(),
            }.into());
        }

        let blob_client = self.client.container_client(&self.container).blob_client(&key);

        blob_client.delete().await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to delete blob: {}", e),
            })?;

        Ok(())
    }

    async fn copy(&self, from: &str, to: &str) -> Result<()> {
        let from_key = self.resolve_key(from);
        let to_key = self.resolve_key(to);

        if !self.blob_exists(&from_key).await? {
            return Err(FilesystemError::FileNotFound {
                path: from.to_string(),
            }.into());
        }

        let source_blob_client = self.client.container_client(&self.container).blob_client(&from_key);
        let dest_blob_client = self.client.container_client(&self.container).blob_client(&to_key);

        // Get source blob URL for copy operation
        let source_url = format!("https://{}.blob.core.windows.net/{}/{}",
                                self.client.account_name(), self.container, from_key);

        dest_blob_client.copy(&source_url).await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to copy blob: {}", e),
            })?;

        Ok(())
    }

    async fn move_file(&self, from: &str, to: &str) -> Result<()> {
        self.copy(from, to).await?;
        self.delete(from).await?;
        Ok(())
    }

    async fn size(&self, path: &str) -> Result<u64> {
        let key = self.resolve_key(path);
        let blob_client = self.client.container_client(&self.container).blob_client(&key);

        let properties = blob_client.get_properties().await
            .map_err(|e| {
                if let azure_core::Error::HttpResponse { status, .. } = &e {
                    if status.as_u16() == 404 {
                        return FilesystemError::FileNotFound { path: path.to_string() };
                    }
                }
                FilesystemError::Network {
                    message: format!("Failed to get blob properties: {}", e),
                }
            })?;

        Ok(properties.blob.properties.content_length)
    }

    async fn last_modified(&self, path: &str) -> Result<DateTime<Utc>> {
        let key = self.resolve_key(path);
        let blob_client = self.client.container_client(&self.container).blob_client(&key);

        let properties = blob_client.get_properties().await
            .map_err(|e| {
                if let azure_core::Error::HttpResponse { status, .. } = &e {
                    if status.as_u16() == 404 {
                        return FilesystemError::FileNotFound { path: path.to_string() };
                    }
                }
                FilesystemError::Network {
                    message: format!("Failed to get blob properties: {}", e),
                }
            })?;

        Ok(properties.blob.properties.last_modified)
    }

    async fn files(&self, directory: &str) -> Result<Vec<String>> {
        let prefix = if directory.is_empty() {
            self.prefix.clone().unwrap_or_default()
        } else {
            self.resolve_key(&format!("{}/", directory.trim_end_matches('/')))
        };

        let container_client = self.client.container_client(&self.container);
        let mut request = container_client.list_blobs();

        if !prefix.is_empty() {
            request = request.prefix(prefix.clone());
        }

        let response = request.await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to list blobs: {}", e),
            })?;

        let mut files = Vec::new();
        for blob in response.blobs.blobs() {
            let name = &blob.name;
            if !name.ends_with('/') && name != &prefix {
                // Only include direct children (not nested)
                let relative_name = self.strip_prefix(name);
                if !relative_name.contains('/') || relative_name.matches('/').count() == 0 {
                    files.push(relative_name);
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

        let container_client = self.client.container_client(&self.container);
        let mut request = container_client.list_blobs();

        if !prefix.is_empty() {
            request = request.prefix(prefix.clone());
        }

        let response = request.await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to list blobs: {}", e),
            })?;

        let mut files = Vec::new();
        for blob in response.blobs.blobs() {
            let name = &blob.name;
            if !name.ends_with('/') && name != &prefix {
                files.push(self.strip_prefix(name));
            }
        }

        Ok(files)
    }

    async fn directories(&self, directory: &str) -> Result<Vec<String>> {
        // Azure Blob Storage doesn't have real directories
        // We simulate them by looking at blob prefixes
        let all_files = self.all_files(directory).await?;
        let mut directories = std::collections::HashSet::new();

        for file in all_files {
            let path = std::path::Path::new(&file);
            if let Some(parent) = path.parent() {
                if parent != std::path::Path::new("") {
                    let parent_str = parent.to_string_lossy().to_string();
                    // Only include direct children directories
                    if !parent_str.contains('/') {
                        directories.insert(parent_str);
                    }
                }
            }
        }

        Ok(directories.into_iter().collect())
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
        // Azure doesn't have real directories, create a placeholder blob
        let key = self.resolve_key(&format!("{}/", path.trim_end_matches('/')));
        self.put(&key, &[]).await
    }

    async fn delete_directory(&self, directory: &str) -> Result<()> {
        let prefix = self.resolve_key(&format!("{}/", directory.trim_end_matches('/')));

        let container_client = self.client.container_client(&self.container);
        let response = container_client.list_blobs()
            .prefix(prefix.clone())
            .await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to list blobs: {}", e),
            })?;

        for blob in response.blobs.blobs() {
            let blob_client = self.client.container_client(&self.container).blob_client(&blob.name);
            blob_client.delete().await
                .map_err(|e| FilesystemError::Network {
                    message: format!("Failed to delete blob: {}", e),
                })?;
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
        // Azure SAS token generation would require additional implementation
        Err(FilesystemError::Config {
            message: "Temporary URLs not yet implemented for Azure Blob Storage".to_string(),
        }.into())
    }

    async fn get_info(&self, path: &str) -> Result<FileInfo> {
        let key = self.resolve_key(path);
        let blob_client = self.client.container_client(&self.container).blob_client(&key);

        let properties = blob_client.get_properties().await
            .map_err(|e| {
                if let azure_core::Error::HttpResponse { status, .. } = &e {
                    if status.as_u16() == 404 {
                        return FilesystemError::FileNotFound { path: path.to_string() };
                    }
                }
                FilesystemError::Network {
                    message: format!("Failed to get blob properties: {}", e),
                }
            })?;

        let is_directory = path.ends_with('/') || key.ends_with('/');

        if is_directory {
            Ok(FileInfo::directory(path.to_string(), properties.blob.properties.last_modified))
        } else {
            Ok(FileInfo::new(
                path.to_string(),
                properties.blob.properties.content_length,
                properties.blob.properties.last_modified,
            ))
        }
    }

    async fn set_visibility(&self, _path: &str, _visibility: &str) -> Result<()> {
        // Azure Blob Storage ACL management would require additional implementation
        Ok(())
    }

    async fn get_visibility(&self, _path: &str) -> Result<String> {
        Ok(self.visibility.clone())
    }

    fn name(&self) -> &str {
        "azure"
    }

    fn is_local(&self) -> bool {
        false
    }
}