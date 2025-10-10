use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::pinned_messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PinnedMessage {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub conversation_id: DieselUlid,
    pub message_id: DieselUlid,
    pub pinned_by_user_id: DieselUlid,
    pub pinned_by_device_id: DieselUlid,
    pub pinned_at: DateTime<Utc>,
    pub unpinned_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatePinnedMessage {
    pub conversation_id: DieselUlid,
    pub message_id: DieselUlid,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct PinnedMessageResponse {
    pub id: DieselUlid,
    pub conversation_id: DieselUlid,
    pub message_id: DieselUlid,
    pub pinned_by_user_id: DieselUlid,
    pub pinned_by_device_id: DieselUlid,
    pub pinned_at: DateTime<Utc>,
    pub unpinned_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

impl PinnedMessage {
    pub fn new(
        conversation_id: DieselUlid,
        message_id: DieselUlid,
        pinned_by_user_id: DieselUlid,
        pinned_by_device_id: DieselUlid,
    ) -> Self {
        let now = Utc::now();
        PinnedMessage {
            id: DieselUlid::new(),
            conversation_id,
            message_id,
            pinned_by_user_id,
            pinned_by_device_id,
            pinned_at: now,
            unpinned_at: None,
            is_active: true,
        }
    }

    pub fn to_response(&self) -> PinnedMessageResponse {
        PinnedMessageResponse {
            id: self.id,
            conversation_id: self.conversation_id,
            message_id: self.message_id,
            pinned_by_user_id: self.pinned_by_user_id,
            pinned_by_device_id: self.pinned_by_device_id,
            pinned_at: self.pinned_at,
            unpinned_at: self.unpinned_at,
            is_active: self.is_active,
        }
    }

    pub fn unpin(&mut self) {
        self.is_active = false;
        self.unpinned_at = Some(Utc::now());
    }

    pub fn repin(&mut self) {
        self.is_active = true;
        self.unpinned_at = None;
        self.pinned_at = Utc::now();
    }

    pub fn is_pinned(&self) -> bool {
        self.is_active && self.unpinned_at.is_none()
    }
}

impl HasId for PinnedMessage {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for PinnedMessage {
    fn table_name() -> &'static str {
        "pinned_messages"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "conversation_id",
            "message_id",
            "pinned_by_user_id",
            "pinned_by_device_id",
            "is_active",
            "pinned_at",
            "unpinned_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "conversation_id",
            "message_id",
            "pinned_by_user_id",
            "is_active",
            "pinned_at",
            "unpinned_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "conversation_id",
            "message_id",
            "pinned_by_user_id",
            "pinned_by_device_id",
            "pinned_at",
            "unpinned_at",
            "is_active",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("pinned_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "conversation",
            "message",
            "pinned_by_user",
            "pinned_by_device",
        ]
    }
}

crate::impl_query_builder_service!(PinnedMessage);