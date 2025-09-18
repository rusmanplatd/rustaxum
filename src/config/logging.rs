use anyhow::Result;
use std::env;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub default: String,
    pub channels: HashMap<String, ChannelConfig>,
}

#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub driver: String,
    pub level: String,
    pub path: Option<String>,
    pub max_files: Option<u32>,
    pub max_file_size: Option<String>,
    pub format: Option<String>,
    pub date_format: Option<String>,
}

impl LoggingConfig {
    pub fn from_env() -> Result<Self> {
        let default_channel = env::var("LOG_CHANNEL").unwrap_or_else(|_| "single".to_string());
        let mut channels = HashMap::new();

        // Single file channel (default)
        channels.insert("single".to_string(), ChannelConfig {
            driver: "single".to_string(),
            level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            path: Some(env::var("LOG_PATH").unwrap_or_else(|_| "storage/logs/app.log".to_string())),
            max_files: None,
            max_file_size: Some(env::var("LOG_MAX_FILE_SIZE").unwrap_or_else(|_| "10MB".to_string())),
            format: Some(env::var("LOG_FORMAT").unwrap_or_else(|_| "default".to_string())),
            date_format: Some(env::var("LOG_DATE_FORMAT").unwrap_or_else(|_| "%Y-%m-%d %H:%M:%S".to_string())),
        });

        // Daily rotating logs
        channels.insert("daily".to_string(), ChannelConfig {
            driver: "daily".to_string(),
            level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            path: Some(env::var("LOG_DAILY_PATH").unwrap_or_else(|_| "storage/logs/app".to_string())),
            max_files: Some(env::var("LOG_MAX_FILES")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .unwrap_or(7)),
            max_file_size: Some(env::var("LOG_MAX_FILE_SIZE").unwrap_or_else(|_| "10MB".to_string())),
            format: Some(env::var("LOG_FORMAT").unwrap_or_else(|_| "default".to_string())),
            date_format: Some(env::var("LOG_DATE_FORMAT").unwrap_or_else(|_| "%Y-%m-%d %H:%M:%S".to_string())),
        });

        // Console output
        channels.insert("stderr".to_string(), ChannelConfig {
            driver: "stderr".to_string(),
            level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            path: None,
            max_files: None,
            max_file_size: None,
            format: Some(env::var("LOG_FORMAT").unwrap_or_else(|_| "default".to_string())),
            date_format: Some(env::var("LOG_DATE_FORMAT").unwrap_or_else(|_| "%Y-%m-%d %H:%M:%S".to_string())),
        });

        // Stack channel (multiple outputs)
        channels.insert("stack".to_string(), ChannelConfig {
            driver: "stack".to_string(),
            level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            path: None,
            max_files: None,
            max_file_size: None,
            format: Some(env::var("LOG_FORMAT").unwrap_or_else(|_| "default".to_string())),
            date_format: Some(env::var("LOG_DATE_FORMAT").unwrap_or_else(|_| "%Y-%m-%d %H:%M:%S".to_string())),
        });

        Ok(LoggingConfig {
            default: default_channel,
            channels,
        })
    }

    pub fn get_channel(&self, channel: &str) -> Option<&ChannelConfig> {
        self.channels.get(channel)
    }

    pub fn get_default_channel(&self) -> Option<&ChannelConfig> {
        self.channels.get(&self.default)
    }

    pub fn is_debug(&self) -> bool {
        if let Some(channel) = self.get_default_channel() {
            channel.level.to_lowercase() == "debug"
        } else {
            false
        }
    }

    pub fn is_trace(&self) -> bool {
        if let Some(channel) = self.get_default_channel() {
            channel.level.to_lowercase() == "trace"
        } else {
            false
        }
    }

    pub fn should_log_sql(&self) -> bool {
        self.is_debug() || self.is_trace()
    }
}