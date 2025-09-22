use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, QueryableByName, Identifiable)]
#[diesel(table_name = crate::schema::sys_roles)]
pub struct Role {
    pub id: Ulid,
    pub name: String,
    pub description: Option<String>,
    pub guard_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateRole {
    pub name: String,
    pub description: Option<String>,
    pub guard_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateRole {
    pub name: Option<String>,
    pub description: Option<String>,
    pub guard_name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RoleResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub guard_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Role {
    pub fn new(name: String, description: Option<String>, guard_name: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new().to_string(),
            name,
            description,
            guard_name: guard_name.unwrap_or_else(|| "api".to_string()),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> RoleResponse {
        RoleResponse {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            guard_name: self.guard_name.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl crate::app::query_builder::Queryable for Role {
    fn table_name() -> &'static str {
        "sys_roles"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "description",
            "guard_name",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "description",
            "guard_name",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "description",
            "guard_name",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("name", SortDirection::Asc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "permissions",
            "users",
        ]
    }
}

// Implement the query builder service for Role
crate::impl_query_builder_service!(Role);

