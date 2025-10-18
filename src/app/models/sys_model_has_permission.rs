use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use crate::app::models::DieselUlid;
use crate::app::models::{HasModelType, activity_log::HasId};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, QueryableByName, Insertable)]
#[diesel(table_name = crate::schema::sys_model_has_permissions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SysModelHasPermission {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    #[schema(example = "User")]
    pub model_type: String,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub model_id: DieselUlid,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub permission_id: DieselUlid,
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
pub struct CreateSysModelHasPermission {
    pub model_type: String,
    pub model_id: DieselUlid,
    pub permission_id: DieselUlid,
    pub scope_type: Option<String>,
    pub scope_id: Option<DieselUlid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateSysModelHasPermission {
    pub model_type: Option<String>,
    pub model_id: Option<DieselUlid>,
    pub permission_id: Option<DieselUlid>,
    pub scope_type: Option<String>,
    pub scope_id: Option<DieselUlid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SysModelHasPermissionResponse {
    pub id: DieselUlid,
    pub model_type: String,
    pub model_id: DieselUlid,
    pub permission_id: DieselUlid,
    pub scope_type: Option<String>,
    pub scope_id: Option<DieselUlid>,
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

impl SysModelHasPermission {
    pub fn new(model_type: String, model_id: DieselUlid, permission_id: DieselUlid, scope_type: Option<String>, scope_id: Option<DieselUlid>, created_by_id: DieselUlid) -> Self {
        let now = Utc::now();

        SysModelHasPermission {
            id: DieselUlid::new(),
            model_type,
            model_id,
            permission_id,
            scope_type,
            scope_id,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by_id,
            updated_by_id: created_by_id,
            deleted_by_id: None,
        }
    }

    pub fn update_with_user(&mut self, user_id: DieselUlid) {
        self.updated_at = Utc::now();
        self.updated_by_id = user_id;
    }

    pub fn soft_delete(&mut self, user_id: DieselUlid) {
        let now = Utc::now();
        self.deleted_at = Some(now);
        self.updated_at = now;
        self.deleted_by_id = Some(user_id);
        self.updated_by_id = user_id;
    }

    pub fn to_response(&self) -> SysModelHasPermissionResponse {
        SysModelHasPermissionResponse {
            id: self.id,
            model_type: self.model_type.clone(),
            model_id: self.model_id,
            permission_id: self.permission_id,
            scope_type: self.scope_type.clone(),
            scope_id: self.scope_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl HasModelType for SysModelHasPermission {
    fn model_type() -> &'static str {
        "SysModelHasPermission"
    }
}

impl HasId for SysModelHasPermission {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

// Implement the query builder service for SysModelHasPermission
crate::impl_query_builder_service!(SysModelHasPermission);