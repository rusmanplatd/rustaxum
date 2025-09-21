use crate::cache::{Cache, manager::{CacheFacade, CacheDriver}};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

pub struct CacheService {
    cache: Option<CacheDriver>,
}

impl CacheService {
    pub fn new() -> Self {
        Self { cache: None }
    }

    pub fn with_cache(cache: CacheDriver) -> Self {
        Self { cache: Some(cache) }
    }

    /// Cache a user for 1 hour
    pub async fn cache_user(&self, user: &User) -> Result<()> {
        let key = format!("user:{}", user.id);
        let ttl = Some(Duration::from_secs(3600)); // 1 hour

        match &self.cache {
            Some(cache) => cache.put(&key, user, ttl).await,
            None => CacheFacade::put(&key, user, ttl).await,
        }
    }

    /// Get a cached user
    pub async fn get_cached_user(&self, user_id: i32) -> Result<Option<User>> {
        let key = format!("user:{}", user_id);

        match &self.cache {
            Some(cache) => cache.get(&key).await,
            None => CacheFacade::get(&key).await,
        }
    }

    /// Get user with caching - if not in cache, load from "database" and cache it
    pub async fn get_user_with_cache(&self, user_id: i32) -> Result<User> {
        let key = format!("user:{}", user_id);
        let ttl = Some(Duration::from_secs(3600)); // 1 hour

        let user_loader = || async move {
            // Simulate database load
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(User {
                id: user_id,
                name: format!("User {}", user_id),
                email: format!("user{}@example.com", user_id),
            })
        };

        match &self.cache {
            Some(cache) => cache.remember(&key, ttl, user_loader).await,
            None => CacheFacade::remember(&key, ttl, user_loader).await,
        }
    }

    /// Cache user stats that never expire
    pub async fn cache_user_stats_forever(&self, user_id: i32, stats: &UserStats) -> Result<()> {
        let key = format!("user_stats:{}", user_id);

        match &self.cache {
            Some(cache) => cache.forever(&key, stats).await,
            None => CacheFacade::forever(&key, stats).await,
        }
    }

    /// Increment user login count
    pub async fn increment_login_count(&self, user_id: i32) -> Result<i64> {
        let key = format!("login_count:{}", user_id);

        match &self.cache {
            Some(cache) => cache.increment(&key, 1).await,
            None => {
                let cache = crate::cache::default_cache().await?;
                cache.increment(&key, 1).await
            }
        }
    }

    /// Clear user cache
    pub async fn clear_user_cache(&self, user_id: i32) -> Result<bool> {
        let key = format!("user:{}", user_id);

        match &self.cache {
            Some(cache) => cache.forget(&key).await,
            None => CacheFacade::forget(&key).await,
        }
    }

    /// Check if user is cached
    pub async fn is_user_cached(&self, user_id: i32) -> Result<bool> {
        let key = format!("user:{}", user_id);

        match &self.cache {
            Some(cache) => cache.has(&key).await,
            None => CacheFacade::has(&key).await,
        }
    }

    /// Cache multiple users at once
    pub async fn cache_multiple_users(&self, users: &[User]) -> Result<()> {
        let user_pairs: Vec<(String, &User)> = users
            .iter()
            .map(|user| (format!("user:{}", user.id), user))
            .collect();

        let key_value_pairs: Vec<(&str, &User)> = user_pairs
            .iter()
            .map(|(key, user)| (key.as_str(), *user))
            .collect();

        let ttl = Some(Duration::from_secs(3600)); // 1 hour

        match &self.cache {
            Some(cache) => cache.put_many(&key_value_pairs, ttl).await,
            None => {
                let cache = crate::cache::default_cache().await?;
                cache.put_many(&key_value_pairs, ttl).await
            }
        }
    }

    /// Get multiple users from cache
    pub async fn get_multiple_users(&self, user_ids: &[i32]) -> Result<Vec<(i32, Option<User>)>> {
        let keys: Vec<String> = user_ids
            .iter()
            .map(|id| format!("user:{}", id))
            .collect();

        let key_refs: Vec<&str> = keys.iter().map(|k| k.as_str()).collect();

        let results = match &self.cache {
            Some(cache) => cache.many(&key_refs).await?,
            None => {
                let cache = crate::cache::default_cache().await?;
                cache.many(&key_refs).await?
            }
        };

        // Convert back to (user_id, Option<User>)
        let mut user_results = Vec::new();
        for (i, (_, user_option)) in results.into_iter().enumerate() {
            user_results.push((user_ids[i], user_option));
        }

        Ok(user_results)
    }

    /// Example of using add() to set a value only if it doesn't exist
    pub async fn try_lock_user(&self, user_id: i32, lock_duration: Duration) -> Result<bool> {
        let key = format!("user_lock:{}", user_id);
        let lock_value = "locked";

        match &self.cache {
            Some(cache) => cache.add(&key, &lock_value, Some(lock_duration)).await,
            None => {
                let cache = crate::cache::default_cache().await?;
                cache.add(&key, &lock_value, Some(lock_duration)).await
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserStats {
    pub total_logins: i64,
    pub last_login: String,
    pub posts_count: i32,
}

impl Default for CacheService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::drivers::MemoryCache;
    use crate::cache::manager::CacheDriver;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_cache_service_with_memory_cache() {
        let memory_cache = Arc::new(MemoryCache::new(Some("test".to_string())));
        let cache_driver = CacheDriver::Memory(memory_cache);
        let service = CacheService::with_cache(cache_driver);

        let user = User {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        };

        // Test caching user
        service.cache_user(&user).await.unwrap();

        // Test retrieving cached user
        let cached_user = service.get_cached_user(1).await.unwrap();
        assert!(cached_user.is_some());
        assert_eq!(cached_user.unwrap().name, "John Doe");

        // Test checking if user is cached
        let is_cached = service.is_user_cached(1).await.unwrap();
        assert!(is_cached);

        // Test clearing cache
        let cleared = service.clear_user_cache(1).await.unwrap();
        assert!(cleared);

        // Verify user is no longer cached
        let is_cached_after_clear = service.is_user_cached(1).await.unwrap();
        assert!(!is_cached_after_clear);
    }

    #[tokio::test]
    async fn test_remember_functionality() {
        let memory_cache = Arc::new(MemoryCache::new(Some("test".to_string())));
        let cache_driver = CacheDriver::Memory(memory_cache);
        let service = CacheService::with_cache(cache_driver);

        // First call should load from "database" and cache
        let user = service.get_user_with_cache(123).await.unwrap();
        assert_eq!(user.id, 123);
        assert_eq!(user.name, "User 123");

        // Second call should return from cache (faster)
        let cached_user = service.get_user_with_cache(123).await.unwrap();
        assert_eq!(cached_user.id, 123);
        assert_eq!(cached_user.name, "User 123");
    }

    #[tokio::test]
    async fn test_increment_functionality() {
        let memory_cache = Arc::new(MemoryCache::new(Some("test".to_string())));
        let cache_driver = CacheDriver::Memory(memory_cache);
        let service = CacheService::with_cache(cache_driver);

        // First increment
        let count1 = service.increment_login_count(1).await.unwrap();
        assert_eq!(count1, 1);

        // Second increment
        let count2 = service.increment_login_count(1).await.unwrap();
        assert_eq!(count2, 2);

        // Third increment
        let count3 = service.increment_login_count(1).await.unwrap();
        assert_eq!(count3, 3);
    }

    #[tokio::test]
    async fn test_multiple_users_caching() {
        let memory_cache = Arc::new(MemoryCache::new(Some("test".to_string())));
        let cache_driver = CacheDriver::Memory(memory_cache);
        let service = CacheService::with_cache(cache_driver);

        let users = vec![
            User {
                id: 1,
                name: "User 1".to_string(),
                email: "user1@example.com".to_string(),
            },
            User {
                id: 2,
                name: "User 2".to_string(),
                email: "user2@example.com".to_string(),
            },
        ];

        // Cache multiple users
        service.cache_multiple_users(&users).await.unwrap();

        // Retrieve multiple users
        let user_ids = vec![1, 2, 3]; // 3 doesn't exist
        let results = service.get_multiple_users(&user_ids).await.unwrap();

        assert_eq!(results.len(), 3);
        assert!(results[0].1.is_some()); // User 1 exists
        assert!(results[1].1.is_some()); // User 2 exists
        assert!(results[2].1.is_none());  // User 3 doesn't exist
    }

    #[tokio::test]
    async fn test_try_lock_functionality() {
        let memory_cache = Arc::new(MemoryCache::new(Some("test".to_string())));
        let cache_driver = CacheDriver::Memory(memory_cache);
        let service = CacheService::with_cache(cache_driver);

        let lock_duration = Duration::from_secs(1);

        // First lock should succeed
        let locked1 = service.try_lock_user(1, lock_duration).await.unwrap();
        assert!(locked1);

        // Second lock should fail (key already exists)
        let locked2 = service.try_lock_user(1, lock_duration).await.unwrap();
        assert!(!locked2);

        // Wait for lock to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Lock should succeed again after expiration
        let locked3 = service.try_lock_user(1, lock_duration).await.unwrap();
        assert!(locked3);
    }
}