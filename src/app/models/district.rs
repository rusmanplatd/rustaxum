use super::DieselUlid;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::{SortDirection};

/// District model representing a district within a city
/// Contains administrative information for sub-city divisions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::ref_geo_districts)]
pub struct District {
    /// Unique district identifier
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    /// ID of the city this district belongs to
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub city_id: String,
    /// District name
    #[schema(example = "Downtown")]
    pub name: String,
    /// Optional district code
    #[schema(example = "DT")]
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

/// Create district payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateDistrict {
    pub city_id: String,
    pub name: String,
    pub code: Option<String>,
}

/// Update district payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateDistrict {
    pub city_id: Option<String>,
    pub name: Option<String>,
    pub code: Option<String>,
}

/// District response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct DistrictResponse {
    pub id: DieselUlid,
    pub city_id: String,
    pub name: String,
    pub code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl District {
    pub fn new(city_id: String, name: String, code: Option<String>, created_by: &str) -> Self {
        let now = Utc::now();
        let creator_id = DieselUlid::from_string(created_by.trim()).expect("Invalid created_by ULID provided to District::new()");
        District {
            id: DieselUlid::new(),
            city_id,
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

    pub fn to_response(&self) -> DistrictResponse {
        DistrictResponse {
            id: self.id,
            city_id: self.city_id.clone(),
            name: self.name.clone(),
            code: self.code.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl crate::app::query_builder::Queryable for District {
    fn table_name() -> &'static str {
        "ref_geo_districts"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "city_id",
            "name",
            "code",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "city_id",
            "name",
            "code",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "city_id",
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
            "city",
            "city.province",
            "city.province.country",
            "villages",
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

// Implement enhanced query builder traits for District
impl crate::app::query_builder::Filterable for District {
    fn apply_basic_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
        match (column, operator) {
            ("id", op) | ("created_by_id", op) | ("updated_by_id", op) | ("deleted_by_id", op) if op != "is_null" && op != "is_not_null" => {
                format!("{} {} '{}'", column, op, value.as_str().unwrap_or(""))
            }
            ("city_id", op) if op != "is_null" && op != "is_not_null" => {
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
            ("name", "in") | ("code", "in") | ("city_id", "in") => {
                let values = value.as_array()
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| format!("'{}'", s.replace('\'', "''")))
                        .collect::<Vec<_>>()
                        .join(","))
                    .unwrap_or_default();
                format!("{} IN ({})", column, values)
            }
            ("name", "not_in") | ("code", "not_in") | ("city_id", "not_in") => {
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

impl crate::app::query_builder::Sortable for District {
    fn apply_basic_sort(column: &str, direction: &str) -> String {
        format!("{} {}", column, direction)
    }
}

impl crate::app::query_builder::Includable for District {
    fn load_relationships(ids: &[String], includes: &[String], _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<()> {
        for include in includes {
            match include.as_str() {
                "roles" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_roles("ref_geo_districts", ids, _conn)?;
                },
                "permissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_permissions("ref_geo_districts", ids, _conn)?;
                },
                "roles.permissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_roles_with_permissions("ref_geo_districts", ids, _conn)?;
                },
                "permissions.roles" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_permissions_with_roles("ref_geo_districts", ids, _conn)?;
                },
                "roles.organization" => {
                    crate::app::query_builder::RolePermissionLoader::load_roles_with_organization("ref_geo_districts", ids, _conn)?;
                },
                "permissions.organization" => {
                    crate::app::query_builder::RolePermissionLoader::load_permissions_with_organization("ref_geo_districts", ids, _conn)?;
                },
                "authorizationContext" => {
                    crate::app::query_builder::RolePermissionLoader::load_complete_authorization_context("ref_geo_districts", ids, _conn)?;
                },
                "scopedRoles" => {
                    crate::app::query_builder::RolePermissionLoader::load_scoped_roles("ref_geo_districts", ids, _conn)?;
                },
                "scopedPermissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_scoped_permissions("ref_geo_districts", ids, _conn)?;
                },
                "city" => {
                    tracing::debug!("Loading city for districts: {:?}", ids);
                },
                "city.province" => {
                    tracing::debug!("Loading city.province for districts: {:?}", ids);
                },
                "city.province.country" => {
                    tracing::debug!("Loading city.province.country for districts: {:?}", ids);
                },
                "villages" => {
                    tracing::debug!("Loading villages for districts: {:?}", ids);
                },
                "createdBy" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_created_by_users("ref_geo_districts", ids, _conn)?;
                },
                "updatedBy" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_updated_by_users("ref_geo_districts", ids, _conn)?;
                },
                "deletedBy" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_deleted_by_users("ref_geo_districts", ids, _conn)?;
                },
                "createdBy.organizations" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_created_by_organizations("ref_geo_districts", ids, _conn)?;
                },
                "updatedBy.organizations" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_updated_by_organizations("ref_geo_districts", ids, _conn)?;
                },
                "deletedBy.organizations" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_deleted_by_organizations("ref_geo_districts", ids, _conn)?;
                },
                "createdBy.organizations.position" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_created_by_positions("ref_geo_districts", ids, _conn)?;
                },
                "updatedBy.organizations.position" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_updated_by_positions("ref_geo_districts", ids, _conn)?;
                },
                "deletedBy.organizations.position" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_deleted_by_positions("ref_geo_districts", ids, _conn)?;
                },
                "createdBy.organizations.position.level" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_created_by_levels("ref_geo_districts", ids, _conn)?;
                },
                "updatedBy.organizations.position.level" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_updated_by_levels("ref_geo_districts", ids, _conn)?;
                },
                "deletedBy.organizations.position.level" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_deleted_by_levels("ref_geo_districts", ids, _conn)?;
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
            "city" => Some(format!("LEFT JOIN ref_geo_cities ON {}.city_id = ref_geo_cities.id", main_table)),
            "city.province" => Some(format!("LEFT JOIN ref_geo_cities ON {}.city_id = ref_geo_cities.id LEFT JOIN ref_geo_provinces ON ref_geo_cities.province_id = ref_geo_provinces.id", main_table)),
            "city.province.country" => Some(format!("LEFT JOIN ref_geo_cities ON {}.city_id = ref_geo_cities.id LEFT JOIN ref_geo_provinces ON ref_geo_cities.province_id = ref_geo_provinces.id LEFT JOIN ref_geo_countries ON ref_geo_provinces.country_id = ref_geo_countries.id", main_table)),
            "villages" => Some(format!("LEFT JOIN ref_geo_villages ON {}.id = ref_geo_villages.district_id", main_table)),
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
            "city" => Some("city_id".to_string()),
            "villages" => Some("district_id".to_string()),
            "createdBy" | "updatedBy" | "deletedBy" => Some("id".to_string()),
            _ => None
        }
    }

    fn should_eager_load(relationship: &str) -> bool {
        matches!(relationship, "city" | "villages")
    }
}

// Implement the query builder service for District
crate::impl_query_builder_service!(District);

impl crate::app::models::HasModelType for District {
    fn model_type() -> &'static str {
        "District"
    }
}

impl crate::app::models::activity_log::HasId for District {
    fn id(&self) -> String {
        self.id.to_string()
    }
}