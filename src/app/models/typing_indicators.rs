use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::typing_indicators)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TypingIndicator {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub conversation_id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_id: DieselUlid,
    pub is_typing: bool,
    pub started_typing_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTypingIndicator {
    pub conversation_id: DieselUlid,
    pub is_typing: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateTypingIndicator {
    pub is_typing: bool,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct TypingIndicatorResponse {
    pub id: DieselUlid,
    pub conversation_id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_id: DieselUlid,
    pub is_typing: bool,
    pub started_typing_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TypingIndicator {
    pub fn new(
        conversation_id: DieselUlid,
        user_id: DieselUlid,
        device_id: DieselUlid,
        is_typing: bool,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(30); // Typing indicators expire after 30 seconds

        TypingIndicator {
            id: DieselUlid::new(),
            conversation_id,
            user_id,
            device_id,
            is_typing,
            started_typing_at: now,
            expires_at,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> TypingIndicatorResponse {
        TypingIndicatorResponse {
            id: self.id,
            conversation_id: self.conversation_id,
            user_id: self.user_id,
            device_id: self.device_id,
            is_typing: self.is_typing,
            started_typing_at: self.started_typing_at,
            expires_at: self.expires_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_active(&self) -> bool {
        self.is_typing && !self.is_expired()
    }

    pub fn stop_typing(&mut self) {
        self.is_typing = false;
        self.updated_at = Utc::now();
    }

    pub fn refresh(&mut self) {
        let now = Utc::now();
        self.started_typing_at = now;
        self.expires_at = now + chrono::Duration::seconds(30);
        self.updated_at = now;
    }
}

impl HasId for TypingIndicator {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for TypingIndicator {
    fn table_name() -> &'static str {
        "typing_indicators"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "conversation_id",
            "user_id",
            "device_id",
            "is_typing",
            "started_typing_at",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "conversation_id",
            "user_id",
            "device_id",
            "started_typing_at",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "conversation_id",
            "user_id",
            "device_id",
            "is_typing",
            "started_typing_at",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("started_typing_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "conversation",
            "user",
            "device",
        ]
    }
}

crate::impl_query_builder_service!(TypingIndicator);