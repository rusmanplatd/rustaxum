use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::scheduled_messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ScheduledMessage {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub message_id: DieselUlid,
    pub conversation_id: DieselUlid,
    pub sender_user_id: DieselUlid,
    pub sender_device_id: DieselUlid,
    pub scheduled_for: DateTime<Utc>,
    pub timezone: String,
    pub is_sent: bool,
    pub sent_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub is_cancelled: bool,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_by_device_id: Option<DieselUlid>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateScheduledMessage {
    pub message_id: DieselUlid,
    pub conversation_id: DieselUlid,
    pub scheduled_for: DateTime<Utc>,
    pub timezone: String,
    pub max_retries: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateScheduledMessage {
    pub scheduled_for: Option<DateTime<Utc>>,
    pub timezone: Option<String>,
    pub max_retries: Option<i32>,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct ScheduledMessageResponse {
    pub id: DieselUlid,
    pub message_id: DieselUlid,
    pub conversation_id: DieselUlid,
    pub sender_user_id: DieselUlid,
    pub sender_device_id: DieselUlid,
    pub scheduled_for: DateTime<Utc>,
    pub timezone: String,
    pub is_sent: bool,
    pub sent_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub is_cancelled: bool,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ScheduledMessage {
    pub fn new(
        message_id: DieselUlid,
        conversation_id: DieselUlid,
        sender_user_id: DieselUlid,
        sender_device_id: DieselUlid,
        scheduled_for: DateTime<Utc>,
        timezone: String,
        max_retries: Option<i32>,
    ) -> Self {
        let now = Utc::now();
        ScheduledMessage {
            id: DieselUlid::new(),
            message_id,
            conversation_id,
            sender_user_id,
            sender_device_id,
            scheduled_for,
            timezone,
            is_sent: false,
            sent_at: None,
            failed_at: None,
            failure_reason: None,
            retry_count: 0,
            max_retries: max_retries.unwrap_or(3),
            next_retry_at: None,
            is_cancelled: false,
            cancelled_at: None,
            cancelled_by_device_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> ScheduledMessageResponse {
        ScheduledMessageResponse {
            id: self.id,
            message_id: self.message_id,
            conversation_id: self.conversation_id,
            sender_user_id: self.sender_user_id,
            sender_device_id: self.sender_device_id,
            scheduled_for: self.scheduled_for,
            timezone: self.timezone.clone(),
            is_sent: self.is_sent,
            sent_at: self.sent_at,
            failed_at: self.failed_at,
            failure_reason: self.failure_reason.clone(),
            retry_count: self.retry_count,
            max_retries: self.max_retries,
            is_cancelled: self.is_cancelled,
            cancelled_at: self.cancelled_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn is_ready_to_send(&self) -> bool {
        !self.is_sent && !self.is_cancelled && Utc::now() >= self.scheduled_for
    }

    pub fn is_pending(&self) -> bool {
        !self.is_sent && !self.is_cancelled && Utc::now() < self.scheduled_for
    }

    pub fn mark_sent(&mut self) {
        self.is_sent = true;
        self.sent_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn mark_failed(&mut self, reason: String) {
        self.failed_at = Some(Utc::now());
        self.failure_reason = Some(reason);
        self.retry_count += 1;
        self.updated_at = Utc::now();

        if self.retry_count < self.max_retries {
            // Schedule retry in 5 minutes
            self.next_retry_at = Some(Utc::now() + chrono::Duration::minutes(5));
        }
    }

    pub fn cancel(&mut self, cancelled_by_device_id: DieselUlid) {
        self.is_cancelled = true;
        self.cancelled_at = Some(Utc::now());
        self.cancelled_by_device_id = Some(cancelled_by_device_id);
        self.updated_at = Utc::now();
    }

    pub fn can_retry(&self) -> bool {
        !self.is_sent && !self.is_cancelled &&
        self.retry_count < self.max_retries &&
        self.failed_at.is_some()
    }

    pub fn is_retry_ready(&self) -> bool {
        self.can_retry() &&
        self.next_retry_at.map(|retry_time| Utc::now() >= retry_time).unwrap_or(false)
    }
}

impl HasId for ScheduledMessage {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for ScheduledMessage {
    fn table_name() -> &'static str {
        "scheduled_messages"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "conversation_id",
            "sender_user_id",
            "sender_device_id",
            "scheduled_for",
            "timezone",
            "is_sent",
            "sent_at",
            "failed_at",
            "retry_count",
            "max_retries",
            "is_cancelled",
            "cancelled_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "scheduled_for",
            "is_sent",
            "sent_at",
            "failed_at",
            "retry_count",
            "is_cancelled",
            "cancelled_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "message_id",
            "conversation_id",
            "sender_user_id",
            "sender_device_id",
            "scheduled_for",
            "timezone",
            "is_sent",
            "sent_at",
            "failed_at",
            "failure_reason",
            "retry_count",
            "max_retries",
            "is_cancelled",
            "cancelled_at",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("scheduled_for", SortDirection::Asc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "message",
            "conversation",
            "sender_user",
            "sender_device",
        ]
    }
}

crate::impl_query_builder_service!(ScheduledMessage);