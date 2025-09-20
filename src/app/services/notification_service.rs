use anyhow::Result;
use crate::app::notifications::notification::{Notification, Notifiable};
use crate::app::notifications::channels::{ChannelManager};
use crate::app::notifications::channels::database_channel::DatabaseChannel;
use crate::app::models::notification::Notification as NotificationModel;

/// Service for managing and sending notifications
pub struct NotificationService {
    channel_manager: ChannelManager,
}

impl NotificationService {
    pub async fn new() -> Self {
        Self {
            channel_manager: ChannelManager::new().await,
        }
    }

    /// Send a notification to a single notifiable entity
    pub async fn send(
        &self,
        notification: &dyn Notification,
        notifiable: &dyn Notifiable,
    ) -> Result<()> {
        // Get the channels this notification should be sent on
        let channels = notification.via(notifiable);

        // Filter channels based on notifiable preferences (if implemented)
        let filtered_channels = if let Some(user) = self.try_as_user(notifiable) {
            let mut filtered = Vec::new();
            for channel in channels {
                if user.prefers_channel(&channel).await {
                    filtered.push(channel);
                }
            }
            filtered
        } else {
            channels
        };

        // Send the notification via the appropriate channels
        self.channel_manager
            .send(notification, notifiable, filtered_channels)
            .await?;

        tracing::info!(
            "Notification sent successfully: {} to {}",
            notification.notification_type(),
            notifiable.get_key()
        );

        Ok(())
    }

    /// Send a notification to multiple notifiable entities
    pub async fn send_to_many(
        &self,
        notification: &dyn Notification,
        notifiables: Vec<&dyn Notifiable>,
    ) -> Result<Vec<Result<()>>> {
        let mut results = Vec::new();

        for notifiable in notifiables {
            let result = self.send(notification, notifiable).await;
            results.push(result);
        }

        Ok(results)
    }

    /// Send a notification immediately without considering preferences
    pub async fn send_now(
        &self,
        notification: &dyn Notification,
        notifiable: &dyn Notifiable,
    ) -> Result<()> {
        let channels = notification.via(notifiable);
        self.channel_manager
            .send(notification, notifiable, channels)
            .await
    }

    /// Get unread notifications for a notifiable entity
    pub async fn get_unread_notifications(
        &self,
        notifiable_type: &str,
        notifiable_id: &str,
    ) -> Result<Vec<NotificationModel>> {
        DatabaseChannel::get_unread_notifications(notifiable_type, notifiable_id).await
    }

    /// Get all notifications for a notifiable entity with pagination
    pub async fn get_notifications(
        &self,
        notifiable_type: &str,
        notifiable_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<NotificationModel>> {
        DatabaseChannel::get_notifications(notifiable_type, notifiable_id, limit, offset).await
    }

    /// Mark a notification as read
    pub async fn mark_as_read(&self, notification_id: &str) -> Result<()> {
        DatabaseChannel::mark_as_read(notification_id).await
    }

    /// Mark all notifications as read for a notifiable entity
    pub async fn mark_all_as_read(
        &self,
        notifiable_type: &str,
        notifiable_id: &str,
    ) -> Result<()> {
        DatabaseChannel::mark_all_as_read(notifiable_type, notifiable_id).await
    }

    /// Delete a notification
    pub async fn delete_notification(&self, notification_id: &str) -> Result<()> {
        DatabaseChannel::delete_notification(notification_id).await
    }

    /// Count unread notifications for a notifiable entity
    pub async fn unread_count(
        &self,
        notifiable_type: &str,
        notifiable_id: &str,
    ) -> Result<i64> {
        let unread = self.get_unread_notifications(notifiable_type, notifiable_id).await?;
        Ok(unread.len() as i64)
    }

    // Helper method to try casting Notifiable to User (for preference checking)
    fn try_as_user(&self, notifiable: &dyn Notifiable) -> Option<&crate::app::models::user::User> {
        // This is a simplified approach. In a more sophisticated implementation,
        // you might want to use downcasting or a different approach
        if notifiable.get_key().starts_with("User_") {
            // For now, we'll return None since we can't safely downcast
            // In a real implementation, you might store the concrete type
            None
        } else {
            None
        }
    }
}


// Convenience functions for common notification operations
impl NotificationService {
    /// Create a simple notification service instance
    pub async fn create() -> Self {
        Self::new().await
    }

    /// Quick method to send a notification without creating a service instance
    pub async fn quick_send(
        notification: &dyn Notification,
        notifiable: &dyn Notifiable,
    ) -> Result<()> {
        let service = Self::new().await;
        service.send(notification, notifiable).await
    }
}