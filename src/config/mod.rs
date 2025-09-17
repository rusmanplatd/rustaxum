use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub app_name: String,
    pub app_env: String,
    pub app_debug: bool,
    pub app_url: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in: u64,
    pub log_level: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            app_name: env::var("APP_NAME").unwrap_or_else(|_| "RustAxum".to_string()),
            app_env: env::var("APP_ENV").unwrap_or_else(|_| "local".to_string()),
            app_debug: env::var("APP_DEBUG")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            app_url: env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://username:password@localhost:5432/rustaxum".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key-here".to_string()),
            jwt_expires_in: env::var("JWT_EXPIRES_IN")
                .unwrap_or_else(|_| "86400".to_string())
                .parse()
                .unwrap_or(86400),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
        })
    }

    pub fn server_addr(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }

    pub fn is_production(&self) -> bool {
        self.app_env == "production"
    }

    pub fn is_development(&self) -> bool {
        self.app_env == "local" || self.app_env == "development"
    }
}