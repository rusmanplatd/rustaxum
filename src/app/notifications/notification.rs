use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Base trait that all notifications must implement
#[async_trait]
pub trait Notification: Send + Sync {
    /// Get the notification channels this notification should be sent on
    async fn via(&self, notifiable: &dyn Notifiable) -> Vec<NotificationChannel>;

    /// Get the mail representation of the notification
    async fn to_mail(&self, notifiable: &dyn Notifiable) -> Result<MailMessage>;

    /// Get the database representation of the notification
    async fn to_database(&self, notifiable: &dyn Notifiable) -> Result<DatabaseMessage>;

    /// Get the broadcast representation of the notification
    async fn to_broadcast(&self, notifiable: &dyn Notifiable) -> Result<BroadcastMessage> {
        Ok(BroadcastMessage {
            data: self.to_database(notifiable).await?.data,
        })
    }

    /// Get the web push representation of the notification
    async fn to_web_push(&self, notifiable: &dyn Notifiable) -> Result<crate::app::notifications::channels::web_push_channel::WebPushMessage> {
        let database_msg = self.to_database(notifiable).await?;
        let title = database_msg.data.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Notification")
            .to_string();
        let message = database_msg.data.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("You have a new notification")
            .to_string();

        Ok(crate::app::notifications::channels::web_push_channel::WebPushMessage::new(title, message)
            .data(database_msg.data))
    }

    /// Get a unique identifier for this notification type
    fn notification_type(&self) -> &'static str;
}

/// Trait for models that can receive notifications
#[async_trait]
pub trait Notifiable: Send + Sync {
    /// Get the notification routing information for the given channel
    async fn route_notification_for(&self, channel: &NotificationChannel) -> Option<String>;

    /// Get the entity's identifier
    fn get_key(&self) -> String;

    /// Get the entity's notification preferences
    async fn notification_preferences(&self) -> HashMap<String, bool> {
        HashMap::new()
    }
}

/// Available notification channels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationChannel {
    Mail,
    Database,
    Broadcast,
    WebPush,
    Sms,
    Slack,
    Custom(String),
}

/// Mail message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailMessage {
    pub from: Option<String>,
    pub to: String,
    pub subject: String,
    pub content: MailContent,
    pub attachments: Vec<String>,
}

/// Mail content types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MailContent {
    Text(String),
    Html(String),
    Markdown(String),
}

/// Database message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMessage {
    pub data: serde_json::Value,
}

/// Broadcast message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub data: serde_json::Value,
}

impl MailMessage {
    pub fn new(to: String, subject: String, content: MailContent) -> Self {
        Self {
            from: None,
            to,
            subject,
            content,
            attachments: Vec::new(),
        }
    }

    pub fn from(mut self, from: String) -> Self {
        self.from = Some(from);
        self
    }

    pub fn attach(mut self, attachment: String) -> Self {
        self.attachments.push(attachment);
        self
    }
}

impl DatabaseMessage {
    pub fn new(data: serde_json::Value) -> Self {
        Self { data }
    }
}