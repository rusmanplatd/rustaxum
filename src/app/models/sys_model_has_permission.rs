use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, QueryableByName)]
#[diesel(table_name = crate::schema::sys_model_has_permissions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SysModelHasPermission {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    #[schema(example = "User")]
    pub model_type: String,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub model_id: String,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub permission_id: String,
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
pub struct CreateSysModelHasPermission {
    pub model_type: String,
    pub model_id: String,
    pub permission_id: String,
    pub scope_type: Option<String>,
    pub scope_id: Option<String>,
}

/// Insertable struct for sys_model_has_permissions
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::sys_model_has_permissions)]
pub struct NewSysModelHasPermission {
    pub id: String,
    pub model_type: String,
    pub model_id: String,
    pub permission_id: String,
    pub scope_type: Option<String>,
    pub scope_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateSysModelHasPermission {
    pub model_type: Option<String>,
    pub model_id: Option<String>,
    pub permission_id: Option<String>,
    pub scope_type: Option<String>,
    pub scope_id: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SysModelHasPermissionResponse {
    pub id: String,
    pub model_type: String,
    pub model_id: String,
    pub permission_id: String,
    pub scope_type: Option<String>,
    pub scope_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl crate::app::query_builder::Queryable for SysModelHasPermission {
    fn table_name() -> &'static str {
        "sys_model_has_permissions"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "model_type",
            "model_id",
            "permission_id",
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
            "model_id",
            "permission_id",
            "scope_type",
            "scope_id",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "model_type",
            "model_id",
            "permission_id",
            "scope_type",
            "scope_id",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "permission",
        ]
    }
}

impl NewSysModelHasPermission {
    pub fn new(model_type: String, model_id: String, permission_id: String, scope_type: Option<String>, scope_id: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new().to_string(),
            model_type,
            model_id,
            permission_id,
            scope_type,
            scope_id,
            created_at: now,
            updated_at: now,
        }
    }
}

impl SysModelHasPermission {
    pub fn new(model_type: String, model_id: String, permission_id: String, scope_type: Option<String>, scope_id: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new().to_string(),
            model_type,
            model_id,
            permission_id,
            scope_type,
            scope_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> SysModelHasPermissionResponse {
        SysModelHasPermissionResponse {
            id: self.id.clone(),
            model_type: self.model_type.clone(),
            model_id: self.model_id.clone(),
            permission_id: self.permission_id.clone(),
            scope_type: self.scope_type.clone(),
            scope_id: self.scope_id.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

// Implement the query builder service for SysModelHasPermission
crate::impl_query_builder_service!(SysModelHasPermission);