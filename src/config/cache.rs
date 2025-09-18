use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub default: String,
    pub stores: HashMap<String, CacheStoreConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStoreConfig {
    pub driver: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub password: Option<String>,
    pub database: Option<String>,
    pub prefix: Option<String>,
    pub url: Option<String>,
}

impl CacheStoreConfig {
    pub fn get_url(&self) -> String {
        if let Some(ref url) = self.url {
            return url.clone();
        }

        match self.driver.as_str() {
            "redis" => {
                let host = self.host.as_deref().unwrap_or("127.0.0.1");
                let port = self.port.unwrap_or(6379);
                let password_part = if let Some(ref password) = self.password {
                    format!(":{}@", password)
                } else {
                    String::new()
                };
                let database = self.database.as_deref().unwrap_or("0");
                format!("redis://{password_part}{host}:{port}/{database}")
            }
            _ => String::new(),
        }
    }
}

impl CacheConfig {
    pub fn from_env() -> Result<Self> {
        let default = env::var("CACHE_DRIVER").unwrap_or_else(|_| "memory".to_string());

        let mut stores = HashMap::new();

        // Memory store
        stores.insert(
            "memory".to_string(),
            CacheStoreConfig {
                driver: "memory".to_string(),
                host: None,
                port: None,
                password: None,
                database: None,
                prefix: env::var("CACHE_PREFIX").ok(),
                url: None,
            },
        );

        // Redis store
        stores.insert(
            "redis".to_string(),
            CacheStoreConfig {
                driver: "redis".to_string(),
                host: env::var("REDIS_HOST").ok(),
                port: env::var("REDIS_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok()),
                password: env::var("REDIS_PASSWORD").ok(),
                database: env::var("REDIS_DATABASE").ok(),
                prefix: env::var("CACHE_PREFIX").ok(),
                url: env::var("REDIS_URL").ok(),
            },
        );

        Ok(CacheConfig { default, stores })
    }

    pub fn get_store(&self, name: &str) -> Option<&CacheStoreConfig> {
        self.stores.get(name)
    }

    pub fn store_names(&self) -> Vec<&String> {
        self.stores.keys().collect()
    }

    pub fn default_store(&self) -> Option<&CacheStoreConfig> {
        self.get_store(&self.default)
    }
}