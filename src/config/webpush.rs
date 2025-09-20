use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct WebPushConfig {
    pub vapid_private_key: Option<String>,
    pub vapid_public_key: Option<String>,
    pub vapid_subject: String,
}

impl WebPushConfig {
    pub fn from_env() -> Result<Self> {
        Ok(WebPushConfig {
            vapid_private_key: env::var("VAPID_PRIVATE_KEY").ok(),
            vapid_public_key: env::var("VAPID_PUBLIC_KEY").ok(),
            vapid_subject: env::var("VAPID_SUBJECT")
                .unwrap_or_else(|_| "mailto:admin@rustaxum.com".to_string()),
        })
    }

    pub fn is_configured(&self) -> bool {
        self.vapid_private_key.is_some() && self.vapid_public_key.is_some()
    }

    pub fn get_public_key(&self) -> Option<&str> {
        self.vapid_public_key.as_deref()
    }
}