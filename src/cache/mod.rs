use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod drivers;
pub mod manager;

pub use manager::{CacheManager, cache, default_cache};

#[async_trait]
pub trait Cache: Send + Sync {
    /// Get a value from the cache
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send;

    /// Put a value into the cache with optional TTL
    async fn put<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync;

    /// Put a value into the cache forever (no expiration)
    async fn forever<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize + Send + Sync;

    /// Get a value or put a default value if it doesn't exist
    async fn remember<T, F, Fut>(&self, key: &str, ttl: Option<Duration>, callback: F) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + Serialize + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send;

    /// Get a value or put a default value forever if it doesn't exist
    async fn remember_forever<T, F, Fut>(&self, key: &str, callback: F) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + Serialize + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send;

    /// Check if a key exists in the cache
    async fn has(&self, key: &str) -> Result<bool>;

    /// Remove a value from the cache
    async fn forget(&self, key: &str) -> Result<bool>;

    /// Clear all values from the cache
    async fn flush(&self) -> Result<()>;

    /// Get multiple values from the cache
    async fn many<T>(&self, keys: &[&str]) -> Result<Vec<(String, Option<T>)>>
    where
        T: for<'de> Deserialize<'de> + Send;

    /// Put multiple values into the cache
    async fn put_many<T>(&self, values: &[(&str, &T)], ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync;

    /// Increment a numeric value in the cache
    async fn increment(&self, key: &str, value: i64) -> Result<i64>;

    /// Decrement a numeric value in the cache
    async fn decrement(&self, key: &str, value: i64) -> Result<i64>;

    /// Add a value to the cache only if it doesn't exist
    async fn add<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<bool>
    where
        T: Serialize + Send + Sync;

    /// Get the name of the cache driver
    fn name(&self) -> &str;

    /// Get the prefix for cache keys
    fn prefix(&self) -> Option<&str>;
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Serialization error: {message}")]
    Serialization { message: String },

    #[error("Deserialization error: {message}")]
    Deserialization { message: String },

    #[error("Connection error: {message}")]
    Connection { message: String },

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Cache operation failed: {message}")]
    Operation { message: String },
}

impl From<serde_json::Error> for CacheError {
    fn from(err: serde_json::Error) -> Self {
        CacheError::Serialization {
            message: err.to_string(),
        }
    }
}