use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::{SortDirection};

/// Organization position model representing specific sys_roles within organization position levels
/// Contains position information and relationship to organization position level hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::organization_positions)]
pub struct OrganizationPosition {
    /// Unique identifier for the organization position
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: Ulid,
    /// Organization position name
    #[schema(example = "Software Engineering Manager")]
    pub name: String,
    /// Optional organization position code
    #[schema(example = "SEM")]
    pub code: Option<String>,
    /// ID of the organization position level this position belongs to
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_position_level_id: Ulid,
    /// Optional description of the organization position
    #[schema(example = "Manages software engineering teams and technical projects")]
    pub description: Option<String>,
    /// Whether the organization position is currently active
    #[schema(example = true)]
    pub is_active: bool,
    /// Creation timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// Create organization position payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateOrganizationPosition {
    pub name: String,
    pub code: Option<String>,
    pub organization_position_level_id: String,
    pub description: Option<String>,
}

/// Insertable struct for organization positions
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::organization_positions)]
pub struct NewOrganizationPosition {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub organization_position_level_id: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Update organization position payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrganizationPosition {
    pub name: Option<String>,
    pub code: Option<String>,
    pub organization_position_level_id: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

/// Organization position response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationPositionResponse {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub organization_position_level_id: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NewOrganizationPosition {
    pub fn new(name: String, code: Option<String>, organization_position_level_id: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new().to_string(),
            name,
            code,
            organization_position_level_id,
            description,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}

impl OrganizationPosition {

    pub fn to_response(&self) -> OrganizationPositionResponse {
        OrganizationPositionResponse {
            id: self.id.clone(),
            name: self.name.clone(),
            code: self.code.clone(),
            organization_position_level_id: self.organization_position_level_id.clone(),
            description: self.description.clone(),
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
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
            "name",
            "code",
            "organization_position_level_id",
            "description",
            "is_active",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "code",
            "organization_position_level_id",
            "description",
            "is_active",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "code",
            "organization_position_level_id",
            "description",
            "is_active",
            "created_at",
            "updated_at",
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