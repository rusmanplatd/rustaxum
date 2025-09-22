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

    /// Detect the notifiable type using multiple strategies for robust type detection
    fn detect_notifiable_type(notifiable: &dyn Notifiable) -> String {
        // Strategy 1: Use std::any::type_name if available through as_any
        if let Some(any_ref) = notifiable.as_any() {
            let type_name = std::any::type_name_of_val(any_ref);

            // Extract the final type name from the full path
            if let Some(last_segment) = type_name.split("::").last() {
                // Clean up generic type parameters if present
                let clean_type = last_segment.split('<').next().unwrap_or(last_segment);

                // Convert to Laravel-style naming convention
                match clean_type {
                    "User" => return "App\\Models\\User".to_string(),
                    "Organization" => return "App\\Models\\Organization".to_string(),
                    "Team" => return "App\\Models\\Team".to_string(),
                    _ => {
                        tracing::debug!("Detected type '{}' for notifiable", clean_type);
                        return format!("App\\Models\\{}", clean_type);
                    }
                }
            }
        }

        // Strategy 2: Parse the key format (e.g., "User_123" -> "User")
        let key = notifiable.get_key();
        if let Some(type_part) = key.split('_').next() {
            match type_part {
                "User" => return "App\\Models\\User".to_string(),
                "Organization" => return "App\\Models\\Organization".to_string(),
                "Team" => return "App\\Models\\Team".to_string(),
                _ => {
                    tracing::debug!("Inferred type '{}' from key format", type_part);
                    return format!("App\\Models\\{}", type_part);
                }
            }
        }

        // Strategy 3: Try to use a registry pattern (future enhancement)
        // This could be implemented with a static HashMap mapping trait object vtables to types

        // Fallback: Use a generic type indicator
        tracing::warn!("Could not detect specific type for notifiable with key '{}', using default", key);
        "App\\Models\\Notifiable".to_string()
    }
}

#[async_trait]
impl Channel for DatabaseChannel {
    async fn send(&self, notification: &dyn Notification, notifiable: &dyn Notifiable) -> Result<()> {
        // Get database message from notification
        let database_message = notification.to_database(notifiable)?;

        // Create notification model
        let notifiable_type = Self::detect_notifiable_type(notifiable);
        let notification_model = NewNotification::new(
            notification.notification_type().to_string(),
            notifiable.get_key(),
            notifiable_type,
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