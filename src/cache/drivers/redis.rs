use crate::cache::{Cache, CacheError};
use anyhow::Result;
use async_trait::async_trait;
use redis::{AsyncCommands, Client};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct RedisCache {
    connection: ConnectionManager,
    prefix: Option<String>,
    name: String,
}

impl RedisCache {
    pub async fn new(url: &str, prefix: Option<String>) -> Result<Self> {
        let client = Client::open(url).map_err(|e| CacheError::Connection {
            message: format!("Failed to create Redis client: {}", e),
        })?;

        let connection = client
            .get_connection_manager()
            .await
            .map_err(|e| CacheError::Connection {
                message: format!("Failed to establish Redis connection: {}", e),
            })?;

        Ok(Self {
            connection,
            prefix,
            name: "redis".to_string(),
        })
    }

    pub fn with_prefix(mut self, prefix: String) -> Self {
        self.prefix = Some(prefix);
        self
    }

    fn build_key(&self, key: &str) -> String {
        match &self.prefix {
            Some(prefix) => format!("{}:{}", prefix, key),
            None => key.to_string(),
        }
    }

    async fn serialize_value<T: Serialize>(&self, value: &T) -> Result<String> {
        serde_json::to_string(value).map_err(|e| CacheError::Serialization {
            message: e.to_string(),
        }.into())
    }

    async fn deserialize_value<T: for<'de> Deserialize<'de>>(&self, value: &str) -> Result<T> {
        serde_json::from_str(value).map_err(|e| CacheError::Deserialization {
            message: e.to_string(),
        }.into())
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        let cache_key = self.build_key(key);
        let mut conn = self.connection.clone();

        let value: Option<String> = conn.get(&cache_key).await.map_err(|e| CacheError::Operation {
            message: format!("Failed to get key '{}': {}", cache_key, e),
        })?;

        match value {
            Some(json_str) => {
                let deserialized = self.deserialize_value(&json_str).await?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    async fn put<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        let cache_key = self.build_key(key);
        let serialized = self.serialize_value(value).await?;
        let mut conn = self.connection.clone();

        match ttl {
            Some(duration) => {
                conn.set_ex::<_, _, ()>(&cache_key, &serialized, duration.as_secs()).await
            }
            None => conn.set::<_, _, ()>(&cache_key, &serialized).await,
        }
        .map_err(|e| CacheError::Operation {
            message: format!("Failed to set key '{}': {}", cache_key, e),
        })?;

        Ok(())
    }

    async fn forever<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        self.put(key, value, None).await
    }

    async fn remember<T, F, Fut>(&self, key: &str, ttl: Option<Duration>, callback: F) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + Serialize + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        if let Some(cached_value) = self.get(key).await? {
            return Ok(cached_value);
        }

        let value = callback().await?;
        self.put(key, &value, ttl).await?;
        Ok(value)
    }

    async fn remember_forever<T, F, Fut>(&self, key: &str, callback: F) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + Serialize + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        self.remember(key, None, callback).await
    }

    async fn has(&self, key: &str) -> Result<bool> {
        let cache_key = self.build_key(key);
        let mut conn = self.connection.clone();

        let exists: bool = conn.exists(&cache_key).await.map_err(|e| CacheError::Operation {
            message: format!("Failed to check existence of key '{}': {}", cache_key, e),
        })?;

        Ok(exists)
    }

    async fn forget(&self, key: &str) -> Result<bool> {
        let cache_key = self.build_key(key);
        let mut conn = self.connection.clone();

        let deleted: i32 = conn.del(&cache_key).await.map_err(|e| CacheError::Operation {
            message: format!("Failed to delete key '{}': {}", cache_key, e),
        })?;

        Ok(deleted > 0)
    }

    async fn flush(&self) -> Result<()> {
        let mut conn = self.connection.clone();

        if let Some(prefix) = &self.prefix {
            // Delete all keys with the prefix
            let pattern = format!("{}:*", prefix);
            let keys: Vec<String> = conn.keys(&pattern).await.map_err(|e| CacheError::Operation {
                message: format!("Failed to get keys with pattern '{}': {}", pattern, e),
            })?;

            if !keys.is_empty() {
                conn.del::<_, ()>(&keys).await.map_err(|e| CacheError::Operation {
                    message: format!("Failed to delete keys: {}", e),
                })?;
            }
        } else {
            // Flush the entire database (use with caution!)
            conn.flushdb::<()>().await.map_err(|e| CacheError::Operation {
                message: format!("Failed to flush database: {}", e),
            })?;
        }

        Ok(())
    }

    async fn many<T>(&self, keys: &[&str]) -> Result<Vec<(String, Option<T>)>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        let cache_keys: Vec<String> = keys.iter().map(|k| self.build_key(k)).collect();
        let mut conn = self.connection.clone();

        let values: Vec<Option<String>> = conn.mget(&cache_keys).await.map_err(|e| CacheError::Operation {
            message: format!("Failed to get multiple keys: {}", e),
        })?;

        let mut results = Vec::new();
        for (i, value) in values.into_iter().enumerate() {
            let key = keys[i].to_string();
            match value {
                Some(json_str) => {
                    let deserialized = self.deserialize_value(&json_str).await?;
                    results.push((key, Some(deserialized)));
                }
                None => results.push((key, None)),
            }
        }

        Ok(results)
    }

    async fn put_many<T>(&self, values: &[(&str, &T)], ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        for (key, value) in values {
            self.put(key, value, ttl).await?;
        }

        Ok(())
    }

    async fn increment(&self, key: &str, value: i64) -> Result<i64> {
        let cache_key = self.build_key(key);
        let mut conn = self.connection.clone();

        let result: i64 = conn.incr(&cache_key, value).await.map_err(|e| CacheError::Operation {
            message: format!("Failed to increment key '{}': {}", cache_key, e),
        })?;

        Ok(result)
    }

    async fn decrement(&self, key: &str, value: i64) -> Result<i64> {
        self.increment(key, -value).await
    }

    async fn add<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<bool>
    where
        T: Serialize + Send + Sync,
    {
        let cache_key = self.build_key(key);
        let serialized = self.serialize_value(value).await?;
        let mut conn = self.connection.clone();

        let result: bool = match ttl {
            Some(duration) => {
                // Use SET with NX (only if not exists) and EX (expiration)
                let result: Option<String> = conn
                    .set_options(&cache_key, &serialized, redis::SetOptions::default()
                        .conditional_set(redis::ExistenceCheck::NX)
                        .get(true)
                        .with_expiration(redis::SetExpiry::EX(duration.as_secs())))
                    .await
                    .map_err(|e| CacheError::Operation {
                        message: format!("Failed to add key '{}': {}", cache_key, e),
                    })?;
                result.is_none() // None means the key was set (didn't exist before)
            }
            None => {
                // Use SET with NX (only if not exists)
                let result: Option<String> = conn
                    .set_options(&cache_key, &serialized, redis::SetOptions::default()
                        .conditional_set(redis::ExistenceCheck::NX)
                        .get(true))
                    .await
                    .map_err(|e| CacheError::Operation {
                        message: format!("Failed to add key '{}': {}", cache_key, e),
                    })?;
                result.is_none() // None means the key was set (didn't exist before)
            }
        };

        Ok(result)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
}