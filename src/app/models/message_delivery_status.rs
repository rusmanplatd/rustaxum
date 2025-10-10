use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::schema::message_delivery_status;
use super::DieselUlid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = message_delivery_status)]
#[diesel(primary_key(id))]
pub struct MessageDeliveryStatus {
    pub id: DieselUlid,
    pub message_id: DieselUlid,
    pub recipient_device_id: DieselUlid,
    pub status: String,
    pub delivered_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Delivered,
    Read,
    Failed,
}

impl From<String> for DeliveryStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "pending" => DeliveryStatus::Pending,
            "sent" => DeliveryStatus::Sent,
            "delivered" => DeliveryStatus::Delivered,
            "read" => DeliveryStatus::Read,
            "failed" => DeliveryStatus::Failed,
            _ => DeliveryStatus::Pending,
        }
    }
}

impl From<DeliveryStatus> for String {
    fn from(ds: DeliveryStatus) -> Self {
        match ds {
            DeliveryStatus::Pending => "pending".to_string(),
            DeliveryStatus::Sent => "sent".to_string(),
            DeliveryStatus::Delivered => "delivered".to_string(),
            DeliveryStatus::Read => "read".to_string(),
            DeliveryStatus::Failed => "failed".to_string(),
        }
    }
}

impl MessageDeliveryStatus {
    pub fn status_enum(&self) -> DeliveryStatus {
        self.status.clone().into()
    }

    pub fn is_delivered(&self) -> bool {
        matches!(self.status_enum(), DeliveryStatus::Delivered | DeliveryStatus::Read)
    }

    pub fn is_read(&self) -> bool {
        matches!(self.status_enum(), DeliveryStatus::Read)
    }

    pub fn needs_retry(&self) -> bool {
        self.retry_count < self.max_retries && self.failed_at.is_some()
    }
}

impl MessageDeliveryStatus {
    pub fn new(message_id: DieselUlid, recipient_device_id: DieselUlid) -> Self {
        let now = chrono::Utc::now();
        MessageDeliveryStatus {
            id: DieselUlid::new(),
            message_id,
            recipient_device_id,
            status: DeliveryStatus::Pending.into(),
            delivered_at: None,
            read_at: None,
            failed_at: None,
            failure_reason: None,
            retry_count: 0,
            max_retries: 3,
            next_retry_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_status(mut self, status: DeliveryStatus) -> Self {
        self.status = status.into();
        self
    }
}