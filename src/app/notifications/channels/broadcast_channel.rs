use anyhow::Result;
use async_trait::async_trait;
use crate::app::notifications::channels::Channel;
use crate::app::notifications::notification::{Notification, Notifiable, NotificationChannel};
use crate::app::broadcasting::{BroadcastMessage as WebSocketBroadcastMessage, websocket::websocket_manager};

#[derive(Debug)]
pub struct BroadcastChannel;

impl BroadcastChannel {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Channel for BroadcastChannel {
    async fn send(&self, notification: &dyn Notification, notifiable: &dyn Notifiable) -> Result<()> {
        // Get broadcast message from notification
        let broadcast_message = notification.to_broadcast(notifiable).await?;

        // Create the channel name for this specific user/entity
        let channel = format!("user.{}", notifiable.get_key());

        // Create the WebSocket broadcast message
        let websocket_message = WebSocketBroadcastMessage {
            channel: channel.clone(),
            event: notification.notification_type().to_string(),
            data: broadcast_message.data,
            timestamp: chrono::Utc::now(),
        };

        // Get the global WebSocket manager and broadcast the message
        let manager = websocket_manager().await;
        match manager.broadcast(websocket_message).await {
            Ok(_) => {
                tracing::info!(
                    "Successfully broadcasted notification to channel '{}' for entity: {} (type: {})",
                    channel,
                    notifiable.get_key(),
                    notification.notification_type()
                );
            }
            Err(e) => {
                tracing::error!(
                    "Failed to broadcast notification to channel '{}': {}",
                    channel,
                    e
                );
                return Err(e);
            }
        }

        Ok(())
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Broadcast
    }
}

// Future implementation ideas:
//
// 1. WebSocket Broadcasting:
// impl BroadcastChannel {
//     async fn broadcast_via_websocket(&self, message: &BroadcastMessage, recipient: &str) -> Result<()> {
//         // Get active WebSocket connections for the recipient
//         // Send the message to all connected clients
//     }
// }
//
// 2. Redis Pub/Sub:
// impl BroadcastChannel {
//     async fn broadcast_via_redis(&self, message: &BroadcastMessage, recipient: &str) -> Result<()> {
//         // Publish to Redis channel
//         // Clients subscribe to these channels for real-time updates
//     }
// }
//
// 3. Server-Sent Events (SSE):
// impl BroadcastChannel {
//     async fn broadcast_via_sse(&self, message: &BroadcastMessage, recipient: &str) -> Result<()> {
//         // Send to SSE connections for the recipient
//     }
// }