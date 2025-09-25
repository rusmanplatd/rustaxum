use anyhow::Result;
use async_trait::async_trait;
use redis::{AsyncCommands, Client};

use crate::app::services::session::SessionHandler;

pub struct RedisSessionHandler {
    client: Client,
}

impl RedisSessionHandler {
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| anyhow::anyhow!("Failed to create Redis client: {}", e))?;

        Ok(Self { client })
    }

    fn session_key(&self, session_id: &str) -> String {
        format!("session:{}", session_id)
    }
}

#[async_trait]
impl SessionHandler for RedisSessionHandler {
    async fn read(&self, session_id: &str) -> Result<Option<String>> {
        let mut conn = self.client.get_multiplexed_async_connection().await
            .map_err(|e| anyhow::anyhow!("Failed to get Redis connection: {}", e))?;
        let key = self.session_key(session_id);

        match conn.get::<_, Option<String>>(&key).await {
            Ok(data) => Ok(data),
            Err(e) => Err(anyhow::anyhow!("Failed to read session from Redis: {}", e)),
        }
    }

    async fn write(&self, session_id: &str, data: &str) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await
            .map_err(|e| anyhow::anyhow!("Failed to get Redis connection: {}", e))?;
        let key = self.session_key(session_id);

        conn.set_ex::<_, _, ()>(&key, data, 7200).await // 2 hours TTL
            .map_err(|e| anyhow::anyhow!("Failed to write session to Redis: {}", e))?;

        Ok(())
    }

    async fn destroy(&self, session_id: &str) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await
            .map_err(|e| anyhow::anyhow!("Failed to get Redis connection: {}", e))?;
        let key = self.session_key(session_id);

        conn.del::<_, ()>(&key).await
            .map_err(|e| anyhow::anyhow!("Failed to destroy session in Redis: {}", e))?;

        Ok(())
    }

    async fn gc(&self, _lifetime: u64) -> Result<()> {
        // Redis handles expiration automatically with TTL
        Ok(())
    }
}