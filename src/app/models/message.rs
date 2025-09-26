use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::schema::messages;
use super::DieselUlid;
use chrono::{DateTime, Utc};

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