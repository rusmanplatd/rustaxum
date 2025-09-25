use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::services::session::SessionHandler;

pub struct FileSessionHandler {
    path: PathBuf,
}

impl FileSessionHandler {
    pub fn new(path: &str) -> Result<Self> {
        let path = PathBuf::from(path);
        std::fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn session_file_path(&self, session_id: &str) -> PathBuf {
        self.path.join(format!("sess_{}", session_id))
    }
}

#[async_trait]
impl SessionHandler for FileSessionHandler {
    async fn read(&self, session_id: &str) -> Result<Option<String>> {
        let file_path = self.session_file_path(session_id);

        if !file_path.exists() {
            return Ok(None);
        }

        match fs::read_to_string(&file_path).await {
            Ok(content) => {
                let parts: Vec<&str> = content.splitn(2, '\n').collect();
                if parts.len() != 2 {
                    return Ok(None);
                }

                let timestamp: u64 = parts[0].parse().unwrap_or(0);
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                if current_time > timestamp {
                    fs::remove_file(&file_path).await.ok();
                    return Ok(None);
                }

                Ok(Some(parts[1].to_string()))
            }
            Err(_) => Ok(None),
        }
    }

    async fn write(&self, session_id: &str, data: &str) -> Result<()> {
        let file_path = self.session_file_path(session_id);

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let lifetime = 7200; // 2 hours default
        let expires = current_time + lifetime;
        let content = format!("{}\n{}", expires, data);

        fs::write(&file_path, content).await?;
        Ok(())
    }

    async fn destroy(&self, session_id: &str) -> Result<()> {
        let file_path = self.session_file_path(session_id);
        if file_path.exists() {
            fs::remove_file(&file_path).await?;
        }
        Ok(())
    }

    async fn gc(&self, _lifetime: u64) -> Result<()> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut entries = fs::read_dir(&self.path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.starts_with("sess_") {
                    let file_path = entry.path();

                    match fs::read_to_string(&file_path).await {
                        Ok(content) => {
                            let parts: Vec<&str> = content.splitn(2, '\n').collect();
                            if parts.len() == 2 {
                                let timestamp: u64 = parts[0].parse().unwrap_or(0);
                                if current_time > timestamp {
                                    fs::remove_file(&file_path).await.ok();
                                }
                            }
                        }
                        Err(_) => {
                            fs::remove_file(&file_path).await.ok();
                        }
                    }
                }
            }
        }

        Ok(())
    }
}