use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use crate::app::models::{DieselUlid, HasModelType};
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, QueryableByName, Identifiable, Insertable)]
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
    pub created_by_id: DieselUlid,
    pub updated_by_id: DieselUlid,
    pub deleted_by_id: Option<DieselUlid>,
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
    pub fn new(name: String, description: Option<String>, guard_name: Option<String>, created_by: DieselUlid) -> Self {
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
            created_by_id: created_by.clone(),
            updated_by_id: created_by,
            deleted_by_id: None,
        }
    }

    pub fn update_with_user(&mut self, updated_by: DieselUlid) {
        self.updated_at = Utc::now();
        self.updated_by_id = updated_by;
    }

    pub fn soft_delete(&mut self, deleted_by: DieselUlid) {
        let now = Utc::now();

        self.deleted_at = Some(now);
        self.updated_at = now;
        self.deleted_by_id = Some(deleted_by.clone());
        self.updated_by_id = deleted_by;
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
            "organization",
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

// Implement enhanced query builder traits for Role
impl crate::app::query_builder::Filterable for Role {
    fn apply_basic_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
        match (column, operator) {
            ("id", op) | ("organization_id", op) | ("scope_id", op) | ("created_by_id", op) | ("updated_by_id", op) | ("deleted_by_id", op) if op != "is_null" && op != "is_not_null" => {
                format!("{} {} '{}'", column, op, value.as_str().unwrap_or(""))
            }
            ("name", "contains") | ("description", "contains") | ("guard_name", "contains") | ("scope_type", "contains") => {
                format!("LOWER({}) LIKE LOWER('%{}%')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "starts_with") | ("description", "starts_with") | ("guard_name", "starts_with") | ("scope_type", "starts_with") => {
                format!("LOWER({}) LIKE LOWER('{}%')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "ends_with") | ("description", "ends_with") | ("guard_name", "ends_with") | ("scope_type", "ends_with") => {
                format!("LOWER({}) LIKE LOWER('%{}')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "like") | ("description", "like") | ("guard_name", "like") | ("scope_type", "like") => {
                format!("{} LIKE '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "ilike") | ("description", "ilike") | ("guard_name", "ilike") | ("scope_type", "ilike") => {
                format!("{} ILIKE '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "eq") | ("description", "eq") | ("guard_name", "eq") | ("scope_type", "eq") => {
                format!("{} = '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "ne") | ("description", "ne") | ("guard_name", "ne") | ("scope_type", "ne") => {
                format!("{} != '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "in") | ("description", "in") | ("guard_name", "in") | ("scope_type", "in") | ("organization_id", "in") => {
                let values = value.as_array()
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| format!("'{}'", s.replace('\'', "''")))
                        .collect::<Vec<_>>()
                        .join(","))
                    .unwrap_or_default();
                format!("{} IN ({})", column, values)
            }
            ("name", "not_in") | ("description", "not_in") | ("guard_name", "not_in") | ("scope_type", "not_in") | ("organization_id", "not_in") => {
                let values = value.as_array()
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| format!("'{}'", s.replace('\'', "''")))
                        .collect::<Vec<_>>()
                        .join(","))
                    .unwrap_or_default();
                format!("{} NOT IN ({})", column, values)
            }
            ("created_at", "gt") | ("updated_at", "gt") | ("deleted_at", "gt") => {
                format!("{} > '{}'", column, value.as_str().unwrap_or(""))
            }
            ("created_at", "gte") | ("updated_at", "gte") | ("deleted_at", "gte") => {
                format!("{} >= '{}'", column, value.as_str().unwrap_or(""))
            }
            ("created_at", "lt") | ("updated_at", "lt") | ("deleted_at", "lt") => {
                format!("{} < '{}'", column, value.as_str().unwrap_or(""))
            }
            ("created_at", "lte") | ("updated_at", "lte") | ("deleted_at", "lte") => {
                format!("{} <= '{}'", column, value.as_str().unwrap_or(""))
            }
            ("created_at", "between") | ("updated_at", "between") | ("deleted_at", "between") => {
                if let Some(range) = value.as_array() {
                    if range.len() >= 2 {
                        format!("{} BETWEEN '{}' AND '{}'",
                               column,
                               range[0].as_str().unwrap_or(""),
                               range[1].as_str().unwrap_or(""))
                    } else {
                        format!("{} IS NOT NULL", column)
                    }
                } else {
                    format!("{} IS NOT NULL", column)
                }
            }
            (_, "is_null") => format!("{} IS NULL", column),
            (_, "is_not_null") => format!("{} IS NOT NULL", column),
            _ => format!("{} = '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
        }
    }
}

impl crate::app::query_builder::Sortable for Role {
    fn apply_basic_sort(column: &str, direction: &str) -> String {
        format!("{} {}", column, direction)
    }
}

impl crate::app::query_builder::Includable for Role {
    fn load_relationships(ids: &[String], includes: &[String], _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<()> {
        for include in includes {
            match include.as_str() {
                "permissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_permissions("sys_roles", ids, _conn)?;
                },
                "users" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_roles("sys_roles", ids, _conn)?;
                },
                "organization" => {
                    tracing::debug!("Loading organization for roles: {:?}", ids);
                },
                "createdBy" => {
                    tracing::debug!("Loading createdBy user for roles: {:?}", ids);
                },
                "updatedBy" => {
                    tracing::debug!("Loading updatedBy user for roles: {:?}", ids);
                },
                "deletedBy" => {
                    tracing::debug!("Loading deletedBy user for roles: {:?}", ids);
                },
                "createdBy.organizations" => {
                    tracing::debug!("Loading createdBy.organizations for roles: {:?}", ids);
                },
                "updatedBy.organizations" => {
                    tracing::debug!("Loading updatedBy.organizations for roles: {:?}", ids);
                },
                "deletedBy.organizations" => {
                    tracing::debug!("Loading deletedBy.organizations for roles: {:?}", ids);
                },
                "createdBy.organizations.position" => {
                    tracing::debug!("Loading createdBy.organizations.position for roles: {:?}", ids);
                },
                "updatedBy.organizations.position" => {
                    tracing::debug!("Loading updatedBy.organizations.position for roles: {:?}", ids);
                },
                "deletedBy.organizations.position" => {
                    tracing::debug!("Loading deletedBy.organizations.position for roles: {:?}", ids);
                },
                "createdBy.organizations.position.level" => {
                    tracing::debug!("Loading createdBy.organizations.position.level for roles: {:?}", ids);
                },
                "updatedBy.organizations.position.level" => {
                    tracing::debug!("Loading updatedBy.organizations.position.level for roles: {:?}", ids);
                },
                "deletedBy.organizations.position.level" => {
                    tracing::debug!("Loading deletedBy.organizations.position.level for roles: {:?}", ids);
                },
                _ => {
                    tracing::warn!("Unknown relationship: {}", include);
                }
            }
        }
        Ok(())
    }

    fn build_join_clause(relationship: &str, main_table: &str) -> Option<String> {
        match relationship {
            "permissions" => Some(format!("LEFT JOIN sys_role_has_permissions ON {}.id = sys_role_has_permissions.role_id LEFT JOIN sys_permissions ON sys_role_has_permissions.permission_id = sys_permissions.id", main_table)),
            "users" => Some(format!("LEFT JOIN sys_model_has_roles ON {}.id = sys_model_has_roles.role_id LEFT JOIN sys_users ON sys_model_has_roles.model_id = sys_users.id", main_table)),
            "organization" => Some(format!("LEFT JOIN organizations ON {}.organization_id = organizations.id", main_table)),
            "createdBy" => {
                Some(format!("LEFT JOIN sys_users AS created_by ON {}.created_by_id = created_by.id", main_table))
            },
            "updatedBy" => {
                Some(format!("LEFT JOIN sys_users AS updated_by ON {}.updated_by_id = updated_by.id", main_table))
            },
            "deletedBy" => {
                Some(format!("LEFT JOIN sys_users AS deleted_by ON {}.deleted_by_id = deleted_by.id", main_table))
            },
            _ => None
        }
    }

    fn get_foreign_key(relationship: &str) -> Option<String> {
        match relationship {
            "organization" => Some("organization_id".to_string()),
            "permissions" => Some("role_id".to_string()),
            "users" => Some("role_id".to_string()),
            "createdBy" | "updatedBy" | "deletedBy" => Some("id".to_string()),
            _ => None
        }
    }

    fn should_eager_load(relationship: &str) -> bool {
        matches!(relationship, "permissions" | "organization")
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
