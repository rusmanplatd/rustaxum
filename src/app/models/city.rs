use super::DieselUlid;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use utoipa::ToSchema;
use crate::app::query_builder::{SortDirection};

/// City model representing a city within a province
/// Contains geographical coordinates and administrative information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::schema::cities)]
pub struct City {
    pub id: DieselUlid,
    pub province_id: String,
    pub name: String,
    pub code: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub latitude: Option<Decimal>,
    #[schema(value_type = Option<f64>)]
    pub longitude: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by_id: Option<DieselUlid>,
    pub updated_by_id: Option<DieselUlid>,
    pub deleted_by_id: Option<DieselUlid>,
}

/// Create city payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateCity {
    pub province_id: String,
    pub name: String,
    pub code: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub latitude: Option<Decimal>,
    #[schema(value_type = Option<f64>)]
    pub longitude: Option<Decimal>,
}

/// Insertable struct for creating new cities in the database
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::cities)]
pub struct NewCity {
    pub id: DieselUlid,
    pub province_id: String,
    pub name: String,
    pub code: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by_id: Option<DieselUlid>,
    pub updated_by_id: Option<DieselUlid>,
    pub deleted_by_id: Option<DieselUlid>,
}

/// Update city payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateCity {
    pub province_id: Option<String>,
    pub name: Option<String>,
    pub code: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub latitude: Option<Decimal>,
    #[schema(value_type = Option<f64>)]
    pub longitude: Option<Decimal>,
}

/// City response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct CityResponse {
    pub id: DieselUlid,
    pub province_id: String,
    pub name: String,
    pub code: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub latitude: Option<Decimal>,
    #[schema(value_type = Option<f64>)]
    pub longitude: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl City {
    pub fn new(
        province_id: String,
        name: String,
        code: Option<String>,
        latitude: Option<Decimal>,
        longitude: Option<Decimal>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            province_id,
            name,
            code,
            latitude,
            longitude,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by_id: None,
            updated_by_id: None,
            deleted_by_id: None,
        }
    }

    pub fn to_response(&self) -> CityResponse {
        CityResponse {
            id: self.id,
            province_id: self.province_id.clone(),
            name: self.name.clone(),
            code: self.code.clone(),
            latitude: self.latitude,
            longitude: self.longitude,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl NewCity {
    pub fn new(
        province_id: String,
        name: String,
        code: Option<String>,
        latitude: Option<Decimal>,
        longitude: Option<Decimal>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            province_id,
            name,
            code,
            latitude,
            longitude,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by_id: None,
            updated_by_id: None,
            deleted_by_id: None,
        }
    }
}


impl crate::app::query_builder::Queryable for City {
    fn table_name() -> &'static str {
        "cities"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "province_id",
            "name",
            "code",
            "latitude",
            "longitude",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "province_id",
            "name",
            "code",
            "latitude",
            "longitude",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "province_id",
            "name",
            "code",
            "latitude",
            "longitude",
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
            "province",
            "province.country",
            "districts",
            "districts.villages",
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

// Implement enhanced query builder traits for City
impl crate::app::query_builder::Filterable for City {
    fn apply_basic_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
        match (column, operator) {
            ("id", op) | ("created_by_id", op) | ("updated_by_id", op) | ("deleted_by_id", op) if op != "is_null" && op != "is_not_null" => {
                format!("{} {} '{}'", column, op, value.as_str().unwrap_or(""))
            }
            ("province_id", op) if op != "is_null" && op != "is_not_null" => {
                format!("{} {} '{}'", column, op, value.as_str().unwrap_or(""))
            }
            ("latitude", op) | ("longitude", op) => {
                if let Some(num_val) = value.as_f64() {
                    format!("{} {} {}", column, op, num_val)
                } else {
                    format!("{} = 0.0", column)
                }
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
            ("name", "in") | ("code", "in") | ("province_id", "in") => {
                let values = value.as_array()
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| format!("'{}'", s.replace('\'', "''")))
                        .collect::<Vec<_>>()
                        .join(","))
                    .unwrap_or_default();
                format!("{} IN ({})", column, values)
            }
            ("name", "not_in") | ("code", "not_in") | ("province_id", "not_in") => {
                let values = value.as_array()
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| format!("'{}'", s.replace('\'', "''")))
                        .collect::<Vec<_>>()
                        .join(","))
                    .unwrap_or_default();
                format!("{} NOT IN ({})", column, values)
            }
            ("latitude", "between") | ("longitude", "between") => {
                if let Some(range) = value.as_array() {
                    if range.len() >= 2 {
                        format!("{} BETWEEN {} AND {}",
                               column,
                               range[0].as_f64().unwrap_or(0.0),
                               range[1].as_f64().unwrap_or(0.0))
                    } else {
                        format!("{} IS NOT NULL", column)
                    }
                } else {
                    format!("{} IS NOT NULL", column)
                }
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

impl crate::app::query_builder::Sortable for City {
    fn apply_basic_sort(column: &str, direction: &str) -> String {
        format!("{} {}", column, direction)
    }
}

impl crate::app::query_builder::Includable for City {
    fn load_relationships(ids: &[String], includes: &[String], _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<()> {
        for include in includes {
            match include.as_str() {
                "roles" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_roles("cities", ids, _conn)?;
                },
                "permissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_permissions("cities", ids, _conn)?;
                },
                "roles.permissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_roles_with_permissions("cities", ids, _conn)?;
                },
                "permissions.roles" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_permissions_with_roles("cities", ids, _conn)?;
                },
                "roles.organization" => {
                    crate::app::query_builder::RolePermissionLoader::load_roles_with_organization("cities", ids, _conn)?;
                },
                "permissions.organization" => {
                    crate::app::query_builder::RolePermissionLoader::load_permissions_with_organization("cities", ids, _conn)?;
                },
                "authorizationContext" => {
                    crate::app::query_builder::RolePermissionLoader::load_complete_authorization_context("cities", ids, _conn)?;
                },
                "scopedRoles" => {
                    crate::app::query_builder::RolePermissionLoader::load_scoped_roles("cities", ids, _conn)?;
                },
                "scopedPermissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_scoped_permissions("cities", ids, _conn)?;
                },
                "province" => {
                    tracing::debug!("Loading province for cities: {:?}", ids);
                },
                "province.country" => {
                    tracing::debug!("Loading province.country for cities: {:?}", ids);
                },
                "districts" => {
                    tracing::debug!("Loading districts for cities: {:?}", ids);
                },
                "districts.villages" => {
                    tracing::debug!("Loading districts.villages for cities: {:?}", ids);
                },
                "createdBy" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_created_by_users("cities", ids, _conn)?;
                },
                "updatedBy" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_updated_by_users("cities", ids, _conn)?;
                },
                "deletedBy" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_deleted_by_users("cities", ids, _conn)?;
                },
                "createdBy.organizations" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_created_by_organizations("cities", ids, _conn)?;
                },
                "updatedBy.organizations" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_updated_by_organizations("cities", ids, _conn)?;
                },
                "deletedBy.organizations" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_deleted_by_organizations("cities", ids, _conn)?;
                },
                "createdBy.organizations.position" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_created_by_positions("cities", ids, _conn)?;
                },
                "updatedBy.organizations.position" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_updated_by_positions("cities", ids, _conn)?;
                },
                "deletedBy.organizations.position" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_deleted_by_positions("cities", ids, _conn)?;
                },
                "createdBy.organizations.position.level" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_created_by_levels("cities", ids, _conn)?;
                },
                "updatedBy.organizations.position.level" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_updated_by_levels("cities", ids, _conn)?;
                },
                "deletedBy.organizations.position.level" => {
                    crate::app::query_builder::AuditRelationshipLoader::load_deleted_by_levels("cities", ids, _conn)?;
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
            "province" => Some(format!("LEFT JOIN provinces ON {}.province_id = provinces.id", main_table)),
            "province.country" => Some(format!("LEFT JOIN provinces ON {}.province_id = provinces.id LEFT JOIN countries ON provinces.country_id = countries.id", main_table)),
            "districts" => Some(format!("LEFT JOIN districts ON {}.id = districts.city_id", main_table)),
            "districts.villages" => Some(format!("LEFT JOIN districts ON {}.id = districts.city_id LEFT JOIN villages ON districts.id = villages.district_id", main_table)),
            "createdBy" => {
                Some(format!("LEFT JOIN sys_users AS created_by ON {}.created_by_id = created_by.id", main_table))
            },
            "updatedBy" => {
                Some(format!("LEFT JOIN sys_users AS updated_by ON {}.updated_by_id = updated_by.id", main_table))
            },
            "deletedBy" => {
                Some(format!("LEFT JOIN sys_users AS deleted_by ON {}.deleted_by_id = deleted_by.id", main_table))
            },
            "createdBy.organizations" => {
                Some(format!("LEFT JOIN sys_users AS created_by ON {}.created_by_id = created_by.id LEFT JOIN user_organizations AS created_by_orgs ON created_by.id = created_by_orgs.user_id", main_table))
            },
            "updatedBy.organizations" => {
                Some(format!("LEFT JOIN sys_users AS updated_by ON {}.updated_by_id = updated_by.id LEFT JOIN user_organizations AS updated_by_orgs ON updated_by.id = updated_by_orgs.user_id", main_table))
            },
            "deletedBy.organizations" => {
                Some(format!("LEFT JOIN sys_users AS deleted_by ON {}.deleted_by_id = deleted_by.id LEFT JOIN user_organizations AS deleted_by_orgs ON deleted_by.id = deleted_by_orgs.user_id", main_table))
            },
            "createdBy.organizations.position" => {
                Some(format!("LEFT JOIN sys_users AS created_by ON {}.created_by_id = created_by.id LEFT JOIN user_organizations AS created_by_orgs ON created_by.id = created_by_orgs.user_id LEFT JOIN organization_positions AS created_by_pos ON created_by_orgs.organization_position_id = created_by_pos.id", main_table))
            },
            "updatedBy.organizations.position" => {
                Some(format!("LEFT JOIN sys_users AS updated_by ON {}.updated_by_id = updated_by.id LEFT JOIN user_organizations AS updated_by_orgs ON updated_by.id = updated_by_orgs.user_id LEFT JOIN organization_positions AS updated_by_pos ON updated_by_orgs.organization_position_id = updated_by_pos.id", main_table))
            },
            "deletedBy.organizations.position" => {
                Some(format!("LEFT JOIN sys_users AS deleted_by ON {}.deleted_by_id = deleted_by.id LEFT JOIN user_organizations AS deleted_by_orgs ON deleted_by.id = deleted_by_orgs.user_id LEFT JOIN organization_positions AS deleted_by_pos ON deleted_by_orgs.organization_position_id = deleted_by_pos.id", main_table))
            },
            "createdBy.organizations.position.level" => {
                Some(format!("LEFT JOIN sys_users AS created_by ON {}.created_by_id = created_by.id LEFT JOIN user_organizations AS created_by_orgs ON created_by.id = created_by_orgs.user_id LEFT JOIN organization_positions AS created_by_pos ON created_by_orgs.organization_position_id = created_by_pos.id LEFT JOIN organization_position_levels AS created_by_level ON created_by_pos.organization_position_level_id = created_by_level.id", main_table))
            },
            "updatedBy.organizations.position.level" => {
                Some(format!("LEFT JOIN sys_users AS updated_by ON {}.updated_by_id = updated_by.id LEFT JOIN user_organizations AS updated_by_orgs ON updated_by.id = updated_by_orgs.user_id LEFT JOIN organization_positions AS updated_by_pos ON updated_by_orgs.organization_position_id = updated_by_pos.id LEFT JOIN organization_position_levels AS updated_by_level ON updated_by_pos.organization_position_level_id = updated_by_level.id", main_table))
            },
            "deletedBy.organizations.position.level" => {
                Some(format!("LEFT JOIN sys_users AS deleted_by ON {}.deleted_by_id = deleted_by.id LEFT JOIN user_organizations AS deleted_by_orgs ON deleted_by.id = deleted_by_orgs.user_id LEFT JOIN organization_positions AS deleted_by_pos ON deleted_by_orgs.organization_position_id = deleted_by_pos.id LEFT JOIN organization_position_levels AS deleted_by_level ON deleted_by_pos.organization_position_level_id = deleted_by_level.id", main_table))
            },
            _ => None
        }
    }

    fn get_foreign_key(relationship: &str) -> Option<String> {
        match relationship {
            "province" => Some("province_id".to_string()),
            "districts" => Some("city_id".to_string()),
            "createdBy" | "updatedBy" | "deletedBy" => Some("id".to_string()),
            _ => None
        }
    }

    fn should_eager_load(relationship: &str) -> bool {
        matches!(relationship, "province" | "districts")
    }
}

// Implement the query builder service for City
crate::impl_query_builder_service!(City);

impl crate::app::models::HasModelType for City {
    fn model_type() -> &'static str {
        "City"
    }
}

impl crate::app::models::activity_log::HasId for City {
    fn id(&self) -> String {
        self.id.to_string()
    }
}