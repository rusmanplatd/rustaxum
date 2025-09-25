use anyhow::Result;
use serde_json::{Value, Map};
use std::sync::Arc;

use crate::config::session::SessionConfig;
use super::{SessionHandler, Session};
use super::drivers;

#[derive(Clone)]
pub struct SessionManager {
    handler: Arc<dyn SessionHandler>,
    config: SessionConfig,
}

impl SessionManager {
    pub async fn new(
        config: SessionConfig,
        pool: Option<&crate::database::DbPool>,
        redis_pool: Option<()>, // Temporarily disabled
    ) -> Result<Self> {
        let handler = drivers::create_session_handler(
            &config.driver,
            &config,
            pool,
            redis_pool,
        ).await?;

        Ok(Self {
            handler: Arc::from(handler),
            config,
        })
    }

    pub async fn start(&self, session_id: Option<String>) -> Result<Session> {
        let id = session_id.unwrap_or_else(|| self.generate_session_id());

        let mut session = Session::new(id.clone());

        if let Some(data) = self.handler.read(&id).await? {
            if let Ok(json_data) = serde_json::from_str::<Map<String, Value>>(&data) {
                session.data = json_data.into_iter().collect();
            }
        }

        session.is_started = true;
        session.age_flash_data();

        Ok(session)
    }

    pub async fn save(&self, session: &mut Session) -> Result<()> {
        if !session.is_dirty {
            return Ok(());
        }

        let data = serde_json::to_string(&session.data)?;
        self.handler.write(&session.id, &data).await?;

        session.is_dirty = false;
        Ok(())
    }

    pub async fn destroy(&self, session_id: &str) -> Result<()> {
        self.handler.destroy(session_id).await
    }

    pub async fn gc(&self) -> Result<()> {
        self.handler.gc(self.config.lifetime_in_seconds()).await
    }

    pub fn generate_session_id(&self) -> String {
        use rand::{distributions::Alphanumeric, Rng};
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }

    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    pub async fn regenerate(&self, session: &mut Session) -> Result<()> {
        let old_id = session.id.clone();
        session.id = self.generate_session_id();

        self.handler.destroy(&old_id).await?;

        session.is_dirty = true;
        self.save(session).await?;

        Ok(())
    }

    pub async fn migrate(&self, session: &mut Session) -> Result<()> {
        self.regenerate(session).await
    }

    pub async fn invalidate(&self, session: &mut Session) -> Result<()> {
        session.flush();
        self.regenerate(session).await
    }
}