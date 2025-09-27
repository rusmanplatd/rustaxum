use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, QueryableByName)]
#[diesel(table_name = crate::schema::sys_model_has_roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SysModelHasRole {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    #[schema(example = "User")]
    pub model_type: String,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub model_id: DieselUlid,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub role_id: DieselUlid,
    #[schema(example = "organization")]
    pub scope_type: Option<String>,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub scope_id: Option<DieselUlid>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub deleted_at: Option<DateTime<Utc>>,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub created_by: DieselUlid,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub updated_by: DieselUlid,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub deleted_by: Option<DieselUlid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSysModelHasRole {
    pub model_type: String,
    pub model_id: DieselUlid,
    pub role_id: DieselUlid,
    pub scope_type: Option<String>,
    pub scope_id: Option<DieselUlid>,
}

/// Insertable struct for sys_model_has_roles
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::sys_model_has_roles)]
pub struct NewSysModelHasRole {
    pub id: DieselUlid,
    pub model_type: String,
    pub model_id: DieselUlid,
    pub role_id: DieselUlid,
    pub scope_type: Option<String>,
    pub scope_id: Option<DieselUlid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by: DieselUlid,
    pub updated_by: DieselUlid,
    pub deleted_by: Option<DieselUlid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateSysModelHasRole {
    pub model_type: Option<String>,
    pub model_id: Option<DieselUlid>,
    pub role_id: Option<DieselUlid>,
    pub scope_type: Option<String>,
    pub scope_id: Option<DieselUlid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SysModelHasRoleResponse {
    pub id: DieselUlid,
    pub model_type: String,
    pub model_id: DieselUlid,
    pub role_id: DieselUlid,
    pub scope_type: Option<String>,
    pub scope_id: Option<DieselUlid>,
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
    pub fn new(model_type: String, model_id: DieselUlid, role_id: DieselUlid, scope_type: Option<String>, scope_id: Option<DieselUlid>) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            model_type,
            model_id,
            role_id,
            scope_type,
            scope_id,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: DieselUlid::new(), // TODO: Should be passed as parameter
            updated_by: DieselUlid::new(), // TODO: Should be passed as parameter
            deleted_by: None,
        }
    }
}

impl SysModelHasRole {
    pub fn new(model_type: String, model_id: DieselUlid, role_id: DieselUlid, scope_type: Option<String>, scope_id: Option<DieselUlid>) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            model_type,
            model_id,
            role_id,
            scope_type,
            scope_id,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: DieselUlid::new(), // TODO: Should be passed as parameter
            updated_by: DieselUlid::new(), // TODO: Should be passed as parameter
            deleted_by: None,
        }
    }

    pub fn to_response(&self) -> SysModelHasRoleResponse {
        SysModelHasRoleResponse {
            id: self.id,
            model_type: self.model_type.clone(),
            model_id: self.model_id,
            role_id: self.role_id,
            scope_type: self.scope_type.clone(),
            scope_id: self.scope_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

// Implement the query builder service for SysModelHasRole
crate::impl_query_builder_service!(SysModelHasRole);