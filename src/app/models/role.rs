use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use crate::app::models::{DieselUlid, HasModelType};
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, QueryableByName, Identifiable)]
#[diesel(table_name = crate::schema::sys_roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Role {
    pub id: DieselUlid,
    pub organization_id: Option<DieselUlid>,
    pub name: String,
    pub description: Option<String>,
    pub guard_name: String,
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
    pub id: DieselUlid,
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
            id: DieselUlid::new(),
            organization_id: None,
            name,
            description,
            guard_name: guard_name.unwrap_or_else(|| "api".to_string()),
            scope_type: None,
            scope_id: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: DieselUlid::new(), // TODO: Should be passed as parameter
            updated_by: DieselUlid::new(), // TODO: Should be passed as parameter
            deleted_by: None,
        }
    }

    pub fn to_response(&self) -> RoleResponse {
        RoleResponse {
            id: self.id,
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

impl HasModelType for Role {
    fn model_type() -> &'static str {
        "Role"
    }
}

impl HasId for Role {
    fn id(&self) -> String {
        self.id.to_string()
    }
}
