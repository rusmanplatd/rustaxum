use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::forward_history)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ForwardHistory {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub message_id: DieselUlid,
    pub original_message_id: DieselUlid,
    pub forwarded_by_user_id: DieselUlid,
    pub forwarded_by_device_id: DieselUlid,
    pub forward_depth: i32,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateForwardHistory {
    pub message_id: DieselUlid,
    pub original_message_id: DieselUlid,
    pub forwarded_by_user_id: DieselUlid,
    pub forwarded_by_device_id: DieselUlid,
    pub forward_depth: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::forward_history)]
pub struct NewForwardHistory {
    pub id: DieselUlid,
    pub message_id: DieselUlid,
    pub original_message_id: DieselUlid,
    pub forwarded_by_user_id: DieselUlid,
    pub forwarded_by_device_id: DieselUlid,
    pub forward_depth: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ForwardHistoryResponse {
    pub id: DieselUlid,
    pub message_id: DieselUlid,
    pub original_message_id: DieselUlid,
    pub forwarded_by_user_id: DieselUlid,
    pub forwarded_by_device_id: DieselUlid,
    pub forward_depth: i32,
    pub created_at: DateTime<Utc>,
}

impl ForwardHistory {
    pub fn new(
        message_id: DieselUlid,
        original_message_id: DieselUlid,
        forwarded_by_user_id: DieselUlid,
        forwarded_by_device_id: DieselUlid,
        forward_depth: i32,
    ) -> NewForwardHistory {
        NewForwardHistory {
            id: DieselUlid::new(),
            message_id,
            original_message_id,
            forwarded_by_user_id,
            forwarded_by_device_id,
            forward_depth,
            created_at: Utc::now(),
        }
    }

    pub fn to_response(&self) -> ForwardHistoryResponse {
        ForwardHistoryResponse {
            id: self.id,
            message_id: self.message_id,
            original_message_id: self.original_message_id,
            forwarded_by_user_id: self.forwarded_by_user_id,
            forwarded_by_device_id: self.forwarded_by_device_id,
            forward_depth: self.forward_depth,
            created_at: self.created_at,
        }
    }

    pub fn is_original_forward(&self) -> bool {
        self.forward_depth == 1
    }

    pub fn is_deeply_forwarded(&self) -> bool {
        self.forward_depth >= 5
    }

    pub fn is_chain_forward(&self) -> bool {
        self.forward_depth > 1
    }

    pub fn get_forward_chain_info(&self) -> String {
        match self.forward_depth {
            1 => "Original forward".to_string(),
            2..=4 => format!("Forwarded {} times", self.forward_depth),
            _ => "Heavily forwarded".to_string(),
        }
    }
}

impl HasId for ForwardHistory {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for ForwardHistory {
    fn table_name() -> &'static str {
        "forward_history"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "original_message_id",
            "forwarded_by_user_id",
            "forwarded_by_device_id",
            "forward_depth",
            "created_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "original_message_id",
            "forwarded_by_user_id",
            "forward_depth",
            "created_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "original_message_id",
            "forwarded_by_user_id",
            "forwarded_by_device_id",
            "forward_depth",
            "created_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "message",
            "original_message",
            "forwarded_by_user",
            "forwarded_by_device",
        ]
    }
}

crate::impl_query_builder_service!(ForwardHistory);