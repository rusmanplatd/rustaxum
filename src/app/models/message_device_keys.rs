use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::message_device_keys)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MessageDeviceKey {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub message_id: DieselUlid,
    pub recipient_device_id: DieselUlid,
    pub encrypted_message_key: String,
    pub key_algorithm: String,
    pub delivered_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateMessageDeviceKey {
    pub message_id: DieselUlid,
    pub recipient_device_id: DieselUlid,
    pub encrypted_message_key: String,
    pub key_algorithm: String,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageDeviceKeyResponse {
    pub id: DieselUlid,
    pub message_id: DieselUlid,
    pub recipient_device_id: DieselUlid,
    pub key_algorithm: String,
    pub delivered_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl MessageDeviceKey {
    pub fn new(
        message_id: DieselUlid,
        recipient_device_id: DieselUlid,
        encrypted_message_key: String,
        key_algorithm: String,
    ) -> Self {
        MessageDeviceKey {
            id: DieselUlid::new(),
            message_id,
            recipient_device_id,
            encrypted_message_key,
            key_algorithm,
            delivered_at: None,
            read_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn to_response(&self) -> MessageDeviceKeyResponse {
        MessageDeviceKeyResponse {
            id: self.id,
            message_id: self.message_id,
            recipient_device_id: self.recipient_device_id,
            key_algorithm: self.key_algorithm.clone(),
            delivered_at: self.delivered_at,
            read_at: self.read_at,
            created_at: self.created_at,
        }
    }

    pub fn is_delivered(&self) -> bool {
        self.delivered_at.is_some()
    }

    pub fn is_read(&self) -> bool {
        self.read_at.is_some()
    }

    pub fn mark_delivered(&mut self) {
        if self.delivered_at.is_none() {
            self.delivered_at = Some(Utc::now());
        }
    }

    pub fn mark_read(&mut self) {
        let now = Utc::now();
        if self.delivered_at.is_none() {
            self.delivered_at = Some(now);
        }
        if self.read_at.is_none() {
            self.read_at = Some(now);
        }
    }

    pub fn get_delivery_status(&self) -> &'static str {
        match (self.delivered_at, self.read_at) {
            (_, Some(_)) => "read",
            (Some(_), None) => "delivered",
            (None, None) => "pending",
        }
    }

    pub fn get_key_algorithm_family(&self) -> &'static str {
        if self.key_algorithm.contains("aes") {
            "aes"
        } else if self.key_algorithm.contains("chacha") {
            "chacha"
        } else if self.key_algorithm.contains("rsa") {
            "rsa"
        } else if self.key_algorithm.contains("curve25519") {
            "curve25519"
        } else {
            "unknown"
        }
    }

    pub fn is_post_quantum(&self) -> bool {
        self.key_algorithm.contains("kyber") ||
        self.key_algorithm.contains("dilithium") ||
        self.key_algorithm.contains("sphincs")
    }
}

impl HasId for MessageDeviceKey {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for MessageDeviceKey {
    fn table_name() -> &'static str {
        "message_device_keys"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "recipient_device_id",
            "key_algorithm",
            "delivered_at",
            "read_at",
            "created_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "recipient_device_id",
            "key_algorithm",
            "delivered_at",
            "read_at",
            "created_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "recipient_device_id",
            "key_algorithm",
            "delivered_at",
            "read_at",
            "created_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "message",
            "recipient_device",
        ]
    }
}

crate::impl_query_builder_service!(MessageDeviceKey);