use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::{Queryable, SortDirection};

/// Job level model representing organizational hierarchy levels
/// Contains level information including rank, code, and description
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, diesel::Queryable, Identifiable)]
#[diesel(table_name = crate::schema::organization_position_levels)]
pub struct OrganizationPositionLevel {
    /// Unique identifier for the organization position level
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: Ulid,
    /// Job level name
    #[schema(example = "Senior Manager")]
    pub name: String,
    /// Optional organization position level code
    #[schema(example = "SM")]
    pub code: Option<String>,
    /// Numeric level ranking (higher number = higher level)
    #[schema(example = 5)]
    pub level: i32,
    /// Optional description of the organization position level
    #[schema(example = "Senior management position with team leadership responsibilities")]
    pub description: Option<String>,
    /// Whether the organization position level is currently active
    #[schema(example = true)]
    pub is_active: bool,
    /// Creation timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// Create organization position level payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateOrganizationPositionLevel {
    pub name: String,
    pub code: Option<String>,
    pub level: i32,
    pub description: Option<String>,
}

/// Insertable struct for organization position levels
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::organization_position_levels)]
pub struct NewOrganizationPositionLevel {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub level: i32,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Update organization position level payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrganizationPositionLevel {
    pub name: Option<String>,
    pub code: Option<String>,
    pub level: Option<i32>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

/// Job level response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationPositionLevelResponse {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub level: i32,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OrganizationPositionLevel {
    pub fn new(name: String, code: Option<String>, level: i32, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            name,
            code,
            level,
            description,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> OrganizationPositionLevelResponse {
        OrganizationPositionLevelResponse {
            id: self.id.to_string(),
            name: self.name.clone(),
            code: self.code.clone(),
            level: self.level,
            description: self.description.clone(),
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl crate::app::query_builder::Queryable for OrganizationPositionLevel {
    fn table_name() -> &'static str {
        "OrganizationPositionLevel"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "code",
            "level",
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
            "level",
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
            "level",
            "description",
            "is_active",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("level", SortDirection::Asc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "positions",
        ]
    }
}

// Implement the query builder service for OrganizationPositionLevel
crate::impl_query_builder_service!(OrganizationPositionLevel);