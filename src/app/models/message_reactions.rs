use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::message_reactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MessageReaction {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub message_id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_id: DieselUlid,
    pub encrypted_reaction: String,
    pub reaction_algorithm: String,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateMessageReaction {
    pub message_id: DieselUlid,
    pub encrypted_reaction: String,
    pub reaction_algorithm: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateMessageReaction {
    pub encrypted_reaction: Option<String>,
    pub reaction_algorithm: Option<String>,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageReactionResponse {
    pub id: DieselUlid,
    pub message_id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_id: DieselUlid,
    pub encrypted_reaction: String,
    pub reaction_algorithm: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MessageReaction {
    pub fn new(
        message_id: DieselUlid,
        user_id: DieselUlid,
        device_id: DieselUlid,
        encrypted_reaction: String,
        reaction_algorithm: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            message_id,
            user_id,
            device_id,
            encrypted_reaction,
            reaction_algorithm,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> MessageReactionResponse {
        MessageReactionResponse {
            id: self.id,
            message_id: self.message_id,
            user_id: self.user_id,
            device_id: self.device_id,
            encrypted_reaction: self.encrypted_reaction.clone(),
            reaction_algorithm: self.reaction_algorithm.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl HasId for MessageReaction {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for MessageReaction {
    fn table_name() -> &'static str {
        "message_reactions"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "user_id",
            "device_id",
            "reaction_algorithm",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "user_id",
            "device_id",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "user_id",
            "device_id",
            "encrypted_reaction",
            "reaction_algorithm",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Asc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "message",
            "user",
            "device",
        ]
    }
}

crate::impl_query_builder_service!(MessageReaction);