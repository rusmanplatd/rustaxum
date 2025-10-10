use super::DieselUlid;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::{SortDirection};

/// Province model representing a state/province within a country
/// Contains geographical and administrative information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::ref_geo_provinces)]
pub struct Province {
    /// Unique province identifier
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    /// ID of the country this province belongs to
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub country_id: String,
    /// Province name
    #[schema(example = "California")]
    pub name: String,
    /// Optional province/state code
    #[schema(example = "CA")]
    pub code: Option<String>,
    /// Creation timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
    /// Soft delete timestamp
    pub deleted_at: Option<DateTime<Utc>>,
    /// User who created this record
    pub created_by_id: DieselUlid,
    /// User who last updated this record
    pub updated_by_id: DieselUlid,
    /// User who deleted this record
    pub deleted_by_id: Option<DieselUlid>,
}

/// Create province payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateProvince {
    pub country_id: String,
    pub name: String,
    pub code: Option<String>,
}

/// Update province payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateProvince {
    pub country_id: Option<String>,
    pub name: Option<String>,
    pub code: Option<String>,
}

/// Province response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct ProvinceResponse {
    pub id: DieselUlid,
    pub country_id: String,
    pub name: String,
    pub code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Province {
    pub fn new(country_id: String, name: String, code: Option<String>, created_by: &str) -> Self {
        let now = Utc::now();
        let creator_id = DieselUlid::from_string(created_by.trim()).expect("Invalid created_by ULID provided to Province::new()");
        Province {
            id: DieselUlid::new(),
            country_id,
            name,
            code,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by_id: creator_id.clone(),
            updated_by_id: creator_id,
            deleted_by_id: None,
        }
    }

    pub fn to_response(&self) -> ProvinceResponse {
        ProvinceResponse {
            id: self.id,
            country_id: self.country_id.clone(),
            name: self.name.clone(),
            code: self.code.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl crate::app::query_builder::Queryable for Province {
    fn table_name() -> &'static str {
        "ref_geo_provinces"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "country_id",
            "name",
            "code",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "country_id",
            "name",
            "code",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "country_id",
            "name",
            "code",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("name", SortDirection::Asc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "roles",
            "permissions",
            "roles.permissions",
            "permissions.roles",
            "roles.organization",
            "permissions.organization",
            "authorizationContext",
            "scopedRoles",
            "scopedPermissions",
            "country",
            "cities",
            "cities.districts",
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

// Implement the enhanced filtering trait
impl crate::app::query_builder::Filterable for Province {
    fn apply_basic_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
        match (column, operator) {
            ("id", op) | ("created_by_id", op) | ("updated_by_id", op) | ("deleted_by_id", op) if op != "is_null" && op != "is_not_null" => {
                format!("{} {} '{}'", column, op, value.as_str().unwrap_or(""))
            }
            ("country_id", op) if op != "is_null" && op != "is_not_null" => {
                format!("{} {} '{}'", column, op, value.as_str().unwrap_or(""))
            }
            ("name", "contains") | ("code", "contains") => {
                format!("LOWER({}) LIKE LOWER('%{}%')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "starts_with") | ("code", "starts_with") => {
                format!("LOWER({}) LIKE LOWER('{}%')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "ends_with") | ("code", "ends_with") => {
                format!("LOWER({}) LIKE LOWER('%{}')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "like") | ("code", "like") => {
                format!("{} LIKE '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "ilike") | ("code", "ilike") => {
                format!("{} ILIKE '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "eq") | ("code", "eq") => {
                format!("{} = '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "ne") | ("code", "ne") => {
                format!("{} != '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "in") | ("code", "in") | ("country_id", "in") => {
                let values = value.as_array()
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| format!("'{}'", s.replace('\'', "''")))
                        .collect::<Vec<_>>()
                        .join(","))
                    .unwrap_or_default();
                format!("{} IN ({})", column, values)
            }
            ("name", "not_in") | ("code", "not_in") | ("country_id", "not_in") => {
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

// Implement the enhanced sorting trait
impl crate::app::query_builder::Sortable for Province {
    fn apply_basic_sort(column: &str, direction: &str) -> String {
        format!("{} {}", column, direction)
    }
}

// Implement the relationship inclusion trait
impl crate::app::query_builder::Includable for Province {
    fn load_relationships(ids: &[String], includes: &[String], _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<()> {
        for include in includes {
            match include.as_str() {
                "roles" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_roles("ref_geo_provinces", ids, _conn)?;
                },
                "permissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_permissions("ref_geo_provinces", ids, _conn)?;
                },
                "roles.permissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_roles_with_permissions("ref_geo_provinces", ids, _conn)?;
                },
                "permissions.roles" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_permissions_with_roles("ref_geo_provinces", ids, _conn)?;
                },
                "roles.organization" => {
                    crate::app::query_builder::RolePermissionLoader::load_roles_with_organization("ref_geo_provinces", ids, _conn)?;
                },
                "permissions.organization" => {
                    crate::app::query_builder::RolePermissionLoader::load_permissions_with_organization("ref_geo_provinces", ids, _conn)?;
                },
                "authorizationContext" => {
                    crate::app::query_builder::RolePermissionLoader::load_complete_authorization_context("ref_geo_provinces", ids, _conn)?;
                },
                "scopedRoles" => {
                    crate::app::query_builder::RolePermissionLoader::load_scoped_roles("ref_geo_provinces", ids, _conn)?;
                },
                "scopedPermissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_scoped_permissions("ref_geo_provinces", ids, _conn)?;
                },
                "country" => {
                    tracing::debug!("Loading country for provinces: {:?}", ids);
                },
                "cities" => {
                    tracing::debug!("Loading cities for provinces: {:?}", ids);
                },
                "cities.districts" => {
                    tracing::debug!("Loading cities.districts for provinces: {:?}", ids);
                },
                "createdBy" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_created_by_users("ref_geo_provinces", ids, _conn)?;
                },
                "updatedBy" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_updated_by_users("ref_geo_provinces", ids, _conn)?;
                },
                "deletedBy" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_deleted_by_users("ref_geo_provinces", ids, _conn)?;
                },
                "createdBy.organizations" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_created_by_organizations("ref_geo_provinces", ids, _conn)?;
                },
                "updatedBy.organizations" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_updated_by_organizations("ref_geo_provinces", ids, _conn)?;
                },
                "deletedBy.organizations" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_deleted_by_organizations("ref_geo_provinces", ids, _conn)?;
                },
                "createdBy.organizations.position" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_created_by_positions("ref_geo_provinces", ids, _conn)?;
                },
                "updatedBy.organizations.position" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_updated_by_positions("ref_geo_provinces", ids, _conn)?;
                },
                "deletedBy.organizations.position" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_deleted_by_positions("ref_geo_provinces", ids, _conn)?;
                },
                "createdBy.organizations.position.level" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_created_by_levels("ref_geo_provinces", ids, _conn)?;
                },
                "updatedBy.organizations.position.level" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_updated_by_levels("ref_geo_provinces", ids, _conn)?;
                },
                "deletedBy.organizations.position.level" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_deleted_by_levels("ref_geo_provinces", ids, _conn)?;
                },
                _ => {
                    tracing::warn!("Unknown relationship: {}", include);
                }
            }
        }
        Ok(())
    }

    fn get_foreign_key(relationship: &str) -> Option<String> {
        match relationship {
            "country" => Some("country_id".to_string()),
            "cities" => Some("province_id".to_string()),
            "createdBy" | "updatedBy" | "deletedBy" => Some("id".to_string()),
            _ => None
        }
    }

    fn build_join_clause(relationship: &str, main_table: &str) -> Option<String> {
        match relationship {
            "country" => {
                Some(format!("LEFT JOIN ref_geo_countries ON {}.country_id = ref_geo_countries.id", main_table))
            },
            "cities" => {
                Some(format!("LEFT JOIN ref_geo_cities ON {}.id = ref_geo_cities.province_id", main_table))
            },
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

    fn should_eager_load(relationship: &str) -> bool {
        matches!(relationship, "country" | "cities")
    }
}

// Implement the query builder service for Province
crate::impl_query_builder_service!(Province);

impl crate::app::models::HasModelType for Province {
    fn model_type() -> &'static str {
        "Province"
    }
}

impl crate::app::models::activity_log::HasId for Province {
    fn id(&self) -> String {
        self.id.to_string()
    }
}