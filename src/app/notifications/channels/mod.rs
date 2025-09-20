pub mod mail_channel;
pub mod database_channel;
pub mod broadcast_channel;
pub mod web_push_channel;

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

        Self {
            mail_channel: mail_channel::MailChannel::new(),
            database_channel: database_channel::DatabaseChannel::new(),
            broadcast_channel: broadcast_channel::BroadcastChannel::new(),
            web_push_channel,
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
                    // TODO: Implement SMS channel
                    tracing::warn!("SMS channel not implemented yet");
                }
                NotificationChannel::Slack => {
                    // TODO: Implement Slack channel
                    tracing::warn!("Slack channel not implemented yet");
                }
                NotificationChannel::Custom(channel_name) => {
                    tracing::warn!("Custom channel '{}' not implemented", channel_name);
                }
            }
        }
        Ok(())
    }
}