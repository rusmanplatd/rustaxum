use anyhow::Result;
use dotenv::dotenv;
use std::env;
use std::path::Path;

pub mod app;
pub mod database;
pub mod mail;
pub mod auth;
pub mod logging;
pub mod storage;

#[derive(Debug, Clone)]
pub struct Config {
    pub app: app::AppConfig,
    pub database: database::DatabaseConfig,
    pub mail: mail::MailConfig,
    pub auth: auth::AuthConfig,
    pub logging: logging::LoggingConfig,
    pub storage: storage::StorageConfig,
}

impl Config {
    pub fn load() -> Result<Self> {
        Self::load_dotenv();

        Ok(Config {
            app: app::AppConfig::from_env()?,
            database: database::DatabaseConfig::from_env()?,
            mail: mail::MailConfig::from_env()?,
            auth: auth::AuthConfig::from_env()?,
            logging: logging::LoggingConfig::from_env()?,
            storage: storage::StorageConfig::from_env()?,
        })
    }

    pub fn from_env() -> Result<Self> {
        Self::load()
    }

    fn load_dotenv() {
        let env_file = match env::var("APP_ENV").unwrap_or_else(|_| "local".to_string()).as_str() {
            "production" => ".env.production",
            "staging" => ".env.staging",
            "testing" => ".env.testing",
            _ => ".env",
        };

        if Path::new(env_file).exists() {
            dotenv::from_filename(env_file).ok();
        } else {
            dotenv().ok();
        }
    }

    pub fn server_addr(&self) -> String {
        format!("0.0.0.0:{}", self.app.port)
    }

    pub fn is_production(&self) -> bool {
        self.app.is_production()
    }

    pub fn is_development(&self) -> bool {
        self.app.is_development()
    }

    pub fn is_testing(&self) -> bool {
        self.app.is_testing()
    }
}