use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::query_builder::{Queryable, SortDirection};

/// Database notification model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Notification {
    /// Unique notification identifier
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: Ulid,
    /// Type of notification (class name)
    #[schema(example = "InvoicePaidNotification")]
    pub notification_type: String,
    /// ID of the notifiable entity
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub notifiable_id: String,
    /// Type of the notifiable entity
    #[schema(example = "User")]
    pub notifiable_type: String,
    /// Notification data as JSON
    pub data: serde_json::Value,
    /// When the notification was read
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub read_at: Option<DateTime<Utc>>,
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

impl Notification {
    pub fn new(
        notification_type: String,
        notifiable_id: String,
        notifiable_type: String,
        data: serde_json::Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            notification_type,
            notifiable_id,
            notifiable_type,
            data,
            read_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> NotificationResponse {
        NotificationResponse {
            id: self.id.to_string(),
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

impl FromRow<'_, PgRow> for Notification {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        let data: serde_json::Value = row.try_get("data")?;

        Ok(Notification {
            id,
            notification_type: row.try_get("notification_type")?,
            notifiable_id: row.try_get("notifiable_id")?,
            notifiable_type: row.try_get("notifiable_type")?,
            data,
            read_at: row.try_get("read_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl Queryable for Notification {
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