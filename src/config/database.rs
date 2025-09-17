use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub pool_max_connections: u32,
    pub pool_min_connections: u32,
    pub pool_acquire_timeout_seconds: u64,
    pub pool_idle_timeout_seconds: u64,
    pub pool_max_lifetime_seconds: u64,
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self> {
        let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port: u16 = env::var("DB_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse()
            .unwrap_or(5432);
        let database = env::var("DB_DATABASE").unwrap_or_else(|_| "rustaxum".to_string());
        let username = env::var("DB_USERNAME").unwrap_or_else(|_| "username".to_string());
        let password = env::var("DB_PASSWORD").unwrap_or_else(|_| "password".to_string());

        let default_url = format!("postgresql://{}:{}@{}:{}/{}", username, password, host, port, database);

        Ok(DatabaseConfig {
            url: env::var("DATABASE_URL").unwrap_or(default_url),
            host,
            port,
            database,
            username,
            password,
            pool_max_connections: env::var("DB_POOL_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            pool_min_connections: env::var("DB_POOL_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            pool_acquire_timeout_seconds: env::var("DB_POOL_ACQUIRE_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            pool_idle_timeout_seconds: env::var("DB_POOL_IDLE_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "600".to_string())
                .parse()
                .unwrap_or(600),
            pool_max_lifetime_seconds: env::var("DB_POOL_MAX_LIFETIME_SECONDS")
                .unwrap_or_else(|_| "1800".to_string())
                .parse()
                .unwrap_or(1800),
        })
    }
}