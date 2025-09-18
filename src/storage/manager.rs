use anyhow::Result;
use std::collections::HashMap;

use crate::config::{storage::StorageConfig, Config};
use crate::storage::drivers::{LocalFilesystem, S3Filesystem};
use crate::storage::filesystem::{Filesystem, FilesystemError};

#[derive(Debug)]
pub enum FilesystemDriver {
    Local(LocalFilesystem),
    S3(S3Filesystem),
}

impl FilesystemDriver {
    pub async fn exists(&self, path: &str) -> Result<bool> {
        match self {
            FilesystemDriver::Local(fs) => fs.exists(path).await,
            FilesystemDriver::S3(fs) => fs.exists(path).await,
        }
    }

    pub async fn get(&self, path: &str) -> Result<Vec<u8>> {
        match self {
            FilesystemDriver::Local(fs) => fs.get(path).await,
            FilesystemDriver::S3(fs) => fs.get(path).await,
        }
    }

    pub async fn put(&self, path: &str, contents: &[u8]) -> Result<()> {
        match self {
            FilesystemDriver::Local(fs) => fs.put(path, contents).await,
            FilesystemDriver::S3(fs) => fs.put(path, contents).await,
        }
    }

    pub async fn put_file_as(&self, path: &str, file: &str, name: Option<String>) -> Result<String> {
        match self {
            FilesystemDriver::Local(fs) => fs.put_file_as(path, file, name).await,
            FilesystemDriver::S3(fs) => fs.put_file_as(path, file, name).await,
        }
    }

    pub async fn prepend(&self, path: &str, data: &[u8]) -> Result<()> {
        match self {
            FilesystemDriver::Local(fs) => fs.prepend(path, data).await,
            FilesystemDriver::S3(fs) => fs.prepend(path, data).await,
        }
    }

    pub async fn append(&self, path: &str, data: &[u8]) -> Result<()> {
        match self {
            FilesystemDriver::Local(fs) => fs.append(path, data).await,
            FilesystemDriver::S3(fs) => fs.append(path, data).await,
        }
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        match self {
            FilesystemDriver::Local(fs) => fs.delete(path).await,
            FilesystemDriver::S3(fs) => fs.delete(path).await,
        }
    }

    pub async fn copy(&self, from: &str, to: &str) -> Result<()> {
        match self {
            FilesystemDriver::Local(fs) => fs.copy(from, to).await,
            FilesystemDriver::S3(fs) => fs.copy(from, to).await,
        }
    }

    pub async fn move_file(&self, from: &str, to: &str) -> Result<()> {
        match self {
            FilesystemDriver::Local(fs) => fs.move_file(from, to).await,
            FilesystemDriver::S3(fs) => fs.move_file(from, to).await,
        }
    }

    pub async fn size(&self, path: &str) -> Result<u64> {
        match self {
            FilesystemDriver::Local(fs) => fs.size(path).await,
            FilesystemDriver::S3(fs) => fs.size(path).await,
        }
    }

    pub async fn last_modified(&self, path: &str) -> Result<chrono::DateTime<chrono::Utc>> {
        match self {
            FilesystemDriver::Local(fs) => fs.last_modified(path).await,
            FilesystemDriver::S3(fs) => fs.last_modified(path).await,
        }
    }

    pub async fn files(&self, directory: &str) -> Result<Vec<String>> {
        match self {
            FilesystemDriver::Local(fs) => fs.files(directory).await,
            FilesystemDriver::S3(fs) => fs.files(directory).await,
        }
    }

    pub async fn all_files(&self, directory: &str) -> Result<Vec<String>> {
        match self {
            FilesystemDriver::Local(fs) => fs.all_files(directory).await,
            FilesystemDriver::S3(fs) => fs.all_files(directory).await,
        }
    }

    pub async fn directories(&self, directory: &str) -> Result<Vec<String>> {
        match self {
            FilesystemDriver::Local(fs) => fs.directories(directory).await,
            FilesystemDriver::S3(fs) => fs.directories(directory).await,
        }
    }

    pub async fn all_directories(&self, directory: &str) -> Result<Vec<String>> {
        match self {
            FilesystemDriver::Local(fs) => fs.all_directories(directory).await,
            FilesystemDriver::S3(fs) => fs.all_directories(directory).await,
        }
    }

    pub async fn make_directory(&self, path: &str) -> Result<()> {
        match self {
            FilesystemDriver::Local(fs) => fs.make_directory(path).await,
            FilesystemDriver::S3(fs) => fs.make_directory(path).await,
        }
    }

    pub async fn delete_directory(&self, directory: &str) -> Result<()> {
        match self {
            FilesystemDriver::Local(fs) => fs.delete_directory(directory).await,
            FilesystemDriver::S3(fs) => fs.delete_directory(directory).await,
        }
    }

    pub async fn url(&self, path: &str) -> Result<Option<String>> {
        match self {
            FilesystemDriver::Local(fs) => fs.url(path).await,
            FilesystemDriver::S3(fs) => fs.url(path).await,
        }
    }

    pub async fn temporary_url(&self, path: &str, expires_at: chrono::DateTime<chrono::Utc>) -> Result<String> {
        match self {
            FilesystemDriver::Local(fs) => fs.temporary_url(path, expires_at).await,
            FilesystemDriver::S3(fs) => fs.temporary_url(path, expires_at).await,
        }
    }

    pub async fn get_info(&self, path: &str) -> Result<crate::storage::filesystem::FileInfo> {
        match self {
            FilesystemDriver::Local(fs) => fs.get_info(path).await,
            FilesystemDriver::S3(fs) => fs.get_info(path).await,
        }
    }

    pub async fn set_visibility(&self, path: &str, visibility: &str) -> Result<()> {
        match self {
            FilesystemDriver::Local(fs) => fs.set_visibility(path, visibility).await,
            FilesystemDriver::S3(fs) => fs.set_visibility(path, visibility).await,
        }
    }

    pub async fn get_visibility(&self, path: &str) -> Result<String> {
        match self {
            FilesystemDriver::Local(fs) => fs.get_visibility(path).await,
            FilesystemDriver::S3(fs) => fs.get_visibility(path).await,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            FilesystemDriver::Local(fs) => fs.name(),
            FilesystemDriver::S3(fs) => fs.name(),
        }
    }

    pub fn is_local(&self) -> bool {
        match self {
            FilesystemDriver::Local(fs) => fs.is_local(),
            FilesystemDriver::S3(fs) => fs.is_local(),
        }
    }

    pub fn is_cloud(&self) -> bool {
        match self {
            FilesystemDriver::Local(fs) => fs.is_cloud(),
            FilesystemDriver::S3(fs) => fs.is_cloud(),
        }
    }
}

pub struct StorageManager {
    config: StorageConfig,
    disks: HashMap<String, FilesystemDriver>,
}

impl StorageManager {
    pub async fn new() -> Result<Self> {
        let config = Config::load()?;
        Ok(Self {
            config: config.storage,
            disks: HashMap::new(),
        })
    }

    pub async fn from_config(config: StorageConfig) -> Self {
        Self {
            config,
            disks: HashMap::new(),
        }
    }

    pub async fn disk(&mut self, name: &str) -> Result<&FilesystemDriver> {
        let disk_name = if name == "default" {
            &self.config.default
        } else {
            name
        };

        // Check if disk is already created
        if !self.disks.contains_key(disk_name) {
            // Create new disk instance
            let disk_config = self.config.get_disk(disk_name).ok_or_else(|| {
                FilesystemError::Config {
                    message: format!("Disk '{}' not found in configuration", disk_name),
                }
            })?;

            let filesystem = match disk_config.driver.as_str() {
                "local" => {
                    let root = disk_config.get_root();
                    let fs = if let Some(url) = disk_config.get_url() {
                        LocalFilesystem::with_url_prefix(root, url.to_string())
                    } else {
                        LocalFilesystem::new(root)
                    };
                    FilesystemDriver::Local(fs)
                }
                "s3" => {
                    let endpoint = disk_config.endpoint.clone();
                    let region = disk_config.region.as_ref()
                        .ok_or_else(|| FilesystemError::Config {
                            message: "S3 region is required".to_string(),
                        })?;
                    let access_key = disk_config.key.as_ref()
                        .ok_or_else(|| FilesystemError::Config {
                            message: "S3 access key is required".to_string(),
                        })?;
                    let secret_key = disk_config.secret.as_ref()
                        .ok_or_else(|| FilesystemError::Config {
                            message: "S3 secret key is required".to_string(),
                        })?;
                    let bucket = disk_config.bucket.as_ref()
                        .ok_or_else(|| FilesystemError::Config {
                            message: "S3 bucket is required".to_string(),
                        })?;

                    let fs = S3Filesystem::new(
                        endpoint,
                        region.clone(),
                        access_key.clone(),
                        secret_key.clone(),
                        bucket.clone(),
                        disk_config.root.clone(),
                        disk_config.url.clone(),
                    ).await.map_err(|e| FilesystemError::Config {
                        message: format!("Failed to create S3 filesystem: {}", e),
                    })?;

                    let fs = if let Some(visibility) = &disk_config.visibility {
                        fs.with_visibility(visibility.clone())
                    } else {
                        fs
                    };

                    FilesystemDriver::S3(fs)
                }
                driver => {
                    return Err(FilesystemError::Config {
                        message: format!("Unknown driver: {}", driver),
                    }.into());
                }
            };

            self.disks.insert(disk_name.to_string(), filesystem);
        }

        Ok(self.disks.get(disk_name).unwrap())
    }

    pub async fn default_disk(&mut self) -> Result<&FilesystemDriver> {
        self.disk("default").await
    }

    pub fn get_config(&self) -> &StorageConfig {
        &self.config
    }

    pub fn disk_names(&self) -> Vec<&String> {
        self.config.disk_names()
    }

    pub async fn forget(&mut self, disk_name: &str) {
        self.disks.remove(disk_name);
    }

    pub async fn set_default(&mut self, name: String) -> Result<()> {
        if !self.config.disks.contains_key(&name) {
            return Err(FilesystemError::Config {
                message: format!("Disk '{}' does not exist", name),
            }.into());
        }
        self.config.default = name;
        Ok(())
    }
}

// Helper functions that create a new manager each time
pub async fn disk(name: &str) -> Result<FilesystemDriver> {
    let mut manager = StorageManager::new().await?;
    let _disk = manager.disk(name).await?;

    // We need to return an owned value, so we recreate the disk
    let disk_config = manager.config.get_disk(name).ok_or_else(|| {
        FilesystemError::Config {
            message: format!("Disk '{}' not found in configuration", name),
        }
    })?;

    let filesystem = match disk_config.driver.as_str() {
        "local" => {
            let root = disk_config.get_root();
            let fs = if let Some(url) = disk_config.get_url() {
                LocalFilesystem::with_url_prefix(root, url.to_string())
            } else {
                LocalFilesystem::new(root)
            };
            FilesystemDriver::Local(fs)
        }
        "s3" => {
            let endpoint = disk_config.endpoint.clone();
            let region = disk_config.region.as_ref()
                .ok_or_else(|| FilesystemError::Config {
                    message: "S3 region is required".to_string(),
                })?;
            let access_key = disk_config.key.as_ref()
                .ok_or_else(|| FilesystemError::Config {
                    message: "S3 access key is required".to_string(),
                })?;
            let secret_key = disk_config.secret.as_ref()
                .ok_or_else(|| FilesystemError::Config {
                    message: "S3 secret key is required".to_string(),
                })?;
            let bucket = disk_config.bucket.as_ref()
                .ok_or_else(|| FilesystemError::Config {
                    message: "S3 bucket is required".to_string(),
                })?;

            let fs = S3Filesystem::new(
                endpoint,
                region.clone(),
                access_key.clone(),
                secret_key.clone(),
                bucket.clone(),
                disk_config.root.clone(),
                disk_config.url.clone(),
            ).await.map_err(|e| FilesystemError::Config {
                message: format!("Failed to create S3 filesystem: {}", e),
            })?;

            let fs = if let Some(visibility) = &disk_config.visibility {
                fs.with_visibility(visibility.clone())
            } else {
                fs
            };

            FilesystemDriver::S3(fs)
        }
        driver => {
            return Err(FilesystemError::Config {
                message: format!("Unknown driver: {}", driver),
            }.into());
        }
    };

    Ok(filesystem)
}

pub async fn default_disk() -> Result<FilesystemDriver> {
    disk("default").await
}