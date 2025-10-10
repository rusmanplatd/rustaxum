use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::message_mentions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MessageMention {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub message_id: DieselUlid,
    pub mentioned_user_id: DieselUlid,
    pub mention_type: String,
    pub mention_start_pos: Option<i32>,
    pub mention_length: Option<i32>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MentionType {
    User,        // @username
    Everyone,    // @everyone
    Here,        // @here
    Role,        // @role
    Channel,     // #channel
}

impl From<String> for MentionType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "user" => MentionType::User,
            "everyone" => MentionType::Everyone,
            "here" => MentionType::Here,
            "role" => MentionType::Role,
            "channel" => MentionType::Channel,
            _ => MentionType::User,
        }
    }
}

impl From<MentionType> for String {
    fn from(mention_type: MentionType) -> Self {
        match mention_type {
            MentionType::User => "user".to_string(),
            MentionType::Everyone => "everyone".to_string(),
            MentionType::Here => "here".to_string(),
            MentionType::Role => "role".to_string(),
            MentionType::Channel => "channel".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateMessageMention {
    pub message_id: DieselUlid,
    pub mentioned_user_id: DieselUlid,
    pub mention_type: String,
    pub mention_start_pos: Option<i32>,
    pub mention_length: Option<i32>,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageMentionResponse {
    pub id: DieselUlid,
    pub message_id: DieselUlid,
    pub mentioned_user_id: DieselUlid,
    pub mention_type: String,
    pub mention_start_pos: Option<i32>,
    pub mention_length: Option<i32>,
    pub created_at: DateTime<Utc>,
}

impl MessageMention {
    pub fn new(
        message_id: DieselUlid,
        mentioned_user_id: DieselUlid,
        mention_type: MentionType,
        mention_start_pos: Option<i32>,
        mention_length: Option<i32>,
    ) -> Self {
        MessageMention {
            id: DieselUlid::new(),
            message_id,
            mentioned_user_id,
            mention_type: mention_type.into(),
            mention_start_pos,
            mention_length,
            created_at: Utc::now(),
        }
    }

    pub fn to_response(&self) -> MessageMentionResponse {
        MessageMentionResponse {
            id: self.id,
            message_id: self.message_id,
            mentioned_user_id: self.mentioned_user_id,
            mention_type: self.mention_type.clone(),
            mention_start_pos: self.mention_start_pos,
            mention_length: self.mention_length,
            created_at: self.created_at,
        }
    }

    pub fn mention_type_enum(&self) -> MentionType {
        self.mention_type.clone().into()
    }

    pub fn is_user_mention(&self) -> bool {
        matches!(self.mention_type_enum(), MentionType::User)
    }

    pub fn is_everyone_mention(&self) -> bool {
        matches!(self.mention_type_enum(), MentionType::Everyone)
    }

    pub fn is_here_mention(&self) -> bool {
        matches!(self.mention_type_enum(), MentionType::Here)
    }

    pub fn is_broadcast_mention(&self) -> bool {
        matches!(self.mention_type_enum(), MentionType::Everyone | MentionType::Here)
    }

    pub fn has_position_data(&self) -> bool {
        self.mention_start_pos.is_some() && self.mention_length.is_some()
    }
}

impl HasId for MessageMention {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for MessageMention {
    fn table_name() -> &'static str {
        "message_mentions"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "mentioned_user_id",
            "mention_type",
            "mention_start_pos",
            "mention_length",
            "created_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "mentioned_user_id",
            "mention_type",
            "mention_start_pos",
            "created_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "mentioned_user_id",
            "mention_type",
            "mention_start_pos",
            "mention_length",
            "created_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Asc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "message",
            "mentioned_user",
        ]
    }
}

crate::impl_query_builder_service!(MessageMention);