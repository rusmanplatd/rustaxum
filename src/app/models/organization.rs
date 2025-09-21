use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::{SortDirection};
use super::{HasModelType, HasRoles};

/// Organization model representing an organizational entity
/// Contains organizational information including hierarchy and metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::organizations)]
pub struct Organization {
    /// Unique identifier for the organization
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// Organization name
    #[schema(example = "Engineering Department")]
    pub name: String,
    /// Type of organization (department, division, company, etc.)
    #[schema(example = "department")]
    #[diesel(column_name = type_)]
    pub organization_type: String,
    /// Parent organization ID for hierarchical structure
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub parent_id: Option<String>,
    /// Optional organization code
    #[schema(example = "ENG-001")]
    pub code: Option<String>,
    /// Optional description of the organization
    #[schema(example = "Software engineering and development department")]
    pub description: Option<String>,
    /// Whether the organization is currently active
    #[schema(example = true)]
    pub is_active: bool,
    /// Creation timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// Create organization payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateOrganization {
    pub name: String,
    pub organization_type: String,
    pub parent_id: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
}

/// Insertable struct for organizations
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::organizations)]
pub struct NewOrganization {
    pub id: String,
    pub name: String,
    #[diesel(column_name = type_)]
    pub organization_type: String,
    pub parent_id: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Update organization payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrganization {
    pub name: Option<String>,
    pub organization_type: Option<String>,
    pub parent_id: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

/// Organization response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationResponse {
    pub id: String,
    pub name: String,
    pub organization_type: String,
    pub parent_id: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NewOrganization {
    pub fn new(name: String, organization_type: String, parent_id: Option<String>, code: Option<String>, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new().to_string(),
            name,
            organization_type,
            parent_id,
            code,
            description,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Organization {

    pub fn to_response(&self) -> OrganizationResponse {
        OrganizationResponse {
            id: self.id.clone(),
            name: self.name.clone(),
            organization_type: self.organization_type.clone(),
            parent_id: self.parent_id.clone(),
            code: self.code.clone(),
            description: self.description.clone(),
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl HasModelType for Organization {
    fn model_type() -> &'static str {
        "Organization"
    }
}

impl HasRoles for Organization {
    fn model_id(&self) -> String {
        self.id.clone()
    }
}


impl crate::app::query_builder::Queryable for Organization {
    fn table_name() -> &'static str {
        "organizations"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "organization_type",
            "parent_id",
            "code",
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
            "organization_type",
            "parent_id",
            "code",
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
            "organization_type",
            "parent_id",
            "code",
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
            "parent",
            "children",
            "positions",
            "users",
        ]
    }
}

// Implement the query builder service for Organization
crate::impl_query_builder_service!(Organization);
