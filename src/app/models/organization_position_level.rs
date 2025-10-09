use super::DieselUlid;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::{SortDirection};

/// Job level model representing organizational hierarchy levels
/// Contains level information including rank, code, and description
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::schema::organization_position_levels)]
pub struct OrganizationPositionLevel {
    /// Unique identifier for the organization position level
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    /// Organization ID this position level belongs to
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_id: DieselUlid,
    /// Position level code
    #[schema(example = "SM")]
    pub code: String,
    /// Position level name
    #[schema(example = "Senior Manager")]
    pub name: String,
    /// Optional description of the organization position level
    #[schema(example = "Senior management position with team leadership responsibilities")]
    pub description: Option<String>,
    /// Numeric level ranking (lower number = higher hierarchy)
    #[schema(example = 5)]
    pub level: i32,
    /// Whether the organization position level is currently active
    #[schema(example = true)]
    pub is_active: bool,
    /// Creation timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
    /// Soft delete timestamp
    pub deleted_at: Option<DateTime<Utc>>,
    /// User who created this position level
    pub created_by_id: DieselUlid,
    /// User who last updated this position level
    pub updated_by_id: DieselUlid,
    /// User who deleted this position level
    pub deleted_by_id: Option<DieselUlid>,
}

/// Create organization position level payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateOrganizationPositionLevel {
    pub organization_id: DieselUlid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub level: i32,
}

/// Insertable struct for organization position levels
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::organization_position_levels)]
pub struct NewOrganizationPositionLevel {
    pub id: DieselUlid,
    pub organization_id: DieselUlid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub level: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by_id: DieselUlid,
    pub updated_by_id: DieselUlid,
    pub deleted_by_id: Option<DieselUlid>,
}

/// Update organization position level payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrganizationPositionLevel {
    pub organization_id: Option<DieselUlid>,
    pub code: Option<String>,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub level: Option<i32>,
    pub is_active: Option<bool>,
}

/// Job level response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationPositionLevelResponse {
    pub id: DieselUlid,
    pub organization_id: DieselUlid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub level: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by_id: DieselUlid,
    pub updated_by_id: DieselUlid,
    pub deleted_by_id: Option<DieselUlid>,
}

impl NewOrganizationPositionLevel {
    pub fn new(create_level: CreateOrganizationPositionLevel, created_by: Option<DieselUlid>) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            organization_id: create_level.organization_id,
            code: create_level.code,
            name: create_level.name,
            description: create_level.description,
            level: create_level.level,
            is_active: true,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by_id: created_by,
            updated_by_id: created_by,
            deleted_by_id: None,
        }
    }
}

impl OrganizationPositionLevel {
    pub fn to_response(&self) -> OrganizationPositionLevelResponse {
        OrganizationPositionLevelResponse {
            id: self.id,
            organization_id: self.organization_id,
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            level: self.level,
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
            created_by_id: self.created_by_id,
            updated_by_id: self.updated_by_id,
            deleted_by_id: self.deleted_by_id,
        }
    }
}

impl crate::app::query_builder::Queryable for OrganizationPositionLevel {
    fn table_name() -> &'static str {
        "organization_position_levels"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "organization_id",
            "code",
            "name",
            "level",
            "description",
            "is_active",
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
            "code",
            "name",
            "level",
            "description",
            "is_active",
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
            "code",
            "name",
            "description",
            "level",
            "is_active",
            "created_at",
            "updated_at",
            "deleted_at",
            "created_by_id",
            "updated_by_id",
            "deleted_by_id",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("level", SortDirection::Asc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "positions",
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

// Implement the query builder service for OrganizationPositionLevel
crate::impl_query_builder_service!(OrganizationPositionLevel);

// Implement activity logging traits
impl crate::app::models::HasModelType for OrganizationPositionLevel {
    fn model_type() -> &'static str {
        "OrganizationPositionLevel"
    }
}

impl crate::app::models::activity_log::HasId for OrganizationPositionLevel {
    fn id(&self) -> String {
        self.id.to_string()
    }
}