use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::schema::conversations;
use super::DieselUlid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = conversations)]
#[diesel(primary_key(id))]
pub struct Conversation {
    pub id: DieselUlid,
    pub conversation_type: String,
    pub is_encrypted: bool,
    pub encryption_immutable: bool,
    pub encrypted_name: Option<String>,
    pub encrypted_description: Option<String>,
    pub encrypted_avatar_url: Option<String>,
    pub preferred_algorithm: Option<String>,
    pub preferred_key_exchange: Option<String>,
    pub preferred_mac: Option<String>,
    pub creator_id: Option<DieselUlid>,
    pub max_participants: Option<i32>,
    pub is_public: bool,
    pub disappearing_messages_timer: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = conversations)]
pub struct NewConversation {
    pub id: DieselUlid,
    pub conversation_type: String,
    pub is_encrypted: Option<bool>,
    pub encryption_immutable: Option<bool>,
    pub encrypted_name: Option<String>,
    pub encrypted_description: Option<String>,
    pub encrypted_avatar_url: Option<String>,
    pub preferred_algorithm: Option<String>,
    pub preferred_key_exchange: Option<String>,
    pub preferred_mac: Option<String>,
    pub creator_id: Option<DieselUlid>,
    pub max_participants: Option<i32>,
    pub is_public: Option<bool>,
    pub disappearing_messages_timer: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationType {
    Direct,
    Group,
    Channel,
}

impl From<String> for ConversationType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "direct" => ConversationType::Direct,
            "group" => ConversationType::Group,
            "channel" => ConversationType::Channel,
            _ => ConversationType::Direct,
        }
    }
}

impl From<ConversationType> for String {
    fn from(ct: ConversationType) -> Self {
        match ct {
            ConversationType::Direct => "direct".to_string(),
            ConversationType::Group => "group".to_string(),
            ConversationType::Channel => "channel".to_string(),
        }
    }
}

impl Conversation {
    pub fn conversation_type_enum(&self) -> ConversationType {
        self.conversation_type.clone().into()
    }
}

impl NewConversation {
    pub fn new(conversation_type: ConversationType, creator_id: Option<DieselUlid>) -> Self {
        Self {
            id: DieselUlid::new(),
            conversation_type: conversation_type.into(),
            is_encrypted: Some(false),
            encryption_immutable: Some(false),
            encrypted_name: None,
            encrypted_description: None,
            encrypted_avatar_url: None,
            preferred_algorithm: Some("aes-256-gcm".to_string()),
            preferred_key_exchange: Some("curve25519".to_string()),
            preferred_mac: Some("hmac-sha256".to_string()),
            creator_id,
            max_participants: None,
            is_public: Some(false),
            disappearing_messages_timer: None,
        }
    }

    pub fn encrypted(mut self, is_encrypted: bool) -> Self {
        self.is_encrypted = Some(is_encrypted);
        self
    }

    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.encrypted_name = name;
        self
    }

    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.encrypted_description = description;
        self
    }

    pub fn public(mut self, is_public: bool) -> Self {
        self.is_public = Some(is_public);
        self
    }

    pub fn with_max_participants(mut self, max_participants: Option<i32>) -> Self {
        self.max_participants = max_participants;
        self
    }
}