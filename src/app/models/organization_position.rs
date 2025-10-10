use super::{DieselUlid, DecimalWrapper};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::{SortDirection};
use serde_json::Value as JsonValue;

/// Organization position model representing specific sys_roles within organization position levels
/// Contains position information and relationship to organization position level hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::schema::organization_positions)]
pub struct OrganizationPosition {
    /// Unique identifier for the organization position
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    /// Organization ID this position belongs to
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_id: DieselUlid,
    /// ID of the organization position level this position belongs to
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_position_level_id: DieselUlid,
    /// Position code
    #[schema(example = "SEM")]
    pub code: String,
    /// Organization position name
    #[schema(example = "Software Engineering Manager")]
    pub name: String,
    /// Optional description of the organization position
    #[schema(example = "Manages software engineering teams and technical projects")]
    pub description: Option<String>,
    /// Whether the organization position is currently active
    #[schema(example = true)]
    pub is_active: bool,
    /// Minimum salary for this position
    pub min_salary: DecimalWrapper,
    /// Maximum salary for this position
    pub max_salary: DecimalWrapper,
    /// Maximum number of incumbents allowed for this position
    pub max_incumbents: i32,
    /// Required qualifications (JSON array)
    pub qualifications: JsonValue,
    /// Position responsibilities (JSON array)
    pub responsibilities: JsonValue,
    /// Creation timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
    /// Soft delete timestamp
    pub deleted_at: Option<DateTime<Utc>>,
    /// User who created this position
    pub created_by_id: DieselUlid,
    /// User who last updated this position
    pub updated_by_id: DieselUlid,
    /// User who deleted this position
    pub deleted_by_id: Option<DieselUlid>,
}

/// Create organization position payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateOrganizationPosition {
    pub organization_id: DieselUlid,
    pub organization_position_level_id: DieselUlid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub min_salary: Option<DecimalWrapper>,
    pub max_salary: Option<DecimalWrapper>,
    pub max_incumbents: Option<i32>,
    pub qualifications: Option<JsonValue>,
    pub responsibilities: Option<JsonValue>,
}

/// Insertable struct for organization positions
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::organization_positions)]
pub struct NewOrganizationPosition {
    pub id: DieselUlid,
    pub organization_id: DieselUlid,
    pub organization_position_level_id: DieselUlid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub min_salary: DecimalWrapper,
    pub max_salary: DecimalWrapper,
    pub max_incumbents: i32,
    pub qualifications: JsonValue,
    pub responsibilities: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by_id: DieselUlid,
    pub updated_by_id: DieselUlid,
    pub deleted_by_id: Option<DieselUlid>,
}

/// Update organization position payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrganizationPosition {
    pub organization_id: Option<DieselUlid>,
    pub organization_position_level_id: Option<DieselUlid>,
    pub code: Option<String>,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub is_active: Option<bool>,
    pub min_salary: Option<DecimalWrapper>,
    pub max_salary: Option<DecimalWrapper>,
    pub max_incumbents: Option<i32>,
    pub qualifications: Option<JsonValue>,
    pub responsibilities: Option<JsonValue>,
}

/// Organization position response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationPositionResponse {
    pub id: DieselUlid,
    pub organization_id: DieselUlid,
    pub organization_position_level_id: DieselUlid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub min_salary: DecimalWrapper,
    pub max_salary: DecimalWrapper,
    pub max_incumbents: i32,
    pub qualifications: JsonValue,
    pub responsibilities: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by_id: DieselUlid,
    pub updated_by_id: DieselUlid,
    pub deleted_by_id: Option<DieselUlid>,
}

impl NewOrganizationPosition {
    pub fn new(create_position: CreateOrganizationPosition, created_by: Option<DieselUlid>) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            organization_id: create_position.organization_id,
            organization_position_level_id: create_position.organization_position_level_id,
            code: create_position.code,
            name: create_position.name,
            description: create_position.description,
            is_active: true,
            min_salary: create_position.min_salary.unwrap_or_else(|| DecimalWrapper::from(0)),
            max_salary: create_position.max_salary.unwrap_or_else(|| DecimalWrapper::from(0)),
            max_incumbents: create_position.max_incumbents.unwrap_or(1),
            qualifications: create_position.qualifications.unwrap_or_else(|| serde_json::json!([])),
            responsibilities: create_position.responsibilities.unwrap_or_else(|| serde_json::json!([])),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by_id: created_by.unwrap_or_else(|| DieselUlid::from_string("01SYSTEM0SEEDER00000000000").unwrap()),
            updated_by_id: created_by.unwrap_or_else(|| DieselUlid::from_string("01SYSTEM0SEEDER00000000000").unwrap()),
            deleted_by_id: None,
        }
    }
}

impl OrganizationPosition {
    pub fn to_response(&self) -> OrganizationPositionResponse {
        OrganizationPositionResponse {
            id: self.id,
            organization_id: self.organization_id,
            organization_position_level_id: self.organization_position_level_id,
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            is_active: self.is_active,
            min_salary: self.min_salary.clone(),
            max_salary: self.max_salary.clone(),
            max_incumbents: self.max_incumbents,
            qualifications: self.qualifications.clone(),
            responsibilities: self.responsibilities.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
            created_by_id: self.created_by_id,
            updated_by_id: self.updated_by_id,
            deleted_by_id: self.deleted_by_id,
        }
    }
}

impl crate::app::query_builder::Queryable for OrganizationPosition {
    fn table_name() -> &'static str {
        "organization_positions"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "organization_id",
            "organization_position_level_id",
            "code",
            "name",
            "description",
            "is_active",
            "min_salary",
            "max_salary",
            "max_incumbents",
            "created_at",
            "updated_at",
            "deleted_at",
            "created_by_id",
            "updated_by_id",
            "deleted_by_id",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "organization_id",
            "organization_position_level_id",
            "code",
            "name",
            "description",
            "is_active",
            "min_salary",
            "max_salary",
            "max_incumbents",
            "created_at",
            "updated_at",
            "deleted_at",
            "created_by_id",
            "updated_by_id",
            "deleted_by_id",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "organization_id",
            "organization_position_level_id",
            "code",
            "name",
            "description",
            "is_active",
            "min_salary",
            "max_salary",
            "max_incumbents",
            "qualifications",
            "responsibilities",
            "created_at",
            "updated_at",
            "deleted_at",
            "created_by_id",
            "updated_by_id",
            "deleted_by_id",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("name", SortDirection::Asc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "organization",
            "level",
            "users",
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

// Implement enhanced query builder traits for OrganizationPosition
impl crate::app::query_builder::Filterable for OrganizationPosition {
    fn apply_basic_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
        match (column, operator) {
            ("name", "contains") => {
                format!("LOWER({}) LIKE LOWER('%{}%')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "starts_with") => {
                format!("LOWER({}) LIKE LOWER('{}%')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("name", "ends_with") => {
                format!("LOWER({}) LIKE LOWER('%{}')", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("code", "=") => {
                format!("{} = '{}'", column, value.as_str().unwrap_or("").replace('\'', "''"))
            }
            ("is_active", "=") => {
                format!("{} = {}", column, value.as_bool().unwrap_or(false))
            }
            ("min_salary", op) | ("max_salary", op) => {
                if let Some(num) = value.as_f64() {
                    format!("{} {} {}", column, op, num)
                } else {
                    format!("{} = 0", column)
                }
            }
            ("max_incumbents", op) => {
                format!("{} {} {}", column, op, value.as_i64().unwrap_or(0))
            }
            _ => {
                match value {
                    serde_json::Value::String(s) => format!("{} {} '{}'", column, operator, s.replace('\'', "''")),
                    serde_json::Value::Number(n) => format!("{} {} {}", column, operator, n),
                    serde_json::Value::Bool(b) => format!("{} {} {}", column, operator, b),
                    serde_json::Value::Null => format!("{} IS NULL", column),
                    _ => format!("{} {} '{}'", column, operator, value.to_string().replace('\'', "''"))
                }
            }
        }
    }
}

impl crate::app::query_builder::Sortable for OrganizationPosition {
    fn apply_basic_sort(column: &str, direction: &str) -> String {
        format!("{} {}", column, direction)
    }
}

impl crate::app::query_builder::Includable for OrganizationPosition {
    fn load_relationships(_ids: &[String], _includes: &[String], _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<()> {
        Ok(())
    }

    fn load_relationship(_ids: &[String], relationship: &str, _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<serde_json::Value> {
        match relationship {
            "organization" => Ok(serde_json::json!({})),
            "level" => Ok(serde_json::json!({})),
            "users" => Ok(serde_json::json!([])),
            _ => Ok(serde_json::json!({}))
        }
    }

    fn build_join_clause(relationship: &str, main_table: &str) -> Option<String> {
        match relationship {
            "organization" => Some(format!("LEFT JOIN organizations ON {}.organization_id = organizations.id", main_table)),
            "level" => Some(format!("LEFT JOIN organization_position_levels ON {}.organization_position_level_id = organization_position_levels.id", main_table)),
            "users" => Some(format!("LEFT JOIN user_organizations ON {}.id = user_organizations.organization_position_id LEFT JOIN sys_users ON user_organizations.user_id = sys_users.id", main_table)),
            "users.organizations" => Some(format!("LEFT JOIN user_organizations ON {}.id = user_organizations.organization_position_id LEFT JOIN sys_users ON user_organizations.user_id = sys_users.id LEFT JOIN organizations AS user_orgs ON user_organizations.organization_id = user_orgs.id", main_table)),
            "organization.parent" => Some(format!("LEFT JOIN organizations ON {}.organization_id = organizations.id LEFT JOIN organizations AS parent_orgs ON organizations.parent_id = parent_orgs.id", main_table)),
            _ => None
        }
    }

    fn get_foreign_key(relationship: &str) -> Option<String> {
        match relationship {
            "organization" => Some("organization_id".to_string()),
            "level" => Some("organization_position_level_id".to_string()),
            "users" => Some("organization_position_id".to_string()),
            _ => None
        }
    }
}

// Implement the query builder service for OrganizationPosition
crate::impl_query_builder_service!(OrganizationPosition);

// Implement activity logging traits
impl crate::app::models::HasModelType for OrganizationPosition {
    fn model_type() -> &'static str {
        "OrganizationPosition"
    }
}

impl crate::app::models::activity_log::HasId for OrganizationPosition {
    fn id(&self) -> String {
        self.id.to_string()
    }
}