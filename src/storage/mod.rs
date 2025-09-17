pub mod filesystem;
pub mod drivers;
pub mod manager;
pub mod macros;

pub use filesystem::{FileInfo, FilesystemError};
pub use drivers::LocalFilesystem;
pub use manager::{StorageManager, FilesystemDriver};

use anyhow::Result;
use std::path::Path;

pub async fn put<P: AsRef<Path>>(path: P, contents: &[u8]) -> Result<()> {
    let disk = manager::default_disk().await?;
    disk.put(path.as_ref().to_str().unwrap(), contents).await
}

pub async fn get<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let disk = manager::default_disk().await?;
    disk.get(path.as_ref().to_str().unwrap()).await
}

pub async fn exists<P: AsRef<Path>>(path: P) -> Result<bool> {
    let disk = manager::default_disk().await?;
    disk.exists(path.as_ref().to_str().unwrap()).await
}

pub async fn delete<P: AsRef<Path>>(path: P) -> Result<()> {
    let disk = manager::default_disk().await?;
    disk.delete(path.as_ref().to_str().unwrap()).await
}

pub async fn copy<P: AsRef<Path>>(from: P, to: P) -> Result<()> {
    let disk = manager::default_disk().await?;
    disk.copy(
        from.as_ref().to_str().unwrap(),
        to.as_ref().to_str().unwrap()
    ).await
}

pub async fn move_file<P: AsRef<Path>>(from: P, to: P) -> Result<()> {
    let disk = manager::default_disk().await?;
    disk.move_file(
        from.as_ref().to_str().unwrap(),
        to.as_ref().to_str().unwrap()
    ).await
}

pub async fn size<P: AsRef<Path>>(path: P) -> Result<u64> {
    let disk = manager::default_disk().await?;
    disk.size(path.as_ref().to_str().unwrap()).await
}

pub async fn last_modified<P: AsRef<Path>>(path: P) -> Result<chrono::DateTime<chrono::Utc>> {
    let disk = manager::default_disk().await?;
    disk.last_modified(path.as_ref().to_str().unwrap()).await
}

pub async fn url<P: AsRef<Path>>(path: P) -> Result<Option<String>> {
    let disk = manager::default_disk().await?;
    disk.url(path.as_ref().to_str().unwrap()).await
}

pub async fn files<P: AsRef<Path>>(directory: P) -> Result<Vec<String>> {
    let disk = manager::default_disk().await?;
    disk.files(directory.as_ref().to_str().unwrap()).await
}

pub async fn directories<P: AsRef<Path>>(directory: P) -> Result<Vec<String>> {
    let disk = manager::default_disk().await?;
    disk.directories(directory.as_ref().to_str().unwrap()).await
}

pub async fn make_directory<P: AsRef<Path>>(path: P) -> Result<()> {
    let disk = manager::default_disk().await?;
    disk.make_directory(path.as_ref().to_str().unwrap()).await
}

pub async fn delete_directory<P: AsRef<Path>>(directory: P) -> Result<()> {
    let disk = manager::default_disk().await?;
    disk.delete_directory(directory.as_ref().to_str().unwrap()).await
}