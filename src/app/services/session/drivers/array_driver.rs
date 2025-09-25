use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::services::session::SessionHandler;

#[derive(Debug, Clone)]
struct SessionData {
    data: String,
    expires_at: u64,
}

#[derive(Debug, Clone)]
pub struct ArraySessionHandler {
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
}

impl ArraySessionHandler {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[async_trait]
impl SessionHandler for ArraySessionHandler {
    async fn read(&self, session_id: &str) -> Result<Option<String>> {
        let sessions = self.sessions.read().await;

        if let Some(session_data) = sessions.get(session_id) {
            if self.current_timestamp() <= session_data.expires_at {
                return Ok(Some(session_data.data.clone()));
            }
        }

        Ok(None)
    }

    async fn write(&self, session_id: &str, data: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let expires_at = self.current_timestamp() + 7200; // 2 hours from now

        sessions.insert(
            session_id.to_string(),
            SessionData {
                data: data.to_string(),
                expires_at,
            },
        );

        Ok(())
    }

    async fn destroy(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }

    async fn gc(&self, _lifetime: u64) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let current_time = self.current_timestamp();

        sessions.retain(|_, session_data| current_time <= session_data.expires_at);

        Ok(())
    }
}