use anyhow::Result;
use async_trait::async_trait;
use diesel::prelude::*;

use crate::app::services::session::SessionHandler;
use crate::app::models::session::{SessionModel, UpdateSession};
use crate::schema::sessions;
use crate::database::DbPool;

pub struct DatabaseSessionHandler {
    pool: DbPool,
}

impl DatabaseSessionHandler {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionHandler for DatabaseSessionHandler {
    async fn read(&self, session_id: &str) -> Result<Option<String>> {
        let conn = &mut self.pool.get()
            .map_err(|e| anyhow::anyhow!("Failed to get database connection: {}", e))?;
        let current_time = chrono::Utc::now().timestamp() as i32;
        let cutoff_time = current_time - 7200; // 2 hours ago

        let result = sessions::table
            .filter(sessions::id.eq(session_id))
            .filter(sessions::last_activity.gt(cutoff_time))
            .select(SessionModel::as_select())
            .first::<SessionModel>(conn)
            .optional();

        match result {
            Ok(Some(session)) => Ok(Some(session.payload)),
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Failed to read session: {}", e)),
        }
    }

    async fn write(&self, session_id: &str, data: &str) -> Result<()> {
        let conn = &mut self.pool.get()
            .map_err(|e| anyhow::anyhow!("Failed to get database connection: {}", e))?;
        let current_time = chrono::Utc::now().timestamp() as i32;

        // Try to update existing session
        let updated = diesel::update(sessions::table.filter(sessions::id.eq(session_id)))
            .set(&UpdateSession::new(data.to_string(), None, None, None))
            .execute(conn)
            .map_err(|e| anyhow::anyhow!("Failed to update session: {}", e))?;

        // If no rows were updated, insert a new session
        if updated == 0 {
            let new_session = SessionModel {
                id: session_id.to_string(),
                user_id: None,
                ip_address: None,
                user_agent: None,
                payload: data.to_string(),
                last_activity: current_time,
            };

            diesel::insert_into(sessions::table)
                .values(&new_session)
                .execute(conn)
                .map_err(|e| anyhow::anyhow!("Failed to insert session: {}", e))?;
        }

        Ok(())
    }

    async fn destroy(&self, session_id: &str) -> Result<()> {
        let conn = &mut self.pool.get()
            .map_err(|e| anyhow::anyhow!("Failed to get database connection: {}", e))?;

        diesel::delete(sessions::table.filter(sessions::id.eq(session_id)))
            .execute(conn)
            .map_err(|e| anyhow::anyhow!("Failed to destroy session: {}", e))?;

        Ok(())
    }

    async fn gc(&self, lifetime: u64) -> Result<()> {
        let conn = &mut self.pool.get()
            .map_err(|e| anyhow::anyhow!("Failed to get database connection: {}", e))?;
        let cutoff_time = chrono::Utc::now().timestamp() as i32 - lifetime as i32;

        diesel::delete(sessions::table.filter(sessions::last_activity.lt(cutoff_time)))
            .execute(conn)
            .map_err(|e| anyhow::anyhow!("Failed to garbage collect sessions: {}", e))?;

        Ok(())
    }
}