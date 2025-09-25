use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub driver: String,
    pub lifetime: u64,
    pub expire_on_close: bool,
    pub encrypt: bool,
    pub files: String,
    pub connection: Option<String>,
    pub table: String,
    pub store: Option<String>,
    pub lottery: [u8; 2],
    pub cookie: String,
    pub path: String,
    pub domain: Option<String>,
    pub secure: Option<bool>,
    pub http_only: bool,
    pub same_site: Option<String>,
    pub partitioned: bool,
}

impl SessionConfig {
    pub fn from_env() -> Result<Self> {
        Ok(SessionConfig {
            driver: env::var("SESSION_DRIVER").unwrap_or_else(|_| "database".to_string()),
            lifetime: env::var("SESSION_LIFETIME")
                .unwrap_or_else(|_| "120".to_string())
                .parse()
                .unwrap_or(120),
            expire_on_close: env::var("SESSION_EXPIRE_ON_CLOSE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            encrypt: env::var("SESSION_ENCRYPT")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            files: env::var("SESSION_FILES").unwrap_or_else(|_| "storage/framework/sessions".to_string()),
            connection: env::var("SESSION_CONNECTION").ok(),
            table: env::var("SESSION_TABLE").unwrap_or_else(|_| "sessions".to_string()),
            store: env::var("SESSION_STORE").ok(),
            lottery: [
                env::var("SESSION_LOTTERY_0")
                    .unwrap_or_else(|_| "2".to_string())
                    .parse()
                    .unwrap_or(2),
                env::var("SESSION_LOTTERY_1")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
            ],
            cookie: env::var("SESSION_COOKIE").unwrap_or_else(|_| "rustaxum_session".to_string()),
            path: env::var("SESSION_PATH").unwrap_or_else(|_| "/".to_string()),
            domain: env::var("SESSION_DOMAIN").ok(),
            secure: env::var("SESSION_SECURE").ok().map(|v| v.parse().unwrap_or(false)),
            http_only: env::var("SESSION_HTTP_ONLY")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            same_site: env::var("SESSION_SAME_SITE").ok(),
            partitioned: env::var("SESSION_PARTITIONED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        })
    }

    pub fn is_file_driver(&self) -> bool {
        self.driver == "file"
    }

    pub fn is_database_driver(&self) -> bool {
        self.driver == "database"
    }

    pub fn is_redis_driver(&self) -> bool {
        self.driver == "redis"
    }

    pub fn is_array_driver(&self) -> bool {
        self.driver == "array"
    }

    pub fn lifetime_in_seconds(&self) -> u64 {
        self.lifetime * 60
    }
}