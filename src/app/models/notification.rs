use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::{SortDirection};

/// Database notification model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable, QueryableByName)]
#[diesel(table_name = crate::schema::notifications)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Notification {
    /// Unique notification identifier
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// Type of notification (class name) - maps to schema type_ field
    #[schema(example = "InvoicePaidNotification")]
    #[diesel(column_name = type_)]
    pub notification_type: String,
    /// Type of the notifiable entity
    #[schema(example = "User")]
    pub notifiable_type: String,
    /// ID of the notifiable entity
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub notifiable_id: String,
    /// Notification data as JSON
    pub data: serde_json::Value,
    /// Channels where notification will be sent
    pub channels: Vec<Option<String>>,
    /// When the notification was read
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub read_at: Option<DateTime<Utc>>,
    /// When the notification was sent
    pub sent_at: Option<DateTime<Utc>>,
    /// When the notification failed
    pub failed_at: Option<DateTime<Utc>>,
    /// Number of retry attempts
    pub retry_count: Option<i32>,
    /// Maximum number of retries
    pub max_retries: Option<i32>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Notification priority
    pub priority: Option<i32>,
    /// When notification should be sent
    pub scheduled_at: Option<DateTime<Utc>>,
    /// Notification creation timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// Create notification payload
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateNotification {
    pub notification_type: String,
    pub notifiable_id: String,
    pub notifiable_type: String,
    pub data: serde_json::Value,
}

/// Insertable struct for notifications
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::notifications)]
pub struct NewNotification {
    pub id: String,
    #[diesel(column_name = type_)]
    pub notification_type: String,
    pub notifiable_type: String,
    pub notifiable_id: String,
    pub data: serde_json::Value,
    pub channels: Vec<Option<String>>,
    pub read_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub retry_count: Option<i32>,
    pub max_retries: Option<i32>,
    pub error_message: Option<String>,
    pub priority: Option<i32>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Update notification payload
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateNotification {
    pub read_at: Option<DateTime<Utc>>,
}

/// Notification response payload
#[derive(Debug, Serialize, ToSchema)]
pub struct NotificationResponse {
    pub id: String,
    pub notification_type: String,
    pub notifiable_id: String,
    pub notifiable_type: String,
    pub data: serde_json::Value,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NewNotification {
    pub fn new(
        notification_type: String,
        notifiable_id: String,
        notifiable_type: String,
        data: serde_json::Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new().to_string(),
            notification_type,
            notifiable_type,
            notifiable_id,
            data,
            channels: vec![],
            read_at: None,
            sent_at: None,
            failed_at: None,
            retry_count: None,
            max_retries: None,
            error_message: None,
            priority: None,
            scheduled_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Notification {

    pub fn to_response(&self) -> NotificationResponse {
        NotificationResponse {
            id: self.id.clone(),
            notification_type: self.notification_type.clone(),
            notifiable_id: self.notifiable_id.clone(),
            notifiable_type: self.notifiable_type.clone(),
            data: self.data.clone(),
            read_at: self.read_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn mark_as_read(&mut self) {
        self.read_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn is_read(&self) -> bool {
        self.read_at.is_some()
    }

    pub fn is_unread(&self) -> bool {
        self.read_at.is_none()
    }
}

impl crate::app::query_builder::Queryable for Notification {
    fn table_name() -> &'static str {
        "notifications"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "notification_type",
            "notifiable_id",
            "notifiable_type",
            "read_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "notification_type",
            "notifiable_id",
            "notifiable_type",
            "read_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "notification_type",
            "notifiable_id",
            "notifiable_type",
            "data",
            "read_at",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }
}

// Implement the query builder service for Notification
crate::impl_query_builder_service!(Notification);