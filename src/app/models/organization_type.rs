use super::DieselUlid;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;

/// OrganizationType model representing the type/level of an organization within a domain
/// Provides hierarchical classification for organizations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::schema::organization_types)]
pub struct OrganizationType {
    /// Unique identifier for the type
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    /// Reference to the organization domain
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub domain_id: DieselUlid,
    /// Optional type code
    #[schema(example = "MIN")]
    pub code: Option<String>,
    /// Type name
    #[schema(example = "Ministry")]
    pub name: String,
    /// Optional description
    #[schema(example = "Government ministry level organization")]
    pub description: Option<String>,
    /// Hierarchical level
    #[schema(example = 1)]
    pub level: i32,
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

/// Create organization type payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateOrganizationType {
    pub domain_id: DieselUlid,
    pub code: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub level: i32,
}

/// Insertable struct for organization types
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::organization_types)]
pub struct NewOrganizationType {
    pub id: DieselUlid,
    pub domain_id: DieselUlid,
    pub code: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub level: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by_id: String,
    pub updated_by_id: String,
    pub deleted_by_id: Option<String>,
}

/// Update organization type payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrganizationType {
    pub domain_id: Option<DieselUlid>,
    pub code: Option<Option<String>>,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub level: Option<i32>,
}

/// OrganizationType response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationTypeResponse {
    pub id: DieselUlid,
    pub domain_id: DieselUlid,
    pub code: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub level: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NewOrganizationType {
    pub fn new(create_data: CreateOrganizationType, created_by: Option<DieselUlid>) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            domain_id: create_data.domain_id,
            code: create_data.code,
            name: create_data.name,
            description: create_data.description,
            level: create_data.level,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by_id: created_by.map(|id| id.to_string()).unwrap_or_else(|| "01SYSTEM000000000000000000".to_string()),
            updated_by_id: created_by.map(|id| id.to_string()).unwrap_or_else(|| "01SYSTEM000000000000000000".to_string()),
            deleted_by_id: None,
        }
    }
}

impl OrganizationType {
    pub fn to_response(&self) -> OrganizationTypeResponse {
        OrganizationTypeResponse {
            id: self.id,
            domain_id: self.domain_id,
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            level: self.level,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl crate::app::models::HasModelType for OrganizationType {
    fn model_type() -> &'static str {
        "OrganizationType"
    }
}

impl crate::app::models::activity_log::HasId for OrganizationType {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for OrganizationType {
    fn table_name() -> &'static str {
        "organization_types"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "domain_id",
            "code",
            "name",
            "description",
            "level",
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
            "domain_id",
            "code",
            "name",
            "level",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "domain_id",
            "code",
            "name",
            "description",
            "level",
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
            "domain",
            "organizations",
            "createdBy",
            "updatedBy",
            "deletedBy",
        ]
    }
}

impl crate::app::query_builder::Filterable for OrganizationType {
    fn apply_basic_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
        match operator {
            "=" => format!("{} = {}", column, Self::format_filter_value(value)),
            "!=" => format!("{} != {}", column, Self::format_filter_value(value)),
            _ => format!("{} {} {}", column, operator, Self::format_filter_value(value))
        }
    }
}

impl crate::app::query_builder::Sortable for OrganizationType {
    fn apply_basic_sort(column: &str, direction: &str) -> String {
        format!("{} {}", column, direction)
    }
}

impl crate::app::query_builder::Includable for OrganizationType {
    fn load_relationships(ids: &[String], includes: &[String], _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<()> {
        for include in includes {
            match include.as_str() {
                "domain" => {
                    tracing::debug!("Loading domain for organization types: {:?}", ids);
                },
                "organizations" => {
                    tracing::debug!("Loading organizations for organization types: {:?}", ids);
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
            "domain" => Some("domain_id".to_string()),
            "organizations" => Some("type_id".to_string()),
            _ => None
        }
    }

    fn should_eager_load(relationship: &str) -> bool {
        matches!(relationship, "domain")
    }
}

crate::impl_query_builder_service!(OrganizationType);
