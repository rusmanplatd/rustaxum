use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::app::query_builder::SortDirection;
use utoipa::ToSchema;
use crate::app::models::{HasModelType, activity_log::HasId};

use crate::schema::sessions;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SessionModel {
    pub id: String,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub payload: String,
    pub last_activity: i32,
}
#[derive(Debug, AsChangeset)]
#[diesel(table_name = sessions)]
pub struct UpdateSession {
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub payload: String,
    pub last_activity: i32,
}

impl SessionModel {
    pub fn new(
        id: String,
        payload: String,
        user_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        SessionModel {
            id,
            user_id,
            ip_address,
            user_agent,
            payload,
            last_activity: chrono::Utc::now().timestamp() as i32,
        }
    }

    pub fn is_expired(&self, lifetime_seconds: i32) -> bool {
        let current_time = chrono::Utc::now().timestamp() as i32;
        current_time - self.last_activity > lifetime_seconds
    }
}

impl UpdateSession {
    pub fn new(
        payload: String,
        user_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            user_id,
            ip_address,
            user_agent,
            payload,
            last_activity: chrono::Utc::now().timestamp() as i32,
        }
    }
}

impl crate::app::query_builder::Queryable for SessionModel {
    fn table_name() -> &'static str {
        "sessions"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "ip_address",
            "user_agent",
            "last_activity",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "ip_address",
            "last_activity",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "ip_address",
            "user_agent",
            "payload",
            "last_activity",
        ]
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec!["user"]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("last_activity", SortDirection::Desc))
    }
}

impl crate::app::query_builder::Filterable for SessionModel {
    fn apply_basic_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
        match (column, operator) {
            ("id", op) => {
                format!("{} {} '{}'", column, op, value.as_str().unwrap_or(""))
            }
            ("user_id", op) if op != "is_null" && op != "is_not_null" => {
                format!("{} {} '{}'", column, op, value.as_str().unwrap_or(""))
            }
            ("ip_address", "contains") | ("user_agent", "contains") => {
                format!("LOWER({}) LIKE LOWER('%{}%')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("ip_address", "starts_with") | ("user_agent", "starts_with") => {
                format!("LOWER({}) LIKE LOWER('{}%')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("ip_address", "ends_with") | ("user_agent", "ends_with") => {
                format!("LOWER({}) LIKE LOWER('%{}')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("user_id", "is_null") => {
                format!("{} IS NULL", column)
            }
            ("user_id", "is_not_null") => {
                format!("{} IS NOT NULL", column)
            }
            ("last_activity", op) => {
                format!("{} {} {}", column, op, value.as_i64().unwrap_or(0))
            }
            _ => format!("{} = '{}'", column, value.as_str().unwrap_or(""))
        }
    }
}

impl crate::app::query_builder::Sortable for SessionModel {
    fn apply_basic_sort(column: &str, direction: &str) -> String {
        match column {
            "id" | "user_id" | "ip_address" | "last_activity" => {
                format!("{} {}", column, direction.to_uppercase())
            }
            _ => format!("last_activity {}", direction.to_uppercase())
        }
    }
}

impl crate::app::query_builder::Includable for SessionModel {
    fn load_relationship(_ids: &[String], _relationship: &str, _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::json!({}))
    }

    fn load_relationships(_ids: &[String], _includes: &[String], _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Session response payload
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionResponse {
    pub id: String,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub payload: String,
    pub last_activity: i32,
}

impl SessionModel {
    pub fn to_response(&self) -> SessionResponse {
        SessionResponse {
            id: self.id.clone(),
            user_id: self.user_id.clone(),
            ip_address: self.ip_address.clone(),
            user_agent: self.user_agent.clone(),
            payload: self.payload.clone(),
            last_activity: self.last_activity,
        }
    }
}

impl HasModelType for SessionModel {
    fn model_type() -> &'static str {
        "SessionModel"
    }
}

impl HasId for SessionModel {
    fn id(&self) -> String {
        self.id.clone()
    }
}

// Implement the query builder service for SessionModel
crate::impl_query_builder_service!(SessionModel);