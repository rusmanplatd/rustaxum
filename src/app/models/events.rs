use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Event {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub event_name: String,
    pub event_data: serde_json::Value,
    pub aggregate_id: Option<String>,
    pub aggregate_type: Option<String>,
    pub version: Option<i32>,
    pub occurred_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateEvent {
    pub event_name: String,
    pub event_data: serde_json::Value,
    pub aggregate_id: Option<String>,
    pub aggregate_type: Option<String>,
    pub version: Option<i32>,
    pub occurred_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::events)]
pub struct NewEvent {
    pub id: DieselUlid,
    pub event_name: String,
    pub event_data: serde_json::Value,
    pub aggregate_id: Option<String>,
    pub aggregate_type: Option<String>,
    pub version: Option<i32>,
    pub occurred_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventResponse {
    pub id: DieselUlid,
    pub event_name: String,
    pub event_data: serde_json::Value,
    pub aggregate_id: Option<String>,
    pub aggregate_type: Option<String>,
    pub version: Option<i32>,
    pub occurred_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Event {
    pub fn new(
        event_name: String,
        event_data: serde_json::Value,
        aggregate_id: Option<String>,
        aggregate_type: Option<String>,
        version: Option<i32>,
        occurred_at: Option<DateTime<Utc>>,
    ) -> NewEvent {
        let now = Utc::now();
        NewEvent {
            id: DieselUlid::new(),
            event_name,
            event_data,
            aggregate_id,
            aggregate_type,
            version,
            occurred_at: occurred_at.unwrap_or(now),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> EventResponse {
        EventResponse {
            id: self.id,
            event_name: self.event_name.clone(),
            event_data: self.event_data.clone(),
            aggregate_id: self.aggregate_id.clone(),
            aggregate_type: self.aggregate_type.clone(),
            version: self.version,
            occurred_at: self.occurred_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn is_aggregate_event(&self) -> bool {
        self.aggregate_id.is_some() && self.aggregate_type.is_some()
    }

    pub fn get_aggregate_key(&self) -> Option<String> {
        match (&self.aggregate_type, &self.aggregate_id) {
            (Some(agg_type), Some(agg_id)) => Some(format!("{}:{}", agg_type, agg_id)),
            _ => None,
        }
    }
}

impl HasId for Event {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for Event {
    fn table_name() -> &'static str {
        "events"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "event_name",
            "aggregate_id",
            "aggregate_type",
            "version",
            "occurred_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "event_name",
            "aggregate_id",
            "aggregate_type",
            "version",
            "occurred_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "event_name",
            "event_data",
            "aggregate_id",
            "aggregate_type",
            "version",
            "occurred_at",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("occurred_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![]
    }
}

crate::impl_query_builder_service!(Event);