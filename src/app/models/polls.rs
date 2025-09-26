use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::polls)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Poll {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub message_id: DieselUlid,
    pub conversation_id: DieselUlid,
    pub encrypted_question: String,
    pub encrypted_options: String,
    pub allows_multiple_votes: bool,
    pub is_anonymous: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_closed: bool,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatePoll {
    pub message_id: DieselUlid,
    pub conversation_id: DieselUlid,
    pub encrypted_question: String,
    pub encrypted_options: String,
    pub allows_multiple_votes: bool,
    pub is_anonymous: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdatePoll {
    pub encrypted_question: Option<String>,
    pub encrypted_options: Option<String>,
    pub allows_multiple_votes: Option<bool>,
    pub is_anonymous: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_closed: Option<bool>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::polls)]
pub struct NewPoll {
    pub id: DieselUlid,
    pub message_id: DieselUlid,
    pub conversation_id: DieselUlid,
    pub encrypted_question: String,
    pub encrypted_options: String,
    pub allows_multiple_votes: bool,
    pub is_anonymous: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_closed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PollResponse {
    pub id: DieselUlid,
    pub message_id: DieselUlid,
    pub conversation_id: DieselUlid,
    pub encrypted_question: String,
    pub encrypted_options: String,
    pub allows_multiple_votes: bool,
    pub is_anonymous: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_closed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Poll {
    pub fn new(
        message_id: DieselUlid,
        conversation_id: DieselUlid,
        encrypted_question: String,
        encrypted_options: String,
        allows_multiple_votes: bool,
        is_anonymous: bool,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            message_id,
            conversation_id,
            encrypted_question,
            encrypted_options,
            allows_multiple_votes,
            is_anonymous,
            expires_at,
            is_closed: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> PollResponse {
        PollResponse {
            id: self.id,
            message_id: self.message_id,
            conversation_id: self.conversation_id,
            encrypted_question: self.encrypted_question.clone(),
            encrypted_options: self.encrypted_options.clone(),
            allows_multiple_votes: self.allows_multiple_votes,
            is_anonymous: self.is_anonymous,
            expires_at: self.expires_at,
            is_closed: self.is_closed,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn is_active(&self) -> bool {
        !self.is_closed && (self.expires_at.is_none() || self.expires_at.unwrap() > Utc::now())
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some() && self.expires_at.unwrap() <= Utc::now()
    }

    pub fn close(&mut self) {
        self.is_closed = true;
        self.updated_at = Utc::now();
    }
}

impl HasId for Poll {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for Poll {
    fn table_name() -> &'static str {
        "polls"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "conversation_id",
            "allows_multiple_votes",
            "is_anonymous",
            "is_closed",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "conversation_id",
            "allows_multiple_votes",
            "is_anonymous",
            "is_closed",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "conversation_id",
            "encrypted_question",
            "encrypted_options",
            "allows_multiple_votes",
            "is_anonymous",
            "expires_at",
            "is_closed",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "message",
            "conversation",
            "votes",
        ]
    }
}

crate::impl_query_builder_service!(Poll);