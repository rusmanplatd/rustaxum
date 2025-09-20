pub mod mail_channel;
pub mod database_channel;
pub mod broadcast_channel;
pub mod web_push_channel;
pub mod sms_channel;
pub mod slack_channel;

use anyhow::Result;
use async_trait::async_trait;
use crate::app::notifications::notification::{Notification, Notifiable, NotificationChannel};

/// Trait for notification channels
#[async_trait]
pub trait Channel: Send + Sync {
    /// Send a notification via this channel
    async fn send(&self, notification: &dyn Notification, notifiable: &dyn Notifiable) -> Result<()>;

    /// Get the channel identifier
    fn channel_type(&self) -> NotificationChannel;
}

/// Channel manager for routing notifications to appropriate channels
pub struct ChannelManager {
    mail_channel: mail_channel::MailChannel,
    database_channel: database_channel::DatabaseChannel,
    broadcast_channel: broadcast_channel::BroadcastChannel,
    web_push_channel: Option<web_push_channel::WebPushChannel>,
    sms_channel: Option<sms_channel::SmsChannel>,
    slack_channel: Option<slack_channel::SlackChannel>,
}

impl ChannelManager {
    pub async fn new() -> Self {
        // Try to initialize web push channel, but don't fail if not configured
        let web_push_channel = match web_push_channel::WebPushChannel::new().await {
            Ok(channel) => Some(channel),
            Err(e) => {
                tracing::warn!("Web push channel not available: {}", e);
                None
            }
        };

        // Try to initialize SMS channel, but don't fail if not configured
        let sms_channel = match sms_channel::SmsChannel::new() {
            Ok(channel) => Some(channel),
            Err(e) => {
                tracing::warn!("SMS channel not available: {}", e);
                None
            }
        };

        // Try to initialize Slack channel, but don't fail if not configured
        let slack_channel = match slack_channel::SlackChannel::new() {
            Ok(channel) => Some(channel),
            Err(e) => {
                tracing::warn!("Slack channel not available: {}", e);
                None
            }
        };

        Self {
            mail_channel: mail_channel::MailChannel::new(),
            database_channel: database_channel::DatabaseChannel::new(),
            broadcast_channel: broadcast_channel::BroadcastChannel::new(),
            web_push_channel,
            sms_channel,
            slack_channel,
        }
    }

    pub async fn send(
        &self,
        notification: &dyn Notification,
        notifiable: &dyn Notifiable,
        channels: Vec<NotificationChannel>,
    ) -> Result<()> {
        for channel in channels {
            match channel {
                NotificationChannel::Mail => {
                    self.mail_channel.send(notification, notifiable).await?;
                }
                NotificationChannel::Database => {
                    self.database_channel.send(notification, notifiable).await?;
                }
                NotificationChannel::Broadcast => {
                    self.broadcast_channel.send(notification, notifiable).await?;
                }
                NotificationChannel::WebPush => {
                    if let Some(web_push_channel) = &self.web_push_channel {
                        web_push_channel.send(notification, notifiable).await?;
                    } else {
                        tracing::warn!("Web push channel not configured");
                    }
                }
                NotificationChannel::Sms => {
                    if let Some(sms_channel) = &self.sms_channel {
                        sms_channel.send(notification, notifiable).await?;
                    } else {
                        tracing::warn!("SMS channel not configured");
                    }
                }
                NotificationChannel::Slack => {
                    if let Some(slack_channel) = &self.slack_channel {
                        slack_channel.send(notification, notifiable).await?;
                    } else {
                        tracing::warn!("Slack channel not configured");
                    }
                }
                NotificationChannel::Custom(channel_name) => {
                    tracing::warn!("Custom channel '{}' not implemented", channel_name);
                }
            }
        }
        Ok(())
    }
}