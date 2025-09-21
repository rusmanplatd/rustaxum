use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use anyhow::Result;

/// Simple in-memory cache for query results
#[derive(Debug, Clone)]
pub struct QueryCache {
    storage: Arc<RwLock<HashMap<String, CacheEntry>>>,
    default_ttl: Duration,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: String, // JSON serialized data
    expires_at: Instant,
}

impl QueryCache {
    /// Create a new query cache with default TTL
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    /// Create a cache with 5-minute default TTL
    pub fn with_default_ttl() -> Self {
        Self::new(Duration::from_secs(300)) // 5 minutes
    }

    /// Get cached value if it exists and is not expired
    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let storage = self.storage.read().ok()?;

        if let Some(entry) = storage.get(key) {
            if Instant::now() < entry.expires_at {
                // Cache hit and not expired
                return serde_json::from_str(&entry.data).ok();
            }
        }

        None
    }

    /// Store a value in the cache with default TTL
    pub fn set<T>(&self, key: String, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.set_with_ttl(key, value, self.default_ttl)
    }

    /// Store a value in the cache with custom TTL
    pub fn set_with_ttl<T>(&self, key: String, value: &T, ttl: Duration) -> Result<()>
    where
        T: Serialize,
    {
        let data = serde_json::to_string(value)?;
        let entry = CacheEntry {
            data,
            expires_at: Instant::now() + ttl,
        };

        if let Ok(mut storage) = self.storage.write() {
            storage.insert(key, entry);
        }

        Ok(())
    }

    /// Remove a specific key from cache
    pub fn remove(&self, key: &str) {
        if let Ok(mut storage) = self.storage.write() {
            storage.remove(key);
        }
    }

    /// Clear all cached entries
    pub fn clear(&self) {
        if let Ok(mut storage) = self.storage.write() {
            storage.clear();
        }
    }

    /// Remove expired entries
    pub fn cleanup_expired(&self) {
        if let Ok(mut storage) = self.storage.write() {
            let now = Instant::now();
            storage.retain(|_, entry| now < entry.expires_at);
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        match self.storage.read() { Ok(storage) => {
            let now = Instant::now();
            let total = storage.len();
            let expired = storage.values().filter(|entry| now >= entry.expires_at).count();

            CacheStats {
                total_entries: total,
                expired_entries: expired,
                active_entries: total - expired,
            }
        } _ => {
            CacheStats::default()
        }}
    }

    /// Generate cache key from query parameters
    pub fn generate_key(&self, table: &str, query_hash: u64) -> String {
        format!("query:{}:{}", table, query_hash)
    }
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
}

/// Cache key builder for generating consistent cache keys
pub struct CacheKeyBuilder {
    components: Vec<String>,
}

impl CacheKeyBuilder {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn table(mut self, table: &str) -> Self {
        self.components.push(format!("table:{}", table));
        self
    }

    pub fn filters(mut self, filters: &std::collections::HashMap<String, String>) -> Self {
        let mut filter_parts: Vec<String> = filters
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect();
        filter_parts.sort(); // Ensure consistent ordering

        if !filter_parts.is_empty() {
            self.components.push(format!("filters:{}", filter_parts.join(",")));
        }
        self
    }

    pub fn sorts(mut self, sorts: &[String]) -> Self {
        if !sorts.is_empty() {
            self.components.push(format!("sorts:{}", sorts.join(",")));
        }
        self
    }

    pub fn fields(mut self, fields: &Option<Vec<String>>) -> Self {
        if let Some(fields) = fields {
            self.components.push(format!("fields:{}", fields.join(",")));
        }
        self
    }

    pub fn includes(mut self, includes: &Option<Vec<String>>) -> Self {
        if let Some(includes) = includes {
            self.components.push(format!("includes:{}", includes.join(",")));
        }
        self
    }

    pub fn page(mut self, page: Option<u64>, per_page: Option<u64>) -> Self {
        if let (Some(p), Some(pp)) = (page, per_page) {
            self.components.push(format!("page:{}:{}", p, pp));
        }
        self
    }

    pub fn build(self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let key_string = self.components.join("|");

        // Hash the key to keep it reasonably sized
        let mut hasher = DefaultHasher::new();
        key_string.hash(&mut hasher);
        let hash = hasher.finish();

        format!("qb:{}", hash)
    }
}

impl Default for CacheKeyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache-aware trait for cached operations
pub trait Cacheable {
    /// Check if caching is enabled for this operation
    fn cache_enabled(&self) -> bool {
        true
    }

    /// Get cache TTL for this operation
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(300) // 5 minutes default
    }

    /// Should this query be cached (can exclude certain conditions)
    fn should_cache(&self) -> bool {
        self.cache_enabled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_basic_operations() {
        let cache = QueryCache::with_default_ttl();

        // Test set and get
        let data = vec!["test1", "test2"];
        cache.set("test_key".to_string(), &data).unwrap();

        let retrieved: Option<Vec<String>> = cache.get("test_key");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), data);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = QueryCache::new(Duration::from_millis(100));

        let data = "test_data";
        cache.set("expire_test".to_string(), &data).unwrap();

        // Should be available immediately
        let retrieved: Option<String> = cache.get("expire_test");
        assert!(retrieved.is_some());

        // Wait for expiration
        thread::sleep(Duration::from_millis(150));

        // Should be expired now
        let expired: Option<String> = cache.get("expire_test");
        assert!(expired.is_none());
    }

    #[test]
    fn test_cache_key_builder() {
        let mut filters = std::collections::HashMap::new();
        filters.insert("status".to_string(), "active".to_string());

        let key = CacheKeyBuilder::new()
            .table("users")
            .filters(&filters)
            .sorts(&vec!["name".to_string()])
            .page(Some(1), Some(10))
            .build();

        assert!(key.starts_with("qb:"));

        // Same parameters should generate same key
        let key2 = CacheKeyBuilder::new()
            .table("users")
            .filters(&filters)
            .sorts(&vec!["name".to_string()])
            .page(Some(1), Some(10))
            .build();

        assert_eq!(key, key2);
    }

    #[test]
    fn test_cache_stats() {
        let cache = QueryCache::with_default_ttl();

        cache.set("key1".to_string(), &"value1").unwrap();
        cache.set("key2".to_string(), &"value2").unwrap();

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.active_entries, 2);
        assert_eq!(stats.expired_entries, 0);
    }

    #[test]
    fn test_cache_cleanup() {
        let cache = QueryCache::new(Duration::from_millis(50));

        cache.set("key1".to_string(), &"value1").unwrap();
        cache.set("key2".to_string(), &"value2").unwrap();

        // Wait for expiration
        thread::sleep(Duration::from_millis(100));

        let stats_before = cache.stats();
        assert_eq!(stats_before.expired_entries, 2);

        cache.cleanup_expired();

        let stats_after = cache.stats();
        assert_eq!(stats_after.total_entries, 0);
    }
}