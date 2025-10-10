use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::{HasModelType, activity_log::HasId};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, QueryableByName, Insertable)]
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
    pub created_by_id: DieselUlid,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub updated_by_id: DieselUlid,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub deleted_by_id: Option<DieselUlid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSysModelHasRole {
    pub model_type: String,
    pub model_id: DieselUlid,
    pub role_id: DieselUlid,
    pub scope_type: Option<String>,
    pub scope_id: Option<DieselUlid>,
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

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "role",
            "model",
            "createdBy",
            "updatedBy",
            "deletedBy",
            "createdBy.organizations",
            "updatedBy.organizations",
            "deletedBy.organizations",
            "createdBy.organizations.position",
            "updatedBy.organizations.position",
            "deletedBy.organizations.position",
            "createdBy.organizations.position.level",
            "updatedBy.organizations.position.level",
            "deletedBy.organizations.position.level",
        ]
    }
}

impl SysModelHasRole {
    pub fn new(model_type: String, model_id: DieselUlid, role_id: DieselUlid, scope_type: Option<String>, scope_id: Option<DieselUlid>, user_id: Option<DieselUlid>) -> Self {
        let now = Utc::now();
        let created_by = user_id.unwrap_or_else(|| DieselUlid::from_string("01SYSTEM0SEEDER00000000000").unwrap_or_else(|_| DieselUlid::new()));

        SysModelHasRole {
            id: DieselUlid::new(),
            model_type,
            model_id,
            role_id,
            scope_type,
            scope_id,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by_id: created_by,
            updated_by_id: created_by,
            deleted_by_id: None,
        }
    }

    pub fn update_with_user(&mut self, user_id: Option<DieselUlid>) {
        self.updated_at = Utc::now();
        self.updated_by_id = user_id.unwrap_or_else(|| DieselUlid::from_string("01SYSTEM0SEEDER00000000000").unwrap());
    }

    pub fn soft_delete(&mut self, user_id: Option<DieselUlid>) {
        let now = Utc::now();
                self.deleted_at = Some(now);
        self.updated_at = now;
        self.deleted_by_id = user_id;
        self.updated_by_id = user_id.unwrap_or_else(|| DieselUlid::from_string("01SYSTEM0SEEDER00000000000").unwrap());
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

impl HasModelType for SysModelHasRole {
    fn model_type() -> &'static str {
        "SysModelHasRole"
    }
}

impl HasId for SysModelHasRole {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

// Implement the query builder service for SysModelHasRole
crate::impl_query_builder_service!(SysModelHasRole);