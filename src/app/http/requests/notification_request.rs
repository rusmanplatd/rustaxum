use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

/// Request structure for creating a new notification
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateNotificationRequest {
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
}

/// Request structure for updating an existing notification
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateNotificationRequest {
    /// When the notification was read
    pub read_at: Option<DateTime<Utc>>,
}