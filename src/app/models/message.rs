use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::schema::messages;
use super::DieselUlid;
use chrono::{DateTime, Utc};
use crate::app::query_builder::SortDirection;
use utoipa::ToSchema;
use crate::app::models::{HasModelType, activity_log::HasId};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = messages)]
#[diesel(primary_key(id))]
pub struct Message {
    pub id: DieselUlid,
    pub conversation_id: DieselUlid,
    pub sender_user_id: DieselUlid,
    pub sender_device_id: DieselUlid,
    pub message_type: String,
    pub encrypted_content: String,
    pub content_algorithm: String,
    pub reply_to_message_id: Option<DieselUlid>,
    pub forward_from_message_id: Option<DieselUlid>,
    pub edit_of_message_id: Option<DieselUlid>,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub sent_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = messages)]
pub struct NewMessage {
    pub id: DieselUlid,
    pub conversation_id: DieselUlid,
    pub sender_user_id: DieselUlid,
    pub sender_device_id: DieselUlid,
    pub message_type: Option<String>,
    pub encrypted_content: String,
    pub content_algorithm: String,
    pub reply_to_message_id: Option<DieselUlid>,
    pub forward_from_message_id: Option<DieselUlid>,
    pub edit_of_message_id: Option<DieselUlid>,
    pub is_edited: Option<bool>,
    pub is_deleted: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    Image,
    File,
    Audio,
    Video,
    Location,
    Contact,
    Poll,
    System,
}

impl From<String> for MessageType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "text" => MessageType::Text,
            "image" => MessageType::Image,
            "file" => MessageType::File,
            "audio" => MessageType::Audio,
            "video" => MessageType::Video,
            "location" => MessageType::Location,
            "contact" => MessageType::Contact,
            "poll" => MessageType::Poll,
            "system" => MessageType::System,
            _ => MessageType::Text,
        }
    }
}

impl From<MessageType> for String {
    fn from(mt: MessageType) -> Self {
        match mt {
            MessageType::Text => "text".to_string(),
            MessageType::Image => "image".to_string(),
            MessageType::File => "file".to_string(),
            MessageType::Audio => "audio".to_string(),
            MessageType::Video => "video".to_string(),
            MessageType::Location => "location".to_string(),
            MessageType::Contact => "contact".to_string(),
            MessageType::Poll => "poll".to_string(),
            MessageType::System => "system".to_string(),
        }
    }
}

impl Message {
    pub fn message_type_enum(&self) -> MessageType {
        self.message_type.clone().into()
    }
}

impl NewMessage {
    pub fn new(
        conversation_id: DieselUlid,
        sender_user_id: DieselUlid,
        sender_device_id: DieselUlid,
        encrypted_content: String,
        content_algorithm: String,
    ) -> Self {
        Self {
            id: DieselUlid::new(),
            conversation_id,
            sender_user_id,
            sender_device_id,
            message_type: Some("text".to_string()),
            encrypted_content,
            content_algorithm,
            reply_to_message_id: None,
            forward_from_message_id: None,
            edit_of_message_id: None,
            is_edited: Some(false),
            is_deleted: Some(false),
            expires_at: None,
            sent_at: Some(Utc::now()),
        }
    }

    pub fn with_type(mut self, message_type: MessageType) -> Self {
        self.message_type = Some(message_type.into());
        self
    }

    pub fn reply_to(mut self, message_id: DieselUlid) -> Self {
        self.reply_to_message_id = Some(message_id);
        self
    }

    pub fn forward_from(mut self, message_id: DieselUlid) -> Self {
        self.forward_from_message_id = Some(message_id);
        self
    }

    pub fn edit_of(mut self, message_id: DieselUlid) -> Self {
        self.edit_of_message_id = Some(message_id);
        self
    }

    pub fn expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}

/// Message response payload
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    pub id: DieselUlid,
    pub conversation_id: DieselUlid,
    pub sender_user_id: DieselUlid,
    pub sender_device_id: DieselUlid,
    pub message_type: String,
    pub encrypted_content: String,
    pub content_algorithm: String,
    pub reply_to_message_id: Option<DieselUlid>,
    pub forward_from_message_id: Option<DieselUlid>,
    pub edit_of_message_id: Option<DieselUlid>,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub sent_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Message {
    pub fn to_response(&self) -> MessageResponse {
        MessageResponse {
            id: self.id,
            conversation_id: self.conversation_id,
            sender_user_id: self.sender_user_id,
            sender_device_id: self.sender_device_id,
            message_type: self.message_type.clone(),
            encrypted_content: self.encrypted_content.clone(),
            content_algorithm: self.content_algorithm.clone(),
            reply_to_message_id: self.reply_to_message_id,
            forward_from_message_id: self.forward_from_message_id,
            edit_of_message_id: self.edit_of_message_id,
            is_edited: self.is_edited,
            is_deleted: self.is_deleted,
            expires_at: self.expires_at,
            sent_at: self.sent_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        }
    }
}

impl crate::app::query_builder::Queryable for Message {
    fn table_name() -> &'static str {
        "messages"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "conversation_id",
            "sender_user_id",
            "sender_device_id",
            "message_type",
            "reply_to_message_id",
            "forward_from_message_id",
            "edit_of_message_id",
            "is_edited",
            "is_deleted",
            "expires_at",
            "sent_at",
            "created_at",
            "updated_at",
            "deleted_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "conversation_id",
            "sender_user_id",
            "message_type",
            "is_edited",
            "is_deleted",
            "expires_at",
            "sent_at",
            "created_at",
            "updated_at",
            "deleted_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "conversation_id",
            "sender_user_id",
            "sender_device_id",
            "message_type",
            "encrypted_content",
            "content_algorithm",
            "reply_to_message_id",
            "forward_from_message_id",
            "edit_of_message_id",
            "is_edited",
            "is_deleted",
            "expires_at",
            "sent_at",
            "created_at",
            "updated_at",
            "deleted_at",
        ]
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec!["conversation", "sender", "sender_device", "reply_to", "forward_from", "edit_of", "mentions"]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("sent_at", SortDirection::Desc))
    }
}

impl crate::app::query_builder::Filterable for Message {
    fn apply_basic_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
        match (column, operator) {
            ("id", op) | ("conversation_id", op) | ("sender_user_id", op) | ("sender_device_id", op) => {
                format!("{} {} '{}'", column, op, value.as_str().unwrap_or(""))
            }
            ("message_type", "contains") => {
                format!("LOWER({}) LIKE LOWER('%{}%')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("message_type", "starts_with") => {
                format!("LOWER({}) LIKE LOWER('{}%')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("message_type", "ends_with") => {
                format!("LOWER({}) LIKE LOWER('%{}')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("is_edited", op) | ("is_deleted", op) => {
                format!("{} {} {}", column, op, value.as_bool().unwrap_or(false))
            }
            ("reply_to_message_id", "is_null") | ("forward_from_message_id", "is_null") |
            ("edit_of_message_id", "is_null") | ("expires_at", "is_null") | ("deleted_at", "is_null") => {
                format!("{} IS NULL", column)
            }
            ("reply_to_message_id", "is_not_null") | ("forward_from_message_id", "is_not_null") |
            ("edit_of_message_id", "is_not_null") | ("expires_at", "is_not_null") | ("deleted_at", "is_not_null") => {
                format!("{} IS NOT NULL", column)
            }
            ("expires_at", op) | ("sent_at", op) | ("created_at", op) | ("updated_at", op) | ("deleted_at", op) => {
                format!("{} {} '{}'", column, op, value.as_str().unwrap_or(""))
            }
            _ => format!("{} = '{}'", column, value.as_str().unwrap_or(""))
        }
    }
}

impl crate::app::query_builder::Sortable for Message {
    fn apply_basic_sort(column: &str, direction: &str) -> String {
        match column {
            "id" | "conversation_id" | "sender_user_id" | "message_type" | "is_edited" | "is_deleted" |
            "expires_at" | "sent_at" | "created_at" | "updated_at" | "deleted_at" => {
                format!("{} {}", column, direction.to_uppercase())
            }
            _ => format!("sent_at {}", direction.to_uppercase())
        }
    }
}

impl crate::app::query_builder::Includable for Message {
    fn load_relationship(_ids: &[String], _relationship: &str, _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::json!({}))
    }

    fn load_relationships(_ids: &[String], _includes: &[String], _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<()> {
        Ok(())
    }
}

impl HasModelType for Message {
    fn model_type() -> &'static str {
        "Message"
    }
}

impl HasId for Message {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

// Implement the query builder service for Message
crate::impl_query_builder_service!(Message);