use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::app::notifications::channels::Channel;
use crate::app::notifications::notification::{Notification, Notifiable, NotificationChannel};
use crate::config::Config;

#[derive(Debug, Clone)]
pub struct SlackChannel {
    webhook_url: Option<String>,
    default_channel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackMessage {
    pub text: String,
    pub channel: Option<String>,
    pub username: Option<String>,
    pub icon_emoji: Option<String>,
    pub attachments: Option<Vec<SlackAttachment>>,
    pub blocks: Option<Vec<SlackBlock>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackAttachment {
    pub color: Option<String>,
    pub title: Option<String>,
    pub text: Option<String>,
    pub fields: Option<Vec<SlackField>>,
    pub footer: Option<String>,
    pub ts: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackField {
    pub title: String,
    pub value: String,
    pub short: Option<bool>,
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

    async fn send_slack_message(&self, message: SlackMessage) -> Result<()> {
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
        if let Some(attachments) = &message.attachments {
            println!("Attachments: {} items", attachments.len());
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

        let attachment = SlackAttachment {
            color: Some(color.to_string()),
            title: Some(title.to_string()),
            text: Some(message.to_string()),
            fields: Some(vec![
                SlackField {
                    title: "Type".to_string(),
                    value: notification_type.to_string(),
                    short: Some(true),
                },
                SlackField {
                    title: "Time".to_string(),
                    value: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                    short: Some(true),
                },
            ]),
            footer: Some("Notification System".to_string()),
            ts: Some(chrono::Utc::now().timestamp()),
        };

        SlackMessage {
            text: format!("*{}*", title),
            channel: Some(self.default_channel.clone()),
            username: Some("Notification Bot".to_string()),
            icon_emoji: Some(":bell:".to_string()),
            attachments: Some(vec![attachment]),
            blocks: None,
        }
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
            SlackBlock {
                block_type: "header".to_string(),
                text: Some(SlackText {
                    text_type: "plain_text".to_string(),
                    text: title.to_string(),
                }),
                elements: None,
            },
            SlackBlock {
                block_type: "section".to_string(),
                text: Some(SlackText {
                    text_type: "mrkdwn".to_string(),
                    text: message.to_string(),
                }),
                elements: None,
            },
            SlackBlock {
                block_type: "context".to_string(),
                text: None,
                elements: Some(vec![
                    SlackElement {
                        element_type: "mrkdwn".to_string(),
                        text: Some(format!("*Type:* {} | *Time:* {}",
                            notification_type,
                            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
                        )),
                    }
                ]),
            },
        ];

        SlackMessage {
            text: title.to_string(), // Fallback text
            channel: Some(self.default_channel.clone()),
            username: Some("Notification Bot".to_string()),
            icon_emoji: Some(":bell:".to_string()),
            attachments: None,
            blocks: Some(blocks),
        }
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

        // Get the notification data
        let database_message = notification.to_database(notifiable).await?;
        let notification_type = notification.notification_type();

        // Create Slack message - use rich blocks for better formatting
        let mut slack_message = self.create_rich_message(notification_type, &database_message.data);
        slack_message.channel = Some(target_channel.clone());

        // Send the message
        match self.send_slack_message(slack_message).await {
            Ok(()) => {
                tracing::info!(
                    "Slack notification sent successfully to channel: {} (type: {})",
                    target_channel,
                    notification_type
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