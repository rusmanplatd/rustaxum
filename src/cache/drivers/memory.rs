use crate::cache::{Cache, CacheError};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
struct CacheEntry {
    value: String,
    expires_at: Option<Instant>,
}

impl CacheEntry {
    fn new(value: String, ttl: Option<Duration>) -> Self {
        let expires_at = ttl.map(|duration| Instant::now() + duration);
        Self { value, expires_at }
    }

    fn is_expired(&self) -> bool {
        self.expires_at
            .map(|expires_at| Instant::now() > expires_at)
            .unwrap_or(false)
    }
}

#[derive(Clone)]
pub struct MemoryCache {
    store: Arc<RwLock<HashMap<String, CacheEntry>>>,
    prefix: Option<String>,
    name: String,
}

impl MemoryCache {
    pub fn new(prefix: Option<String>) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            prefix,
            name: "memory".to_string(),
        }
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

    async fn cleanup_expired(&self) {
        let mut store = self.store.write().await;
        store.retain(|_, entry| !entry.is_expired());
    }
}

#[async_trait]
impl Cache for MemoryCache {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        let cache_key = self.build_key(key);
        let store = self.store.read().await;

        match store.get(&cache_key) {
            Some(entry) if !entry.is_expired() => {
                let deserialized = self.deserialize_value(&entry.value).await?;
                Ok(Some(deserialized))
            }
            _ => Ok(None),
        }
    }

    async fn put<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        let cache_key = self.build_key(key);
        let serialized = self.serialize_value(value).await?;
        let entry = CacheEntry::new(serialized, ttl);

        let mut store = self.store.write().await;
        store.insert(cache_key, entry);

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
        let store = self.store.read().await;

        match store.get(&cache_key) {
            Some(entry) => Ok(!entry.is_expired()),
            None => Ok(false),
        }
    }

    async fn forget(&self, key: &str) -> Result<bool> {
        let cache_key = self.build_key(key);
        let mut store = self.store.write().await;

        Ok(store.remove(&cache_key).is_some())
    }

    async fn flush(&self) -> Result<()> {
        let mut store = self.store.write().await;

        if let Some(prefix) = &self.prefix {
            // Remove only keys with the prefix
            let prefix_pattern = format!("{}:", prefix);
            store.retain(|key, _| !key.starts_with(&prefix_pattern));
        } else {
            // Clear everything
            store.clear();
        }

        Ok(())
    }

    async fn many<T>(&self, keys: &[&str]) -> Result<Vec<(String, Option<T>)>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        let store = self.store.read().await;
        let mut results = Vec::new();

        for key in keys {
            let cache_key = self.build_key(key);
            let value = match store.get(&cache_key) {
                Some(entry) if !entry.is_expired() => {
                    Some(self.deserialize_value(&entry.value).await?)
                }
                _ => None,
            };
            results.push((key.to_string(), value));
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
        let mut store = self.store.write().await;

        let new_value = match store.get(&cache_key) {
            Some(entry) if !entry.is_expired() => {
                // Try to parse the existing value as i64
                let current: i64 = entry.value.parse().map_err(|_| CacheError::Operation {
                    message: format!("Cannot increment non-numeric value for key '{}'", cache_key),
                })?;
                current + value
            }
            _ => value, // Key doesn't exist or is expired, start with the increment value
        };

        let entry = CacheEntry::new(new_value.to_string(), None);
        store.insert(cache_key, entry);

        Ok(new_value)
    }

    async fn decrement(&self, key: &str, value: i64) -> Result<i64> {
        self.increment(key, -value).await
    }

    async fn add<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<bool>
    where
        T: Serialize + Send + Sync,
    {
        let cache_key = self.build_key(key);
        let mut store = self.store.write().await;

        // Check if key exists and is not expired
        match store.get(&cache_key) {
            Some(entry) if !entry.is_expired() => Ok(false), // Key exists
            _ => {
                // Key doesn't exist or is expired, add it
                let serialized = self.serialize_value(value).await?;
                let entry = CacheEntry::new(serialized, ttl);
                store.insert(cache_key, entry);
                Ok(true)
            }
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new(None)
    }
}

// Background task to periodically clean up expired entries
impl MemoryCache {
    pub async fn start_cleanup_task(self) -> Self {
        let cache = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Clean up every minute
            loop {
                interval.tick().await;
                cache.cleanup_expired().await;
            }
        });
        self
    }
}