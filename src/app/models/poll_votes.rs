use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::poll_votes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PollVote {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub poll_id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_id: DieselUlid,
    pub encrypted_vote_data: String,
    pub vote_algorithm: String,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatePollVote {
    pub poll_id: DieselUlid,
    pub encrypted_vote_data: String,
    pub vote_algorithm: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdatePollVote {
    pub encrypted_vote_data: Option<String>,
    pub vote_algorithm: Option<String>,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct PollVoteResponse {
    pub id: DieselUlid,
    pub poll_id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_id: DieselUlid,
    pub encrypted_vote_data: String,
    pub vote_algorithm: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PollVote {
    pub fn new(
        poll_id: DieselUlid,
        user_id: DieselUlid,
        device_id: DieselUlid,
        encrypted_vote_data: String,
        vote_algorithm: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            poll_id,
            user_id,
            device_id,
            encrypted_vote_data,
            vote_algorithm,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> PollVoteResponse {
        PollVoteResponse {
            id: self.id,
            poll_id: self.poll_id,
            user_id: self.user_id,
            device_id: self.device_id,
            encrypted_vote_data: self.encrypted_vote_data.clone(),
            vote_algorithm: self.vote_algorithm.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl HasId for PollVote {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for PollVote {
    fn table_name() -> &'static str {
        "poll_votes"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "poll_id",
            "user_id",
            "device_id",
            "vote_algorithm",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "poll_id",
            "user_id",
            "device_id",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "poll_id",
            "user_id",
            "device_id",
            "encrypted_vote_data",
            "vote_algorithm",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Asc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "poll",
            "user",
            "device",
        ]
    }
}

crate::impl_query_builder_service!(PollVote);