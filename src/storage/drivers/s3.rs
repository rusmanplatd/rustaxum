use async_trait::async_trait;
use anyhow::Result;
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::Credentials;
use aws_sdk_s3::{Client, Config};
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::operation::head_object::HeadObjectError;
use aws_sdk_s3::primitives::ByteStream;
use chrono::{DateTime, Utc};
use std::time::SystemTime;

use crate::storage::filesystem::{FileInfo, Filesystem, FilesystemError};

#[derive(Debug)]
pub struct S3Filesystem {
    client: Client,
    bucket: String,
    region: String,
    prefix: Option<String>,
    url_prefix: Option<String>,
    visibility: String,
}

impl S3Filesystem {
    pub async fn new(
        endpoint: Option<String>,
        region: String,
        access_key: String,
        secret_key: String,
        bucket: String,
        prefix: Option<String>,
        url_prefix: Option<String>,
    ) -> Result<Self> {
        let credentials = Credentials::new(
            access_key,
            secret_key,
            None,
            None,
            "rustaxum-storage",
        );

        let mut config_builder = Config::builder()
            .region(Region::new(region.clone()))
            .credentials_provider(credentials)
            .behavior_version(BehaviorVersion::v2025_08_07());

        // For MinIO or custom S3-compatible endpoints
        if let Some(endpoint_url) = endpoint {
            config_builder = config_builder
                .endpoint_url(endpoint_url)
                .force_path_style(true);
        }

        let config = config_builder.build();
        let client = Client::from_conf(config);

        Ok(Self {
            client,
            bucket,
            region,
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
        match self.client.head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(err) => {
                if let Some(HeadObjectError::NotFound(_)) = err.as_service_error() {
                    Ok(false)
                } else {
                    Err(FilesystemError::Network {
                        message: format!("Failed to check object existence: {}", err),
                    }.into())
                }
            }
        }
    }

    async fn list_objects(&self, prefix: &str, delimiter: Option<&str>) -> Result<(Vec<String>, Vec<String>)> {
        let mut files = Vec::new();
        let mut directories = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self.client.list_objects_v2()
                .bucket(&self.bucket)
                .prefix(prefix);

            if let Some(delim) = delimiter {
                request = request.delimiter(delim);
            }

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request.send().await
                .map_err(|e| FilesystemError::Network {
                    message: format!("Failed to list objects: {}", e),
                })?;

            // Process objects (files)
            for object in response.contents() {
                if let Some(key) = object.key() {
                    if !key.ends_with('/') && key != prefix {
                        files.push(self.strip_prefix(key));
                    }
                }
            }

            // Process common prefixes (directories)
            for prefix_obj in response.common_prefixes() {
                if let Some(prefix_key) = prefix_obj.prefix() {
                    let dir_name = self.strip_prefix(prefix_key.trim_end_matches('/'));
                    if !dir_name.is_empty() {
                        directories.push(dir_name);
                    }
                }
            }

            if !response.is_truncated().unwrap_or(false) {
                break;
            }

            continuation_token = response.next_continuation_token().map(|s| s.to_string());
        }

        Ok((files, directories))
    }

    fn generate_public_url(&self, path: &str) -> Option<String> {
        if let Some(ref url_prefix) = self.url_prefix {
            Some(format!("{}/{}", url_prefix.trim_end_matches('/'), path.trim_start_matches('/')))
        } else {
            // Generate default S3 URL
            let key = self.resolve_key(path);
            Some(format!("https://{}.s3.{}.amazonaws.com/{}", self.bucket, self.region, key))
        }
    }
}

#[async_trait]
impl Filesystem for S3Filesystem {
    async fn exists(&self, path: &str) -> Result<bool> {
        let key = self.resolve_key(path);
        self.object_exists(&key).await
    }

    async fn get(&self, path: &str) -> Result<Vec<u8>> {
        let key = self.resolve_key(path);

        let response = self.client.get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| {
                if let Some(GetObjectError::NoSuchKey(_)) = e.as_service_error() {
                    FilesystemError::FileNotFound { path: path.to_string() }
                } else {
                    FilesystemError::Network {
                        message: format!("Failed to get object: {}", e),
                    }
                }
            })?;

        let data = response.body.collect().await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to read object body: {}", e),
            })?;

        Ok(data.into_bytes().to_vec())
    }

    async fn put(&self, path: &str, contents: &[u8]) -> Result<()> {
        let key = self.resolve_key(path);
        let body = ByteStream::from(contents.to_vec());

        let mut request = self.client.put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(body);

        // Set ACL based on visibility
        if self.visibility == "public" {
            request = request.acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead);
        }

        request.send().await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to put object: {}", e),
            })?;

        Ok(())
    }

    async fn put_file_as(&self, path: &str, file: &str, name: Option<String>) -> Result<String> {
        // Read the source file
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

        self.client.delete_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to delete object: {}", e),
            })?;

        Ok(())
    }

    async fn copy(&self, from: &str, to: &str) -> Result<()> {
        let from_key = self.resolve_key(from);
        let to_key = self.resolve_key(to);

        if !self.object_exists(&from_key).await? {
            return Err(FilesystemError::FileNotFound {
                path: from.to_string(),
            }.into());
        }

        let copy_source = format!("{}/{}", self.bucket, from_key);

        let mut request = self.client.copy_object()
            .bucket(&self.bucket)
            .key(&to_key)
            .copy_source(&copy_source);

        if self.visibility == "public" {
            request = request.acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead);
        }

        request.send().await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to copy object: {}", e),
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

        let response = self.client.head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| {
                if let Some(HeadObjectError::NotFound(_)) = e.as_service_error() {
                    FilesystemError::FileNotFound { path: path.to_string() }
                } else {
                    FilesystemError::Network {
                        message: format!("Failed to get object metadata: {}", e),
                    }
                }
            })?;

        Ok(response.content_length().unwrap_or(0) as u64)
    }

    async fn last_modified(&self, path: &str) -> Result<DateTime<Utc>> {
        let key = self.resolve_key(path);

        let response = self.client.head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| {
                if let Some(HeadObjectError::NotFound(_)) = e.as_service_error() {
                    FilesystemError::FileNotFound { path: path.to_string() }
                } else {
                    FilesystemError::Network {
                        message: format!("Failed to get object metadata: {}", e),
                    }
                }
            })?;

        if let Some(last_modified) = response.last_modified() {
            let system_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(last_modified.secs() as u64);
            Ok(system_time.into())
        } else {
            Ok(Utc::now())
        }
    }

    async fn files(&self, directory: &str) -> Result<Vec<String>> {
        let prefix = if directory.is_empty() {
            self.prefix.clone().unwrap_or_default()
        } else {
            self.resolve_key(&format!("{}/", directory.trim_end_matches('/')))
        };

        let (files, _) = self.list_objects(&prefix, Some("/")).await?;
        Ok(files)
    }

    async fn all_files(&self, directory: &str) -> Result<Vec<String>> {
        let prefix = if directory.is_empty() {
            self.prefix.clone().unwrap_or_default()
        } else {
            self.resolve_key(&format!("{}/", directory.trim_end_matches('/')))
        };

        let (files, _) = self.list_objects(&prefix, None).await?;
        Ok(files)
    }

    async fn directories(&self, directory: &str) -> Result<Vec<String>> {
        let prefix = if directory.is_empty() {
            self.prefix.clone().unwrap_or_default()
        } else {
            self.resolve_key(&format!("{}/", directory.trim_end_matches('/')))
        };

        let (_, directories) = self.list_objects(&prefix, Some("/")).await?;
        Ok(directories)
    }

    async fn all_directories(&self, directory: &str) -> Result<Vec<String>> {
        // For S3, we need to collect all objects and extract directory paths
        let prefix = if directory.is_empty() {
            self.prefix.clone().unwrap_or_default()
        } else {
            self.resolve_key(&format!("{}/", directory.trim_end_matches('/')))
        };

        let (all_files, _) = self.list_objects(&prefix, None).await?;
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
        // S3 doesn't have real directories, but we create a placeholder object
        let key = self.resolve_key(&format!("{}/", path.trim_end_matches('/')));
        let body = ByteStream::from(vec![]);

        self.client.put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(body)
            .send()
            .await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to create directory marker: {}", e),
            })?;

        Ok(())
    }

    async fn delete_directory(&self, directory: &str) -> Result<()> {
        let prefix = self.resolve_key(&format!("{}/", directory.trim_end_matches('/')));

        // List all objects with this prefix
        let (all_files, _) = self.list_objects(&prefix, None).await?;

        // Delete all objects in batches
        const BATCH_SIZE: usize = 1000; // S3 delete batch limit
        for chunk in all_files.chunks(BATCH_SIZE) {
            let mut delete_objects = Vec::new();

            for file in chunk {
                let key = self.resolve_key(file);
                delete_objects.push(
                    aws_sdk_s3::types::ObjectIdentifier::builder()
                        .key(key)
                        .build()
                        .map_err(|e| FilesystemError::Config {
                            message: format!("Failed to build object identifier: {}", e),
                        })?
                );
            }

            if !delete_objects.is_empty() {
                let delete_request = aws_sdk_s3::types::Delete::builder()
                    .set_objects(Some(delete_objects))
                    .build()
                    .map_err(|e| FilesystemError::Config {
                        message: format!("Failed to build delete request: {}", e),
                    })?;

                self.client.delete_objects()
                    .bucket(&self.bucket)
                    .delete(delete_request)
                    .send()
                    .await
                    .map_err(|e| FilesystemError::Network {
                        message: format!("Failed to delete objects: {}", e),
                    })?;
            }
        }

        // Also delete the directory marker if it exists
        let dir_marker = format!("{}/", directory.trim_end_matches('/'));
        if self.exists(&dir_marker).await? {
            self.delete(&dir_marker).await?;
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
        let key = self.resolve_key(path);

        let duration_since_epoch = expires_at.signed_duration_since(Utc::now());
        let expires_in = duration_since_epoch.num_seconds().max(1) as u64;

        let presigned_request = self.client.get_object()
            .bucket(&self.bucket)
            .key(&key)
            .presigned(
                aws_sdk_s3::presigning::PresigningConfig::expires_in(
                    std::time::Duration::from_secs(expires_in)
                ).map_err(|e| FilesystemError::Config {
                    message: format!("Failed to create presigning config: {}", e),
                })?
            )
            .await
            .map_err(|e| FilesystemError::Config {
                message: format!("Failed to create presigned URL: {}", e),
            })?;

        Ok(presigned_request.uri().to_string())
    }

    async fn get_info(&self, path: &str) -> Result<FileInfo> {
        let key = self.resolve_key(path);

        let response = self.client.head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| {
                if let Some(HeadObjectError::NotFound(_)) = e.as_service_error() {
                    FilesystemError::FileNotFound { path: path.to_string() }
                } else {
                    FilesystemError::Network {
                        message: format!("Failed to get object metadata: {}", e),
                    }
                }
            })?;

        let size = response.content_length().unwrap_or(0) as u64;
        let last_modified = if let Some(lm) = response.last_modified() {
            let system_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(lm.secs() as u64);
            system_time.into()
        } else {
            Utc::now()
        };

        let is_directory = path.ends_with('/') || key.ends_with('/');

        if is_directory {
            Ok(FileInfo::directory(path.to_string(), last_modified))
        } else {
            Ok(FileInfo::new(path.to_string(), size, last_modified))
        }
    }

    async fn set_visibility(&self, path: &str, visibility: &str) -> Result<()> {
        let key = self.resolve_key(path);

        let acl = match visibility {
            "public" => aws_sdk_s3::types::ObjectCannedAcl::PublicRead,
            "private" => aws_sdk_s3::types::ObjectCannedAcl::Private,
            _ => return Err(FilesystemError::Config {
                message: format!("Unsupported visibility: {}", visibility),
            }.into()),
        };

        self.client.put_object_acl()
            .bucket(&self.bucket)
            .key(&key)
            .acl(acl)
            .send()
            .await
            .map_err(|e| FilesystemError::Network {
                message: format!("Failed to set object ACL: {}", e),
            })?;

        Ok(())
    }

    async fn get_visibility(&self, _path: &str) -> Result<String> {
        // TODO: This would require additional S3 API calls to get ACL
        Ok(self.visibility.clone())
    }

    fn name(&self) -> &str {
        "s3"
    }

    fn is_local(&self) -> bool {
        false
    }
}