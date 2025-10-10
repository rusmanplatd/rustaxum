use super::DieselUlid;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;

/// OrganizationDomain model representing the domain/sector of an organization
/// Provides high-level categorization for organizations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::schema::organization_domains)]
pub struct OrganizationDomain {
    /// Unique identifier for the domain
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    /// Optional domain code
    #[schema(example = "GOV")]
    pub code: Option<String>,
    /// Domain name
    #[schema(example = "Government")]
    pub name: String,
    /// Optional description
    #[schema(example = "Government and public sector organizations")]
    pub description: Option<String>,
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

/// Create organization domain payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateOrganizationDomain {
    pub code: Option<String>,
    pub name: String,
    pub description: Option<String>,
}

/// Insertable struct for organization domains
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::organization_domains)]
pub struct NewOrganizationDomain {
    pub id: DieselUlid,
    pub code: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by_id: String,
    pub updated_by_id: String,
    pub deleted_by_id: Option<String>,
}

/// Update organization domain payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrganizationDomain {
    pub code: Option<Option<String>>,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
}

/// OrganizationDomain response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationDomainResponse {
    pub id: DieselUlid,
    pub code: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NewOrganizationDomain {
    pub fn new(create_data: CreateOrganizationDomain, created_by: Option<DieselUlid>) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            code: create_data.code,
            name: create_data.name,
            description: create_data.description,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by_id: created_by.map(|id| id.to_string()).unwrap_or_else(|| "01SYSTEM0SEEDER00000000000".to_string()),
            updated_by_id: created_by.map(|id| id.to_string()).unwrap_or_else(|| "01SYSTEM0SEEDER00000000000".to_string()),
            deleted_by_id: None,
        }
    }
}

impl OrganizationDomain {
    pub fn to_response(&self) -> OrganizationDomainResponse {
        OrganizationDomainResponse {
            id: self.id,
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl crate::app::models::HasModelType for OrganizationDomain {
    fn model_type() -> &'static str {
        "OrganizationDomain"
    }
}

impl crate::app::models::activity_log::HasId for OrganizationDomain {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for OrganizationDomain {
    fn table_name() -> &'static str {
        "organization_domains"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "code",
            "name",
            "description",
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
            "code",
            "name",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "code",
            "name",
            "description",
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
            "types",
            "organizations",
            "createdBy",
            "updatedBy",
            "deletedBy",
        ]
    }
}

impl crate::app::query_builder::Filterable for OrganizationDomain {
    fn apply_basic_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
        match operator {
            "=" => format!("{} = {}", column, Self::format_filter_value(value)),
            "!=" => format!("{} != {}", column, Self::format_filter_value(value)),
            _ => format!("{} {} {}", column, operator, Self::format_filter_value(value))
        }
    }
}

impl crate::app::query_builder::Sortable for OrganizationDomain {
    fn apply_basic_sort(column: &str, direction: &str) -> String {
        format!("{} {}", column, direction)
    }
}

impl crate::app::query_builder::Includable for OrganizationDomain {
    fn load_relationships(ids: &[String], includes: &[String], _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<()> {
        for include in includes {
            match include.as_str() {
                "types" => {
                    tracing::debug!("Loading types for organization domains: {:?}", ids);
                },
                "organizations" => {
                    tracing::debug!("Loading organizations for organization domains: {:?}", ids);
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
            "types" => Some("domain_id".to_string()),
            "organizations" => Some("domain_id".to_string()),
            _ => None
        }
    }

    fn should_eager_load(relationship: &str) -> bool {
        matches!(relationship, "types")
    }
}

crate::impl_query_builder_service!(OrganizationDomain);
