use async_trait::async_trait;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FilesystemError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Directory not found: {path}")]
    DirectoryNotFound { path: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("Invalid path: {path}")]
    InvalidPath { path: String },

    #[error("Disk full")]
    DiskFull,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Network error: {message}")]
    Network { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub last_modified: DateTime<Utc>,
    pub mime_type: Option<String>,
    pub is_file: bool,
    pub is_directory: bool,
}

impl FileInfo {
    pub fn new(path: String, size: u64, last_modified: DateTime<Utc>) -> Self {
        let mime_type = if path.contains('.') {
            mime_guess::from_path(&path).first().map(|m| m.to_string())
        } else {
            None
        };

        Self {
            path,
            size,
            last_modified,
            mime_type,
            is_file: true,
            is_directory: false,
        }
    }

    pub fn directory(path: String, last_modified: DateTime<Utc>) -> Self {
        Self {
            path,
            size: 0,
            last_modified,
            mime_type: None,
            is_file: false,
            is_directory: true,
        }
    }
}

#[async_trait]
pub trait Filesystem: Send + Sync {
    async fn exists(&self, path: &str) -> Result<bool>;

    async fn get(&self, path: &str) -> Result<Vec<u8>>;

    async fn put(&self, path: &str, contents: &[u8]) -> Result<()>;

    async fn put_file_as(
        &self,
        path: &str,
        file: &str,
        name: Option<String>
    ) -> Result<String>;

    async fn prepend(&self, path: &str, data: &[u8]) -> Result<()>;

    async fn append(&self, path: &str, data: &[u8]) -> Result<()>;

    async fn delete(&self, path: &str) -> Result<()>;

    async fn copy(&self, from: &str, to: &str) -> Result<()>;

    async fn move_file(&self, from: &str, to: &str) -> Result<()>;

    async fn size(&self, path: &str) -> Result<u64>;

    async fn last_modified(&self, path: &str) -> Result<DateTime<Utc>>;

    async fn files(&self, directory: &str) -> Result<Vec<String>>;

    async fn all_files(&self, directory: &str) -> Result<Vec<String>>;

    async fn directories(&self, directory: &str) -> Result<Vec<String>>;

    async fn all_directories(&self, directory: &str) -> Result<Vec<String>>;

    async fn make_directory(&self, path: &str) -> Result<()>;

    async fn delete_directory(&self, directory: &str) -> Result<()>;

    async fn url(&self, path: &str) -> Result<Option<String>>;

    async fn temporary_url(
        &self,
        path: &str,
        expires_at: DateTime<Utc>
    ) -> Result<String>;

    async fn get_info(&self, path: &str) -> Result<FileInfo>;

    async fn set_visibility(&self, path: &str, visibility: &str) -> Result<()>;

    async fn get_visibility(&self, path: &str) -> Result<String>;

    fn name(&self) -> &str;

    fn is_local(&self) -> bool {
        false
    }

    fn is_cloud(&self) -> bool {
        !self.is_local()
    }
}