use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, QueryableByName)]
#[diesel(table_name = crate::schema::sys_model_has_roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SysModelHasRole {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    #[schema(example = "User")]
    pub model_type: String,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub model_id: String,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub role_id: String,
    #[schema(example = "organization")]
    pub scope_type: Option<String>,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub scope_id: Option<String>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSysModelHasRole {
    pub model_type: String,
    pub model_id: String,
    pub role_id: String,
    pub scope_type: Option<String>,
    pub scope_id: Option<String>,
}

/// Insertable struct for sys_model_has_roles
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::sys_model_has_roles)]
pub struct NewSysModelHasRole {
    pub id: String,
    pub model_type: String,
    pub model_id: String,
    pub role_id: String,
    pub scope_type: Option<String>,
    pub scope_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateSysModelHasRole {
    pub model_type: Option<String>,
    pub model_id: Option<String>,
    pub role_id: Option<String>,
    pub scope_type: Option<String>,
    pub scope_id: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SysModelHasRoleResponse {
    pub id: String,
    pub model_type: String,
    pub model_id: String,
    pub role_id: String,
    pub scope_type: Option<String>,
    pub scope_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl crate::app::query_builder::Queryable for SysModelHasRole {
    fn table_name() -> &'static str {
        "sys_model_has_roles"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "model_type",
            "model_id",
            "role_id",
            "scope_type",
            "scope_id",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "model_type",
            "scope_type",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "model_type",
            "model_id",
            "role_id",
            "scope_type",
            "scope_id",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }
}

impl NewSysModelHasRole {
    pub fn new(model_type: String, model_id: String, role_id: String, scope_type: Option<String>, scope_id: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new().to_string(),
            model_type,
            model_id,
            role_id,
            scope_type,
            scope_id,
            created_at: now,
            updated_at: now,
        }
    }
}

impl SysModelHasRole {
    pub fn new(model_type: String, model_id: String, role_id: String, scope_type: Option<String>, scope_id: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new().to_string(),
            model_type,
            model_id,
            role_id,
            scope_type,
            scope_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> SysModelHasRoleResponse {
        SysModelHasRoleResponse {
            id: self.id.clone(),
            model_type: self.model_type.clone(),
            model_id: self.model_id.clone(),
            role_id: self.role_id.clone(),
            scope_type: self.scope_type.clone(),
            scope_id: self.scope_id.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

// Implement the query builder service for SysModelHasRole
crate::impl_query_builder_service!(SysModelHasRole);