use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct NotificationsConfig {
    pub default_channels: Vec<String>,
    pub queue_notifications: bool,
    pub database_notifications_table: String,
    pub slack_webhook_url: Option<String>,
    pub sms_provider: String,
    pub sms_api_key: Option<String>,
    pub notification_preferences_enabled: bool,
}

impl NotificationsConfig {
    pub fn from_env() -> Result<Self> {
        let default_channels_str = env::var("NOTIFICATIONS_DEFAULT_CHANNELS")
            .unwrap_or_else(|_| "mail,database".to_string());
        let default_channels = default_channels_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Self {
            default_channels,
            queue_notifications: env::var("NOTIFICATIONS_QUEUE_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            database_notifications_table: env::var("NOTIFICATIONS_DATABASE_TABLE")
                .unwrap_or_else(|_| "notifications".to_string()),
            slack_webhook_url: env::var("NOTIFICATIONS_SLACK_WEBHOOK_URL").ok(),
            sms_provider: env::var("NOTIFICATIONS_SMS_PROVIDER")
                .unwrap_or_else(|_| "twilio".to_string()),
            sms_api_key: env::var("NOTIFICATIONS_SMS_API_KEY").ok(),
            notification_preferences_enabled: env::var("NOTIFICATIONS_PREFERENCES_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }

    pub fn is_channel_enabled(&self, channel: &str) -> bool {
        self.default_channels.contains(&channel.to_string())
    }

    pub fn has_slack_integration(&self) -> bool {
        self.slack_webhook_url.is_some()
    }

    pub fn has_sms_integration(&self) -> bool {
        self.sms_api_key.is_some()
    }
}