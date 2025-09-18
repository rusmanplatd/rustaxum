use crate::cache::drivers::{MemoryCache, RedisCache};
use crate::cache::{Cache, CacheError};
use crate::config::{cache::CacheConfig, Config};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub enum CacheDriver {
    Memory(Arc<MemoryCache>),
    Redis(Arc<RedisCache>),
}

#[async_trait]
impl Cache for CacheDriver {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        match self {
            CacheDriver::Memory(cache) => cache.get(key).await,
            CacheDriver::Redis(cache) => cache.get(key).await,
        }
    }

    async fn put<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        match self {
            CacheDriver::Memory(cache) => cache.put(key, value, ttl).await,
            CacheDriver::Redis(cache) => cache.put(key, value, ttl).await,
        }
    }

    async fn forever<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        match self {
            CacheDriver::Memory(cache) => cache.forever(key, value).await,
            CacheDriver::Redis(cache) => cache.forever(key, value).await,
        }
    }

    async fn remember<T, F, Fut>(&self, key: &str, ttl: Option<Duration>, callback: F) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + Serialize + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        match self {
            CacheDriver::Memory(cache) => cache.remember(key, ttl, callback).await,
            CacheDriver::Redis(cache) => cache.remember(key, ttl, callback).await,
        }
    }

    async fn remember_forever<T, F, Fut>(&self, key: &str, callback: F) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + Serialize + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        match self {
            CacheDriver::Memory(cache) => cache.remember_forever(key, callback).await,
            CacheDriver::Redis(cache) => cache.remember_forever(key, callback).await,
        }
    }

    async fn has(&self, key: &str) -> Result<bool> {
        match self {
            CacheDriver::Memory(cache) => cache.has(key).await,
            CacheDriver::Redis(cache) => cache.has(key).await,
        }
    }

    async fn forget(&self, key: &str) -> Result<bool> {
        match self {
            CacheDriver::Memory(cache) => cache.forget(key).await,
            CacheDriver::Redis(cache) => cache.forget(key).await,
        }
    }

    async fn flush(&self) -> Result<()> {
        match self {
            CacheDriver::Memory(cache) => cache.flush().await,
            CacheDriver::Redis(cache) => cache.flush().await,
        }
    }

    async fn many<T>(&self, keys: &[&str]) -> Result<Vec<(String, Option<T>)>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        match self {
            CacheDriver::Memory(cache) => cache.many(keys).await,
            CacheDriver::Redis(cache) => cache.many(keys).await,
        }
    }

    async fn put_many<T>(&self, values: &[(&str, &T)], ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        match self {
            CacheDriver::Memory(cache) => cache.put_many(values, ttl).await,
            CacheDriver::Redis(cache) => cache.put_many(values, ttl).await,
        }
    }

    async fn increment(&self, key: &str, value: i64) -> Result<i64> {
        match self {
            CacheDriver::Memory(cache) => cache.increment(key, value).await,
            CacheDriver::Redis(cache) => cache.increment(key, value).await,
        }
    }

    async fn decrement(&self, key: &str, value: i64) -> Result<i64> {
        match self {
            CacheDriver::Memory(cache) => cache.decrement(key, value).await,
            CacheDriver::Redis(cache) => cache.decrement(key, value).await,
        }
    }

    async fn add<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<bool>
    where
        T: Serialize + Send + Sync,
    {
        match self {
            CacheDriver::Memory(cache) => cache.add(key, value, ttl).await,
            CacheDriver::Redis(cache) => cache.add(key, value, ttl).await,
        }
    }

    fn name(&self) -> &str {
        match self {
            CacheDriver::Memory(cache) => cache.name(),
            CacheDriver::Redis(cache) => cache.name(),
        }
    }

    fn prefix(&self) -> Option<&str> {
        match self {
            CacheDriver::Memory(cache) => cache.prefix(),
            CacheDriver::Redis(cache) => cache.prefix(),
        }
    }
}

pub struct CacheManager {
    config: CacheConfig,
    stores: HashMap<String, CacheDriver>,
}

impl CacheManager {
    pub async fn new() -> Result<Self> {
        let config = Config::load()?;
        Ok(Self {
            config: config.cache,
            stores: HashMap::new(),
        })
    }

    pub async fn from_config(config: CacheConfig) -> Self {
        Self {
            config,
            stores: HashMap::new(),
        }
    }

    pub async fn store(&mut self, name: &str) -> Result<CacheDriver> {
        let store_name = if name == "default" {
            &self.config.default
        } else {
            name
        };

        // Check if store is already created
        if !self.stores.contains_key(store_name) {
            // Create new store instance
            let store_config = self.config.get_store(store_name).ok_or_else(|| {
                CacheError::Config {
                    message: format!("Cache store '{}' not found in configuration", store_name),
                }
            })?;

            let driver = match store_config.driver.as_str() {
                "memory" => {
                    let prefix = store_config.prefix.clone();
                    let cache = MemoryCache::new(prefix).start_cleanup_task().await;
                    CacheDriver::Memory(Arc::new(cache))
                }
                "redis" => {
                    let url = store_config.get_url();
                    let prefix = store_config.prefix.clone();

                    let mut cache = RedisCache::new(&url, prefix).await.map_err(|e| {
                        CacheError::Config {
                            message: format!("Failed to create Redis cache: {}", e),
                        }
                    })?;

                    CacheDriver::Redis(Arc::new(cache))
                }
                driver => {
                    return Err(CacheError::Config {
                        message: format!("Unknown cache driver: {}", driver),
                    }.into());
                }
            };

            self.stores.insert(store_name.to_string(), driver);
        }

        Ok(self.stores.get(store_name).unwrap().clone())
    }

    pub async fn default_store(&mut self) -> Result<CacheDriver> {
        self.store("default").await
    }

    pub fn get_config(&self) -> &CacheConfig {
        &self.config
    }

    pub fn store_names(&self) -> Vec<&String> {
        self.config.store_names()
    }

    pub async fn forget(&mut self, store_name: &str) {
        self.stores.remove(store_name);
    }

    pub async fn set_default(&mut self, name: String) -> Result<()> {
        if !self.config.stores.contains_key(&name) {
            return Err(CacheError::Config {
                message: format!("Cache store '{}' does not exist", name),
            }.into());
        }
        self.config.default = name;
        Ok(())
    }

    /// Flush all cache stores
    pub async fn flush_all(&mut self) -> Result<()> {
        let store_names: Vec<String> = self.store_names().iter().map(|s| s.to_string()).collect();
        for store_name in store_names {
            let cache = self.store(&store_name).await?;
            cache.flush().await?;
        }
        Ok(())
    }
}

// Helper functions that create a new manager each time
pub async fn cache(name: &str) -> Result<CacheDriver> {
    let mut manager = CacheManager::new().await?;
    manager.store(name).await
}

pub async fn default_cache() -> Result<CacheDriver> {
    cache("default").await
}

// Facade-like interface for easy access
pub struct CacheFacade;

impl CacheFacade {
    pub async fn get<T>(key: &str) -> Result<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de> + Send,
    {
        let cache = default_cache().await?;
        cache.get(key).await
    }

    pub async fn put<T>(key: &str, value: &T, ttl: Option<std::time::Duration>) -> Result<()>
    where
        T: serde::Serialize + Send + Sync,
    {
        let cache = default_cache().await?;
        cache.put(key, value, ttl).await
    }

    pub async fn forever<T>(key: &str, value: &T) -> Result<()>
    where
        T: serde::Serialize + Send + Sync,
    {
        let cache = default_cache().await?;
        cache.forever(key, value).await
    }

    pub async fn remember<T, F, Fut>(
        key: &str,
        ttl: Option<std::time::Duration>,
        callback: F,
    ) -> Result<T>
    where
        T: for<'de> serde::Deserialize<'de> + serde::Serialize + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        let cache = default_cache().await?;
        cache.remember(key, ttl, callback).await
    }

    pub async fn remember_forever<T, F, Fut>(key: &str, callback: F) -> Result<T>
    where
        T: for<'de> serde::Deserialize<'de> + serde::Serialize + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        let cache = default_cache().await?;
        cache.remember_forever(key, callback).await
    }

    pub async fn has(key: &str) -> Result<bool> {
        let cache = default_cache().await?;
        cache.has(key).await
    }

    pub async fn forget(key: &str) -> Result<bool> {
        let cache = default_cache().await?;
        cache.forget(key).await
    }

    pub async fn flush() -> Result<()> {
        let cache = default_cache().await?;
        cache.flush().await
    }

    pub async fn store(name: &str) -> Result<CacheDriver> {
        cache(name).await
    }
}