use anyhow::Result;
use async_trait::async_trait;
use crate::database::DbPool;
use crate::app::notifications::channels::Channel;
use crate::app::notifications::notification::{Notification, Notifiable, NotificationChannel};
use crate::app::models::notification::{Notification as NotificationModel, NewNotification};
use crate::config::Config;
use diesel::prelude::*;
use crate::schema::notifications;

#[derive(Debug)]
pub struct DatabaseChannel;

impl DatabaseChannel {
    pub fn new() -> Self {
        Self
    }

    fn get_database_pool() -> Result<DbPool> {
        let config = Config::load()?;
        let pool = crate::database::create_pool(&config)?;
        Ok(pool)
    }
}

#[async_trait]
impl Channel for DatabaseChannel {
    async fn send(&self, notification: &dyn Notification, notifiable: &dyn Notifiable) -> Result<()> {
        // Get database message from notification
        let database_message = notification.to_database(notifiable)?;

        // Create notification model
        let notification_model = NewNotification::new(
            notification.notification_type().to_string(),
            notifiable.get_key(),
            // Extract the type name from the notifiable (simple implementation)
            // In a real implementation, you might want to make this more sophisticated
            notifiable.get_key().split('_').next().unwrap_or("Unknown").to_string(),
            database_message.data,
        );

        // Save to database
        let pool = Self::get_database_pool()?;
        let mut conn = pool.get()?;

        diesel::insert_into(notifications::table)
            .values((
                notifications::id.eq(notification_model.id.to_string()),
                notifications::type_.eq(&notification_model.notification_type),
                notifications::notifiable_id.eq(&notification_model.notifiable_id),
                notifications::notifiable_type.eq(&notification_model.notifiable_type),
                notifications::data.eq(serde_json::to_value(&notification_model.data).unwrap()),
                notifications::created_at.eq(notification_model.created_at),
                notifications::updated_at.eq(notification_model.updated_at),
            ))
            .execute(&mut conn)?;

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
    pub fn get_unread_notifications(
        notifiable_type: &str,
        notifiable_id: &str,
    ) -> Result<Vec<NotificationModel>> {
        let pool = Self::get_database_pool()?;
        let mut conn = pool.get()?;

        let query = r#"
            SELECT id, type as notification_type, notifiable_id, notifiable_type, data, read_at, created_at, updated_at,
                   channels, sent_at, failed_at, retry_count, max_retries, error_message, priority, scheduled_at
            FROM notifications
            WHERE notifiable_type = $1 AND notifiable_id = $2 AND read_at IS NULL
            ORDER BY created_at DESC
        "#;

        let notifications = diesel::sql_query(query)
            .bind::<diesel::sql_types::Text, _>(notifiable_type)
            .bind::<diesel::sql_types::Text, _>(notifiable_id)
            .load::<NotificationModel>(&mut conn)?;

        Ok(notifications)
    }

    /// Get all notifications for a notifiable entity
    pub fn get_notifications(
        notifiable_type: &str,
        notifiable_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<NotificationModel>> {
        let pool = Self::get_database_pool()?;
        let mut conn = pool.get()?;

        let query = if let (Some(limit), Some(offset)) = (limit, offset) {
            format!(r#"
                SELECT id, type as notification_type, notifiable_id, notifiable_type, data, read_at, created_at, updated_at,
                       channels, sent_at, failed_at, retry_count, max_retries, error_message, priority, scheduled_at
                FROM notifications
                WHERE notifiable_type = $1 AND notifiable_id = $2
                ORDER BY created_at DESC
                LIMIT {} OFFSET {}
            "#, limit, offset)
        } else {
            r#"
                SELECT id, type as notification_type, notifiable_id, notifiable_type, data, read_at, created_at, updated_at,
                       channels, sent_at, failed_at, retry_count, max_retries, error_message, priority, scheduled_at
                FROM notifications
                WHERE notifiable_type = $1 AND notifiable_id = $2
                ORDER BY created_at DESC
            "#.to_string()
        };

        let notifications = diesel::sql_query(query)
            .bind::<diesel::sql_types::Text, _>(notifiable_type)
            .bind::<diesel::sql_types::Text, _>(notifiable_id)
            .load::<NotificationModel>(&mut conn)?;

        Ok(notifications)
    }

    /// Mark notification as read
    pub fn mark_as_read(notification_id: &str) -> Result<()> {
        let pool = Self::get_database_pool()?;
        let mut conn = pool.get()?;

        let query = r#"
            UPDATE notifications
            SET read_at = NOW(), updated_at = NOW()
            WHERE id = $1
        "#;

        diesel::sql_query(query)
            .bind::<diesel::sql_types::Text, _>(notification_id)
            .execute(&mut conn)?;

        Ok(())
    }

    /// Mark all notifications as read for a notifiable entity
    pub fn mark_all_as_read(notifiable_type: &str, notifiable_id: &str) -> Result<()> {
        let pool = Self::get_database_pool()?;
        let mut conn = pool.get()?;

        let query = r#"
            UPDATE notifications
            SET read_at = NOW(), updated_at = NOW()
            WHERE notifiable_type = $1 AND notifiable_id = $2 AND read_at IS NULL
        "#;

        diesel::sql_query(query)
            .bind::<diesel::sql_types::Text, _>(notifiable_type)
            .bind::<diesel::sql_types::Text, _>(notifiable_id)
            .execute(&mut conn)?;

        Ok(())
    }

    /// Delete notification
    pub fn delete_notification(notification_id: &str) -> Result<()> {
        let pool = Self::get_database_pool()?;
        let mut conn = pool.get()?;

        diesel::delete(notifications::table.filter(notifications::id.eq(notification_id)))
            .execute(&mut conn)?;

        Ok(())
    }
}