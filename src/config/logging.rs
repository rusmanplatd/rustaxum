use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub channel: String,
    pub path: String,
    pub max_files: u32,
    pub max_file_size: String,
}

impl LoggingConfig {
    pub fn from_env() -> Result<Self> {
        Ok(LoggingConfig {
            level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            channel: env::var("LOG_CHANNEL").unwrap_or_else(|_| "single".to_string()),
            path: env::var("LOG_PATH").unwrap_or_else(|_| "storage/logs/app.log".to_string()),
            max_files: env::var("LOG_MAX_FILES")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            max_file_size: env::var("LOG_MAX_FILE_SIZE")
                .unwrap_or_else(|_| "10MB".to_string()),
        })
    }

    pub fn is_debug(&self) -> bool {
        self.level.to_lowercase() == "debug"
    }

    pub fn is_trace(&self) -> bool {
        self.level.to_lowercase() == "trace"
    }

    pub fn should_log_sql(&self) -> bool {
        self.is_debug() || self.is_trace()
    }
}