use anyhow::Result;
use async_trait::async_trait;
use crate::database::DbPool;
use crate::app::notifications::channels::Channel;
use crate::app::notifications::notification::{Notification, Notifiable, NotificationChannel};
use crate::app::models::notification::Notification as NotificationModel;
use crate::config::Config;

#[derive(Debug)]
pub struct DatabaseChannel;

impl DatabaseChannel {
    pub fn new() -> Self {
        Self
    }

    async fn get_database_pool() -> Result<DbPool> {
        let config = Config::load()?;
        let pool = DbPool::connect(&config.database.url).await?;
        Ok(pool)
    }
}

#[async_trait]
impl Channel for DatabaseChannel {
    async fn send(&self, notification: &dyn Notification, notifiable: &dyn Notifiable) -> Result<()> {
        // Get database message from notification
        let database_message = notification.to_database(notifiable)?;

        // Create notification model
        let notification_model = NotificationModel::new(
            notification.notification_type().to_string(),
            notifiable.get_key(),
            // Extract the type name from the notifiable (simple implementation)
            // In a real implementation, you might want to make this more sophisticated
            notifiable.get_key().split('_').next().unwrap_or("Unknown").to_string(),
            database_message.data,
        );

        // Save to database
        let pool = Self::get_database_pool().await?;

        let query = r#"
            INSERT INTO notifications (id, notification_type, notifiable_id, notifiable_type, data, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#;

        sqlx::query(query)
            .bind(notification_model.id.to_string())
            .bind(&notification_model.notification_type)
            .bind(&notification_model.notifiable_id)
            .bind(&notification_model.notifiable_type)
            .bind(&notification_model.data)
            .bind(notification_model.created_at)
            .bind(notification_model.updated_at)
            .execute(&pool)
            .await?;

        tracing::info!(
            "Database notification stored for entity: {} (type: {})",
            notifiable.get_key(),
            notification.notification_type()
        );

        Ok(())
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Database
    }
}

impl DatabaseChannel {
    /// Get unread notifications for a notifiable entity
    pub async fn get_unread_notifications(
        notifiable_type: &str,
        notifiable_id: &str,
    ) -> Result<Vec<NotificationModel>> {
        let pool = Self::get_database_pool().await?;

        let query = r#"
            SELECT id, notification_type, notifiable_id, notifiable_type, data, read_at, created_at, updated_at
            FROM notifications
            WHERE notifiable_type = $1 AND notifiable_id = $2 AND read_at IS NULL
            ORDER BY created_at DESC
        "#;

        let notifications = sqlx::query_as::<_, NotificationModel>(query)
            .bind(notifiable_type)
            .bind(notifiable_id)
            .fetch_all(&pool)
            .await?;

        Ok(notifications)
    }

    /// Get all notifications for a notifiable entity
    pub async fn get_notifications(
        notifiable_type: &str,
        notifiable_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<NotificationModel>> {
        let pool = Self::get_database_pool().await?;

        let query = if let (Some(limit), Some(offset)) = (limit, offset) {
            format!(r#"
                SELECT id, notification_type, notifiable_id, notifiable_type, data, read_at, created_at, updated_at
                FROM notifications
                WHERE notifiable_type = $1 AND notifiable_id = $2
                ORDER BY created_at DESC
                LIMIT {} OFFSET {}
            "#, limit, offset)
        } else {
            r#"
                SELECT id, notification_type, notifiable_id, notifiable_type, data, read_at, created_at, updated_at
                FROM notifications
                WHERE notifiable_type = $1 AND notifiable_id = $2
                ORDER BY created_at DESC
            "#.to_string()
        };

        let notifications = sqlx::query_as::<_, NotificationModel>(&query)
            .bind(notifiable_type)
            .bind(notifiable_id)
            .fetch_all(&pool)
            .await?;

        Ok(notifications)
    }

    /// Mark notification as read
    pub async fn mark_as_read(notification_id: &str) -> Result<()> {
        let pool = Self::get_database_pool().await?;

        let query = r#"
            UPDATE notifications
            SET read_at = NOW(), updated_at = NOW()
            WHERE id = $1
        "#;

        sqlx::query(query)
            .bind(notification_id)
            .execute(&pool)
            .await?;

        Ok(())
    }

    /// Mark all notifications as read for a notifiable entity
    pub async fn mark_all_as_read(notifiable_type: &str, notifiable_id: &str) -> Result<()> {
        let pool = Self::get_database_pool().await?;

        let query = r#"
            UPDATE notifications
            SET read_at = NOW(), updated_at = NOW()
            WHERE notifiable_type = $1 AND notifiable_id = $2 AND read_at IS NULL
        "#;

        sqlx::query(query)
            .bind(notifiable_type)
            .bind(notifiable_id)
            .execute(&pool)
            .await?;

        Ok(())
    }

    /// Delete notification
    pub async fn delete_notification(notification_id: &str) -> Result<()> {
        let pool = Self::get_database_pool().await?;

        let query = "DELETE FROM notifications WHERE id = $1";

        sqlx::query(query)
            .bind(notification_id)
            .execute(&pool)
            .await?;

        Ok(())
    }
}