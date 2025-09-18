use async_trait::async_trait;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::storage::filesystem::{FileInfo, Filesystem, FilesystemError};

#[derive(Debug)]
pub struct LocalFilesystem {
    root: PathBuf,
    url_prefix: Option<String>,
    visibility: String,
}

impl LocalFilesystem {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            url_prefix: None,
            visibility: "private".to_string(),
        }
    }

    pub fn with_url_prefix<P: AsRef<Path>>(root: P, url_prefix: String) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            url_prefix: Some(url_prefix),
            visibility: "public".to_string(),
        }
    }

    pub fn set_visibility(&mut self, visibility: String) {
        self.visibility = visibility;
    }

    fn resolve_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = path.as_ref();
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        }
    }

    async fn ensure_directory_exists<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await?;
            }
        }
        Ok(())
    }

    fn list_entries_recursive<'a>(&'a self, directory: &'a std::path::Path, files_only: bool, recursive: bool, entries: &'a mut Vec<String>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            if !directory.exists() {
                return Err(FilesystemError::DirectoryNotFound {
                    path: directory.to_string_lossy().to_string(),
                }.into());
            }

            let mut read_dir = fs::read_dir(directory).await?;

            while let Some(entry) = read_dir.next_entry().await? {
                let entry_path = entry.path();
                let relative_path = entry_path.strip_prefix(&self.root)
                    .map_err(|_| FilesystemError::InvalidPath {
                        path: entry_path.to_string_lossy().to_string(),
                    })?;

                let metadata = entry.metadata().await?;

                if files_only && metadata.is_dir() {
                    if recursive {
                        self.list_entries_recursive(&entry_path, files_only, recursive, entries).await?;
                    }
                    continue;
                }

                if !files_only && metadata.is_file() {
                    continue;
                }

                entries.push(relative_path.to_string_lossy().to_string());

                if recursive && metadata.is_dir() {
                    self.list_entries_recursive(&entry_path, files_only, recursive, entries).await?;
                }
            }

            Ok(())
        })
    }

    async fn list_entries<P: AsRef<Path>>(&self, directory: P, files_only: bool, recursive: bool) -> Result<Vec<String>> {
        let full_path = self.resolve_path(directory);
        let mut entries = Vec::new();
        self.list_entries_recursive(&full_path, files_only, recursive, &mut entries).await?;
        Ok(entries)
    }
}

#[async_trait]
impl Filesystem for LocalFilesystem {
    async fn exists(&self, path: &str) -> Result<bool> {
        let full_path = self.resolve_path(path);
        Ok(full_path.exists())
    }

    async fn get(&self, path: &str) -> Result<Vec<u8>> {
        let full_path = self.resolve_path(path);

        if !full_path.exists() {
            return Err(FilesystemError::FileNotFound {
                path: full_path.to_string_lossy().to_string(),
            }.into());
        }

        let contents = fs::read(&full_path).await?;
        Ok(contents)
    }

    async fn put(&self, path: &str, contents: &[u8]) -> Result<()> {
        let full_path = self.resolve_path(path);
        self.ensure_directory_exists(&full_path).await?;
        fs::write(&full_path, contents).await?;
        Ok(())
    }

    async fn put_file_as(
        &self,
        path: &str,
        file: &str,
        name: Option<String>
    ) -> Result<String> {
        let source_path = Path::new(file);
        let dest_name = name.unwrap_or_else(|| {
            source_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "file".to_string())
        });

        let dest_path = Path::new(path).join(&dest_name);
        let full_dest_path = self.resolve_path(&dest_path);

        self.ensure_directory_exists(&full_dest_path).await?;
        fs::copy(source_path, &full_dest_path).await?;

        Ok(dest_path.to_string_lossy().to_string())
    }

    async fn prepend(&self, path: &str, data: &[u8]) -> Result<()> {
        let full_path = self.resolve_path(path);

        let existing_content = if full_path.exists() {
            fs::read(&full_path).await?
        } else {
            Vec::new()
        };

        let mut new_content = data.to_vec();
        new_content.extend_from_slice(&existing_content);

        self.ensure_directory_exists(&full_path).await?;
        fs::write(&full_path, new_content).await?;
        Ok(())
    }

    async fn append(&self, path: &str, data: &[u8]) -> Result<()> {
        let full_path = self.resolve_path(path);
        self.ensure_directory_exists(&full_path).await?;

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&full_path)
            .await?;

        file.write_all(data).await?;
        Ok(())
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.resolve_path(path);

        if !full_path.exists() {
            return Err(FilesystemError::FileNotFound {
                path: full_path.to_string_lossy().to_string(),
            }.into());
        }

        fs::remove_file(&full_path).await?;
        Ok(())
    }

    async fn copy(&self, from: &str, to: &str) -> Result<()> {
        let from_path = self.resolve_path(from);
        let to_path = self.resolve_path(to);

        if !from_path.exists() {
            return Err(FilesystemError::FileNotFound {
                path: from_path.to_string_lossy().to_string(),
            }.into());
        }

        self.ensure_directory_exists(&to_path).await?;
        fs::copy(&from_path, &to_path).await?;
        Ok(())
    }

    async fn move_file(&self, from: &str, to: &str) -> Result<()> {
        let from_path = self.resolve_path(from);
        let to_path = self.resolve_path(to);

        if !from_path.exists() {
            return Err(FilesystemError::FileNotFound {
                path: from_path.to_string_lossy().to_string(),
            }.into());
        }

        self.ensure_directory_exists(&to_path).await?;
        fs::rename(&from_path, &to_path).await?;
        Ok(())
    }

    async fn size(&self, path: &str) -> Result<u64> {
        let full_path = self.resolve_path(path);

        if !full_path.exists() {
            return Err(FilesystemError::FileNotFound {
                path: full_path.to_string_lossy().to_string(),
            }.into());
        }

        let metadata = fs::metadata(&full_path).await?;
        Ok(metadata.len())
    }

    async fn last_modified(&self, path: &str) -> Result<DateTime<Utc>> {
        let full_path = self.resolve_path(path);

        if !full_path.exists() {
            return Err(FilesystemError::FileNotFound {
                path: full_path.to_string_lossy().to_string(),
            }.into());
        }

        let metadata = fs::metadata(&full_path).await?;
        let modified = metadata.modified()?;
        let datetime: DateTime<Utc> = modified.into();
        Ok(datetime)
    }

    async fn files(&self, directory: &str) -> Result<Vec<String>> {
        self.list_entries(directory, true, false).await
    }

    async fn all_files(&self, directory: &str) -> Result<Vec<String>> {
        self.list_entries(directory, true, true).await
    }

    async fn directories(&self, directory: &str) -> Result<Vec<String>> {
        self.list_entries(directory, false, false).await
    }

    async fn all_directories(&self, directory: &str) -> Result<Vec<String>> {
        self.list_entries(directory, false, true).await
    }

    async fn make_directory(&self, path: &str) -> Result<()> {
        let full_path = self.resolve_path(path);
        fs::create_dir_all(&full_path).await?;
        Ok(())
    }

    async fn delete_directory(&self, directory: &str) -> Result<()> {
        let full_path = self.resolve_path(directory);

        if !full_path.exists() {
            return Err(FilesystemError::DirectoryNotFound {
                path: full_path.to_string_lossy().to_string(),
            }.into());
        }

        fs::remove_dir_all(&full_path).await?;
        Ok(())
    }

    async fn url(&self, path: &str) -> Result<Option<String>> {
        if let Some(ref prefix) = self.url_prefix {
            Ok(Some(format!("{}/{}", prefix.trim_end_matches('/'), path.trim_start_matches('/'))))
        } else {
            Ok(None)
        }
    }

    async fn temporary_url(
        &self,
        path: &str,
        _expires_at: DateTime<Utc>
    ) -> Result<String> {
        // For local filesystem, temporary URLs are the same as regular URLs
        // In a real implementation, you might want to generate signed URLs with expiration
        self.url(path).await?.ok_or_else(|| {
            FilesystemError::Config {
                message: "No URL prefix configured for local filesystem".to_string(),
            }.into()
        })
    }

    async fn get_info(&self, path: &str) -> Result<FileInfo> {
        let full_path = self.resolve_path(path);

        if !full_path.exists() {
            return Err(FilesystemError::FileNotFound {
                path: full_path.to_string_lossy().to_string(),
            }.into());
        }

        let metadata = fs::metadata(&full_path).await?;
        let modified: DateTime<Utc> = metadata.modified()?.into();

        if metadata.is_file() {
            Ok(FileInfo::new(path.to_string(), metadata.len(), modified))
        } else {
            Ok(FileInfo::directory(path.to_string(), modified))
        }
    }

    async fn set_visibility(&self, _path: &str, _visibility: &str) -> Result<()> {
        // Local filesystem doesn't support visibility in the same way as cloud storage
        // This could be extended to set file permissions
        Ok(())
    }

    async fn get_visibility(&self, _path: &str) -> Result<String> {
        Ok(self.visibility.clone())
    }

    fn name(&self) -> &str {
        "local"
    }

    fn is_local(&self) -> bool {
        true
    }
}