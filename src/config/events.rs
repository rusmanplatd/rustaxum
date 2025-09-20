use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct EventsConfig {
    pub default_queue: String,
    pub event_store_enabled: bool,
    pub event_store_table: String,
    pub retry_attempts: u32,
    pub retry_delay_seconds: u64,
}

impl EventsConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            default_queue: env::var("EVENTS_DEFAULT_QUEUE")
                .unwrap_or_else(|_| "events".to_string()),
            event_store_enabled: env::var("EVENTS_STORE_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            event_store_table: env::var("EVENTS_STORE_TABLE")
                .unwrap_or_else(|_| "events".to_string()),
            retry_attempts: env::var("EVENTS_RETRY_ATTEMPTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            retry_delay_seconds: env::var("EVENTS_RETRY_DELAY_SECONDS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
        })
    }
}