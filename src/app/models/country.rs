use super::DieselUlid;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::{SortDirection};

/// Country model representing a country entity
/// Contains country information including name, ISO code, and phone code
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::schema::ref_geo_countries)]
pub struct Country {
    /// Unique identifier for the country
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    /// Country name
    #[schema(example = "United States")]
    pub name: String,
    /// ISO country code (2-3 letters)
    #[schema(example = "US")]
    pub iso_code: String,
    /// Optional phone country code
    #[schema(example = "+1")]
    pub phone_code: Option<String>,
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

/// Create country payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateCountry {
    pub name: String,
    pub iso_code: String,
    pub phone_code: Option<String>,
}

/// Insertable struct for countries
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::ref_geo_countries)]
pub struct NewCountry {
    pub id: DieselUlid,
    pub name: String,
    pub iso_code: String,
    pub phone_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by_id: DieselUlid,
    pub updated_by_id: DieselUlid,
    pub deleted_by_id: Option<DieselUlid>,
}

/// Update country payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateCountry {
    pub name: Option<String>,
    pub iso_code: Option<String>,
    pub phone_code: Option<String>,
}

/// Country response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct CountryResponse {
    pub id: DieselUlid,
    pub name: String,
    pub iso_code: String,
    pub phone_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NewCountry {
    pub fn new(name: String, iso_code: String, phone_code: Option<String>, created_by: Option<&str>) -> Self {
        let now = Utc::now();
        let system_id = created_by
            .and_then(|s| DieselUlid::from_string(s.trim()).ok())
            .unwrap_or_else(|| DieselUlid::from_string("01SYSTEM0SEEDER00000000000").unwrap());
        Self {
            id: DieselUlid::new(),
            name,
            iso_code,
            phone_code,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by_id: system_id.clone(),
            updated_by_id: system_id,
            deleted_by_id: None,
        }
    }
}

impl Country {
    pub fn new(name: String, iso_code: String, phone_code: Option<String>) -> Self {
        let now = Utc::now();
        let system_id = DieselUlid::from_string("01SYSTEM0SEEDER00000000000").unwrap();
        Self {
            id: DieselUlid::new(),
            name,
            iso_code,
            phone_code,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by_id: system_id.clone(),
            updated_by_id: system_id,
            deleted_by_id: None,
        }
    }

    pub fn to_response(&self) -> CountryResponse {
        CountryResponse {
            id: self.id,
            name: self.name.clone(),
            iso_code: self.iso_code.clone(),
            phone_code: self.phone_code.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}


impl crate::app::query_builder::Queryable for Country {
    fn table_name() -> &'static str {
        "ref_geo_countries"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "iso_code",
            "phone_code",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "iso_code",
            "phone_code",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "iso_code",
            "phone_code",
            "created_at",
            "updated_at",
        ]
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
            "provinces",
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

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("name", SortDirection::Asc))
    }
}

// Implement the enhanced filtering trait
impl crate::app::query_builder::Filterable for Country {
    fn apply_basic_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
        match (column, operator) {
            ("id", op) | ("created_by_id", op) | ("updated_by_id", op) | ("deleted_by_id", op) if op != "is_null" && op != "is_not_null" => {
                format!("{} {} '{}'", column, op, value.as_str().unwrap_or(""))
            }
            ("name", "contains") | ("iso_code", "contains") | ("phone_code", "contains") => {
                format!("LOWER({}) LIKE LOWER('%{}%')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "starts_with") | ("iso_code", "starts_with") | ("phone_code", "starts_with") => {
                format!("LOWER({}) LIKE LOWER('{}%')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "ends_with") | ("iso_code", "ends_with") | ("phone_code", "ends_with") => {
                format!("LOWER({}) LIKE LOWER('%{}')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "like") | ("iso_code", "like") | ("phone_code", "like") => {
                format!("{} LIKE '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "ilike") | ("iso_code", "ilike") | ("phone_code", "ilike") => {
                format!("{} ILIKE '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "eq") | ("iso_code", "eq") | ("phone_code", "eq") => {
                format!("{} = '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "ne") | ("iso_code", "ne") | ("phone_code", "ne") => {
                format!("{} != '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "in") | ("iso_code", "in") | ("phone_code", "in") => {
                let values = value.as_array()
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| format!("'{}'", s.replace('\'', "''")))
                        .collect::<Vec<_>>()
                        .join(","))
                    .unwrap_or_default();
                format!("{} IN ({})", column, values)
            }
            ("name", "not_in") | ("iso_code", "not_in") | ("phone_code", "not_in") => {
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
impl crate::app::query_builder::Sortable for Country {
    fn apply_basic_sort(column: &str, direction: &str) -> String {
        format!("{} {}", column, direction)
    }
}

// Implement the relationship inclusion trait
impl crate::app::query_builder::Includable for Country {
    fn load_relationships(ids: &[String], includes: &[String], _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<()> {
        for include in includes {
            match include.as_str() {
                "roles" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_roles("ref_geo_countries", ids, _conn)?;
                },
                "permissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_permissions("ref_geo_countries", ids, _conn)?;
                },
                "roles.permissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_roles_with_permissions("ref_geo_countries", ids, _conn)?;
                },
                "permissions.roles" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_permissions_with_roles("ref_geo_countries", ids, _conn)?;
                },
                "roles.organization" => {
                    crate::app::query_builder::RolePermissionLoader::load_roles_with_organization("ref_geo_countries", ids, _conn)?;
                },
                "permissions.organization" => {
                    crate::app::query_builder::RolePermissionLoader::load_permissions_with_organization("ref_geo_countries", ids, _conn)?;
                },
                "authorizationContext" => {
                    crate::app::query_builder::RolePermissionLoader::load_complete_authorization_context("ref_geo_countries", ids, _conn)?;
                },
                "scopedRoles" => {
                    crate::app::query_builder::RolePermissionLoader::load_scoped_roles("ref_geo_countries", ids, _conn)?;
                },
                "scopedPermissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_scoped_permissions("ref_geo_countries", ids, _conn)?;
                },
                "provinces" => {
                    tracing::debug!("Loading provinces for countries: {:?}", ids);
                },
                "createdBy" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_created_by_users("ref_geo_countries", ids, _conn)?;
                },
                "updatedBy" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_updated_by_users("ref_geo_countries", ids, _conn)?;
                },
                "deletedBy" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_deleted_by_users("ref_geo_countries", ids, _conn)?;
                },
                "createdBy.organizations" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_created_by_organizations("ref_geo_countries", ids, _conn)?;
                },
                "updatedBy.organizations" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_updated_by_organizations("ref_geo_countries", ids, _conn)?;
                },
                "deletedBy.organizations" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_deleted_by_organizations("ref_geo_countries", ids, _conn)?;
                },
                "createdBy.organizations.position" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_created_by_positions("ref_geo_countries", ids, _conn)?;
                },
                "updatedBy.organizations.position" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_updated_by_positions("ref_geo_countries", ids, _conn)?;
                },
                "deletedBy.organizations.position" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_deleted_by_positions("ref_geo_countries", ids, _conn)?;
                },
                "createdBy.organizations.position.level" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_created_by_levels("ref_geo_countries", ids, _conn)?;
                },
                "updatedBy.organizations.position.level" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_updated_by_levels("ref_geo_countries", ids, _conn)?;
                },
                "deletedBy.organizations.position.level" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_deleted_by_levels("ref_geo_countries", ids, _conn)?;
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
            "provinces" => Some("country_id".to_string()),
            "createdBy" | "updatedBy" | "deletedBy" => Some("id".to_string()),
            _ => None
        }
    }

    fn build_join_clause(relationship: &str, main_table: &str) -> Option<String> {
        match relationship {
            "provinces" => {
                Some(format!("LEFT JOIN ref_geo_provinces ON {}.id = ref_geo_provinces.country_id", main_table))
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
        matches!(relationship, "provinces")
    }
}

// Implement the query builder service for Country
crate::impl_query_builder_service!(Country);

impl crate::app::models::HasModelType for Country {
    fn model_type() -> &'static str {
        "Country"
    }
}

impl crate::app::models::activity_log::HasId for Country {
    fn id(&self) -> String {
        self.id.to_string()
    }
}