use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub name: String,
    pub env: String,
    pub debug: bool,
    pub url: String,
    pub port: u16,
    pub key: String,
    pub templates_path: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(AppConfig {
            name: env::var("APP_NAME").unwrap_or_else(|_| "RustAxum".to_string()),
            env: env::var("APP_ENV").unwrap_or_else(|_| "local".to_string()),
            debug: env::var("APP_DEBUG")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            url: env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            key: env::var("APP_KEY").unwrap_or_else(|_| "".to_string()),
            templates_path: env::var("TEMPLATES_PATH").unwrap_or_else(|_| "resources/views".to_string()),
        })
    }

    pub fn is_production(&self) -> bool {
        self.env == "production"
    }

    pub fn is_development(&self) -> bool {
        self.env == "local" || self.env == "development"
    }

    pub fn is_testing(&self) -> bool {
        self.env == "testing"
    }

    pub fn is_staging(&self) -> bool {
        self.env == "staging"
    }
}