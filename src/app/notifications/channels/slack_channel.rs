use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::app::notifications::channels::Channel;
use crate::app::notifications::notification::{Notification, Notifiable, NotificationChannel, SlackMessage, SlackAttachment, SlackField};
use crate::config::Config;

#[derive(Debug, Clone)]
pub struct SlackChannel {
    webhook_url: Option<String>,
    default_channel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: Option<SlackText>,
    pub elements: Option<Vec<SlackElement>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackText {
    #[serde(rename = "type")]
    pub text_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackElement {
    #[serde(rename = "type")]
    pub element_type: String,
    pub text: Option<String>,
}

impl SlackChannel {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;

        Ok(Self {
            webhook_url: config.notifications.slack_webhook_url.clone(),
            default_channel: std::env::var("SLACK_DEFAULT_CHANNEL")
                .unwrap_or_else(|_| "#general".to_string()),
        })
    }

    pub fn with_webhook_url(webhook_url: String) -> Self {
        Self {
            webhook_url: Some(webhook_url),
            default_channel: "#general".to_string(),
        }
    }

    pub fn with_default_channel(mut self, channel: String) -> Self {
        self.default_channel = channel;
        self
    }

    async fn send_slack_message(&self, message: &SlackMessage) -> Result<()> {
        let webhook_url = match &self.webhook_url {
            Some(url) => url,
            None => {
                tracing::warn!("No Slack webhook URL configured, logging message instead");
                self.log_slack_message(&message).await;
                return Ok(());
            }
        };

        let client = reqwest::Client::new();

        let response = client
            .post(webhook_url)
            .json(&message)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("Slack message sent successfully");
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Slack webhook failed: {}", error_text))
        }
    }

    async fn log_slack_message(&self, message: &SlackMessage) {
        println!("===============================");
        println!("SLACK LOG ENTRY");
        println!("===============================");
        println!("Channel: {}", message.channel.as_ref().unwrap_or(&self.default_channel));
        println!("Text: {}", message.text);
        if let Some(username) = &message.username {
            println!("Username: {}", username);
        }
        if let Some(emoji) = &message.icon_emoji {
            println!("Icon: {}", emoji);
        }
        if !message.attachments.is_empty() {
            println!("Attachments: {} items", message.attachments.len());
        }
        println!("Timestamp: {}", chrono::Utc::now());
        println!("===============================");
    }

    fn create_notification_message(&self, notification_type: &str, data: &serde_json::Value) -> SlackMessage {
        let title = data.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Notification");

        let message = data.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("You have a new notification");

        let color = match notification_type {
            "UserRegistered" => "#36a64f", // Green
            "OrderShipped" => "#2196F3",    // Blue
            "PaymentFailed" => "#f44336",   // Red
            "SystemAlert" => "#ff9800",     // Orange
            _ => "#9e9e9e",                 // Gray
        };

        let attachment = SlackAttachment::new()
            .color(color.to_string())
            .title(title.to_string())
            .text(message.to_string())
            .field(SlackField::new("Type".to_string(), notification_type.to_string(), true))
            .field(SlackField::new("Time".to_string(), chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(), true));

        SlackMessage::new(format!("*{}*", title))
            .channel(self.default_channel.clone())
            .username("Notification Bot".to_string())
            .icon_emoji(":bell:".to_string())
            .attachment(attachment)
    }

    fn create_rich_message(&self, notification_type: &str, data: &serde_json::Value) -> SlackMessage {
        let title = data.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Notification");

        let message = data.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("You have a new notification");

        // Create blocks for rich formatting
        let blocks = vec![
            serde_json::json!({
                "type": "header",
                "text": {
                    "type": "plain_text",
                    "text": title
                }
            }),
            serde_json::json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": message
                }
            }),
            serde_json::json!({
                "type": "context",
                "elements": [{
                    "type": "mrkdwn",
                    "text": format!("*Type:* {} | *Time:* {}",
                        notification_type,
                        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
                    )
                }]
            })
        ];

        let mut message = SlackMessage::new(title.to_string())
            .channel(self.default_channel.clone())
            .username("Notification Bot".to_string())
            .icon_emoji(":bell:".to_string());

        for block in blocks {
            message = message.block(block);
        }

        message
    }
}

impl Default for SlackChannel {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            webhook_url: None,
            default_channel: "#general".to_string(),
        })
    }
}

#[async_trait]
impl Channel for SlackChannel {
    async fn send(&self, notification: &dyn Notification, notifiable: &dyn Notifiable) -> Result<()> {
        // Get Slack channel for the notifiable entity (optional)
        let target_channel = notifiable.route_notification_for(&NotificationChannel::Slack).await
            .unwrap_or_else(|| self.default_channel.clone());

        // Try to get the slack message from the notification first
        let mut slack_message = match notification.to_slack(notifiable) {
            Ok(message) => message,
            Err(_) => {
                // Fallback to creating from database message
                let database_message = notification.to_database(notifiable)?;
                self.create_rich_message(notification.notification_type(), &database_message.data)
            }
        };

        // Override channel if not set
        if slack_message.channel.is_none() {
            slack_message.channel = Some(target_channel.clone());
        }

        // Send the message
        match self.send_slack_message(&slack_message).await {
            Ok(()) => {
                tracing::info!(
                    "Slack notification sent successfully to channel: {} (type: {})",
                    target_channel,
                    notification.notification_type()
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    "Failed to send Slack notification to channel {}: {}",
                    target_channel,
                    e
                );
                Err(e)
            }
        }
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Slack
    }
}