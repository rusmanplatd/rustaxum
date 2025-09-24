use super::{DieselUlid, DecimalWrapper};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::{SortDirection};
use serde_json::Value as JsonValue;

/// Organization position model representing specific sys_roles within organization position levels
/// Contains position information and relationship to organization position level hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable)]
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
    pub created_by: Option<DieselUlid>,
    /// User who last updated this position
    pub updated_by: Option<DieselUlid>,
    /// User who deleted this position
    pub deleted_by: Option<DieselUlid>,
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
    pub created_by: Option<DieselUlid>,
    pub updated_by: Option<DieselUlid>,
    pub deleted_by: Option<DieselUlid>,
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
    pub created_by: Option<DieselUlid>,
    pub updated_by: Option<DieselUlid>,
    pub deleted_by: Option<DieselUlid>,
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
            created_by,
            updated_by: created_by,
            deleted_by: None,
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
            created_by: self.created_by,
            updated_by: self.updated_by,
            deleted_by: self.deleted_by,
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
            "created_by",
            "updated_by",
            "deleted_by",
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
            "created_by",
            "updated_by",
            "deleted_by",
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
            "created_by",
            "updated_by",
            "deleted_by",
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
        ]
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