use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::sys_permissions)]
pub struct Permission {
    pub id: DieselUlid,
    pub organization_id: Option<DieselUlid>,
    pub guard_name: String,
    pub resource: Option<String>,
    pub action: String,
    pub scope_type: Option<String>,
    pub scope_id: Option<DieselUlid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by_id: DieselUlid,
    pub updated_by_id: DieselUlid,
    pub deleted_by_id: Option<DieselUlid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatePermission {
    pub guard_name: Option<String>,
    pub resource: Option<String>,
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdatePermission {
    pub guard_name: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionResponse {
    pub id: DieselUlid,
    pub guard_name: String,
    pub resource: Option<String>,
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Permission {
    pub fn new(guard_name: Option<String>, resource: Option<String>, action: String, user_id: Option<DieselUlid>) -> Self {
        let now = Utc::now();
        let created_by = user_id.unwrap_or_else(|| DieselUlid::from_string("01SYSTEM000000000000000000").unwrap_or_else(|_| DieselUlid::new()));

        Self {
            id: DieselUlid::new(),
            organization_id: None,
            guard_name: guard_name.unwrap_or_else(|| "api".to_string()),
            resource,
            action,
            scope_type: None,
            scope_id: None,
            created_at: now,
            updated_at: now,
            created_by_id: created_by,
            updated_by_id: created_by,
            deleted_by_id: None,
        }
    }

    pub fn update_with_user(&mut self, user_id: Option<DieselUlid>) {
        self.updated_at = Utc::now();
        self.updated_by_id = user_id.unwrap_or_else(|| DieselUlid::from_string("01SYSTEM000000000000000000").unwrap());
    }

    pub fn soft_delete(&mut self, user_id: Option<DieselUlid>) {
                self.updated_at = Utc::now();
        self.deleted_by_id = user_id;
        self.updated_by_id = user_id.unwrap_or_else(|| DieselUlid::from_string("01SYSTEM000000000000000000").unwrap());
    }

    pub fn to_response(&self) -> PermissionResponse {
        PermissionResponse {
            id: self.id,
            guard_name: self.guard_name.clone(),
            resource: self.resource.clone(),
            action: self.action.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl crate::app::query_builder::Queryable for Permission {
    fn table_name() -> &'static str {
        "sys_permissions"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "guard_name",
            "resource",
            "action",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "guard_name",
            "resource",
            "action",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "guard_name",
            "resource",
            "action",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("action", SortDirection::Asc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "roles",
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

// Implement enhanced query builder traits for Permission
impl crate::app::query_builder::Filterable for Permission {
    fn apply_basic_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
        match (column, operator) {
            ("id", op) | ("organization_id", op) | ("scope_id", op) | ("created_by_id", op) | ("updated_by_id", op) | ("deleted_by_id", op) if op != "is_null" && op != "is_not_null" => {
                format!("{} {} '{}'", column, op, value.as_str().unwrap_or(""))
            }
            ("action", "contains") | ("resource", "contains") | ("guard_name", "contains") | ("scope_type", "contains") => {
                format!("LOWER({}) LIKE LOWER('%{}%')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("action", "starts_with") | ("resource", "starts_with") | ("guard_name", "starts_with") | ("scope_type", "starts_with") => {
                format!("LOWER({}) LIKE LOWER('{}%')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("action", "ends_with") | ("resource", "ends_with") | ("guard_name", "ends_with") | ("scope_type", "ends_with") => {
                format!("LOWER({}) LIKE LOWER('%{}')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("action", "like") | ("resource", "like") | ("guard_name", "like") | ("scope_type", "like") => {
                format!("{} LIKE '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("action", "ilike") | ("resource", "ilike") | ("guard_name", "ilike") | ("scope_type", "ilike") => {
                format!("{} ILIKE '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("action", "eq") | ("resource", "eq") | ("guard_name", "eq") | ("scope_type", "eq") => {
                format!("{} = '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("action", "ne") | ("resource", "ne") | ("guard_name", "ne") | ("scope_type", "ne") => {
                format!("{} != '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("action", "in") | ("resource", "in") | ("guard_name", "in") | ("scope_type", "in") | ("organization_id", "in") => {
                let values = value.as_array()
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| format!("'{}'", s.replace('\'', "''")))
                        .collect::<Vec<_>>()
                        .join(","))
                    .unwrap_or_default();
                format!("{} IN ({})", column, values)
            }
            ("action", "not_in") | ("resource", "not_in") | ("guard_name", "not_in") | ("scope_type", "not_in") | ("organization_id", "not_in") => {
                let values = value.as_array()
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| format!("'{}'", s.replace('\'', "''")))
                        .collect::<Vec<_>>()
                        .join(","))
                    .unwrap_or_default();
                format!("{} NOT IN ({})", column, values)
            }
            ("created_at", "gt") | ("updated_at", "gt") => {
                format!("{} > '{}'", column, value.as_str().unwrap_or(""))
            }
            ("created_at", "gte") | ("updated_at", "gte") => {
                format!("{} >= '{}'", column, value.as_str().unwrap_or(""))
            }
            ("created_at", "lt") | ("updated_at", "lt") => {
                format!("{} < '{}'", column, value.as_str().unwrap_or(""))
            }
            ("created_at", "lte") | ("updated_at", "lte") => {
                format!("{} <= '{}'", column, value.as_str().unwrap_or(""))
            }
            ("created_at", "between") | ("updated_at", "between") => {
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

impl crate::app::query_builder::Sortable for Permission {
    fn apply_basic_sort(column: &str, direction: &str) -> String {
        format!("{} {}", column, direction)
    }
}

impl crate::app::query_builder::Includable for Permission {
    fn load_relationships(ids: &[String], includes: &[String], _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<()> {
        for include in includes {
            match include.as_str() {
                "roles" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_roles("sys_permissions", ids, _conn)?;
                },
                "organization" => {
                    tracing::debug!("Loading organization for permissions: {:?}", ids);
                },
                "createdBy" => {
                    tracing::debug!("Loading createdBy user for permissions: {:?}", ids);
                },
                "updatedBy" => {
                    tracing::debug!("Loading updatedBy user for permissions: {:?}", ids);
                },
                "deletedBy" => {
                    tracing::debug!("Loading deletedBy user for permissions: {:?}", ids);
                },
                "createdBy.organizations" => {
                    tracing::debug!("Loading createdBy.organizations for permissions: {:?}", ids);
                },
                "updatedBy.organizations" => {
                    tracing::debug!("Loading updatedBy.organizations for permissions: {:?}", ids);
                },
                "deletedBy.organizations" => {
                    tracing::debug!("Loading deletedBy.organizations for permissions: {:?}", ids);
                },
                "createdBy.organizations.position" => {
                    tracing::debug!("Loading createdBy.organizations.position for permissions: {:?}", ids);
                },
                "updatedBy.organizations.position" => {
                    tracing::debug!("Loading updatedBy.organizations.position for permissions: {:?}", ids);
                },
                "deletedBy.organizations.position" => {
                    tracing::debug!("Loading deletedBy.organizations.position for permissions: {:?}", ids);
                },
                "createdBy.organizations.position.level" => {
                    tracing::debug!("Loading createdBy.organizations.position.level for permissions: {:?}", ids);
                },
                "updatedBy.organizations.position.level" => {
                    tracing::debug!("Loading updatedBy.organizations.position.level for permissions: {:?}", ids);
                },
                "deletedBy.organizations.position.level" => {
                    tracing::debug!("Loading deletedBy.organizations.position.level for permissions: {:?}", ids);
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
            "roles" => Some(format!("LEFT JOIN sys_role_has_permissions ON {}.id = sys_role_has_permissions.permission_id LEFT JOIN sys_roles ON sys_role_has_permissions.role_id = sys_roles.id", main_table)),
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
            "roles" => Some("permission_id".to_string()),
            "createdBy" | "updatedBy" | "deletedBy" => Some("id".to_string()),
            _ => None
        }
    }

    fn should_eager_load(relationship: &str) -> bool {
        matches!(relationship, "roles" | "organization")
    }
}

// Implement the query builder service for Permission
crate::impl_query_builder_service!(Permission);

impl crate::app::models::HasModelType for Permission {
    fn model_type() -> &'static str {
        "Permission"
    }
}

impl crate::app::models::activity_log::HasId for Permission {
    fn id(&self) -> String {
        self.id.to_string()
    }
}
