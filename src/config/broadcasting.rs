use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct BroadcastingConfig {
    pub default_driver: String,
    pub websocket_enabled: bool,
    pub websocket_port: u16,
    pub redis_enabled: bool,
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_password: Option<String>,
    pub redis_database: u8,
    pub channels_prefix: String,
}

impl BroadcastingConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            default_driver: env::var("BROADCAST_DRIVER")
                .unwrap_or_else(|_| "log".to_string()),
            websocket_enabled: env::var("BROADCAST_WEBSOCKET_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            websocket_port: env::var("BROADCAST_WEBSOCKET_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            redis_enabled: env::var("BROADCAST_REDIS_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            redis_host: env::var("BROADCAST_REDIS_HOST")
                .unwrap_or_else(|_| "localhost".to_string()),
            redis_port: env::var("BROADCAST_REDIS_PORT")
                .unwrap_or_else(|_| "6379".to_string())
                .parse()
                .unwrap_or(6379),
            redis_password: env::var("BROADCAST_REDIS_PASSWORD").ok(),
            redis_database: env::var("BROADCAST_REDIS_DATABASE")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .unwrap_or(0),
            channels_prefix: env::var("BROADCAST_CHANNELS_PREFIX")
                .unwrap_or_else(|_| "".to_string()),
        })
    }

    pub fn redis_url(&self) -> String {
        if let Some(ref password) = self.redis_password {
            format!("redis://:{}@{}:{}/{}", password, self.redis_host, self.redis_port, self.redis_database)
        } else {
            format!("redis://{}:{}/{}", self.redis_host, self.redis_port, self.redis_database)
        }
    }
}