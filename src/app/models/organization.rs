use super::{DieselUlid, DecimalWrapper};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc, NaiveDate};
use utoipa::ToSchema;
use crate::app::query_builder::{SortDirection};
use super::{HasModelType, HasRoles};
use serde_json::Value as JsonValue;

/// Organization model representing an organizational entity
/// Contains organizational information including hierarchy and metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::organizations)]
pub struct Organization {
    /// Unique identifier for the organization
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    /// Reference to the organization domain
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub domain_id: DieselUlid,
    /// Parent organization ID for hierarchical structure
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub parent_id: Option<DieselUlid>,
    /// Reference to the organization type
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub type_id: DieselUlid,
    #[schema(example = "ENG-001")]
    pub code: Option<String>,
    /// Organization name
    #[schema(example = "Engineering Department")]
    pub name: String,
    /// Organization address
    #[schema(example = "123 Main St, City, Country")]
    pub address: Option<String>,
    /// Authorized capital amount
    pub authorized_capital: Option<DecimalWrapper>,
    /// Business activities description
    pub business_activities: Option<String>,
    /// Contact persons information (JSON)
    pub contact_persons: Option<JsonValue>,
    /// Optional description of the organization
    #[schema(example = "Software engineering and development department")]
    pub description: Option<String>,
    /// Organization email
    #[schema(example = "engineering@company.com")]
    pub email: Option<String>,
    /// Date of establishment
    pub establishment_date: Option<NaiveDate>,
    /// Governance structure (JSON)
    pub governance_structure: Option<JsonValue>,
    /// Legal status
    pub legal_status: Option<String>,
    /// Paid capital amount
    pub paid_capital: Option<DecimalWrapper>,
    /// Hierarchical path
    pub path: Option<String>,
    /// Organization phone
    #[schema(example = "+1234567890")]
    pub phone: Option<String>,
    /// Registration number
    pub registration_number: Option<String>,
    /// Tax number
    pub tax_number: Option<String>,
    /// Organization website
    #[schema(example = "https://engineering.company.com")]
    pub website: Option<String>,
    /// Whether the organization is currently active
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
    /// User who created this organization
    pub created_by_id: DieselUlid,
    /// User who last updated this organization
    pub updated_by_id: DieselUlid,
    /// User who deleted this organization
    pub deleted_by_id: Option<DieselUlid>,
}

/// Create organization payload for service layer
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateOrganization {
    pub domain_id: DieselUlid,
    pub type_id: DieselUlid,
    pub name: String,
    pub parent_id: Option<DieselUlid>,
    pub code: Option<String>,
    pub address: Option<String>,
    pub authorized_capital: Option<DecimalWrapper>,
    pub business_activities: Option<String>,
    pub contact_persons: Option<JsonValue>,
    pub description: Option<String>,
    pub email: Option<String>,
    pub establishment_date: Option<NaiveDate>,
    pub governance_structure: Option<JsonValue>,
    pub legal_status: Option<String>,
    pub paid_capital: Option<DecimalWrapper>,
    pub path: Option<String>,
    pub phone: Option<String>,
    pub registration_number: Option<String>,
    pub tax_number: Option<String>,
    pub website: Option<String>,
}

/// Update organization payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrganization {
    pub domain_id: Option<DieselUlid>,
    pub type_id: Option<DieselUlid>,
    pub name: Option<String>,
    pub parent_id: Option<Option<DieselUlid>>,
    pub code: Option<Option<String>>,
    pub address: Option<Option<String>>,
    pub authorized_capital: Option<Option<DecimalWrapper>>,
    pub business_activities: Option<Option<String>>,
    pub contact_persons: Option<Option<JsonValue>>,
    pub description: Option<Option<String>>,
    pub email: Option<Option<String>>,
    pub establishment_date: Option<Option<NaiveDate>>,
    pub governance_structure: Option<Option<JsonValue>>,
    pub legal_status: Option<Option<String>>,
    pub paid_capital: Option<Option<DecimalWrapper>>,
    pub path: Option<Option<String>>,
    pub phone: Option<Option<String>>,
    pub registration_number: Option<Option<String>>,
    pub tax_number: Option<Option<String>>,
    pub website: Option<Option<String>>,
    pub is_active: Option<bool>,
}

/// Organization response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationResponse {
    pub id: DieselUlid,
    pub domain_id: DieselUlid,
    pub parent_id: Option<DieselUlid>,
    pub type_id: DieselUlid,
    pub code: Option<String>,
    pub name: String,
    pub address: Option<String>,
    pub authorized_capital: Option<DecimalWrapper>,
    pub business_activities: Option<String>,
    pub contact_persons: Option<JsonValue>,
    pub description: Option<String>,
    pub email: Option<String>,
    pub establishment_date: Option<NaiveDate>,
    pub governance_structure: Option<JsonValue>,
    pub legal_status: Option<String>,
    pub paid_capital: Option<DecimalWrapper>,
    pub path: Option<String>,
    pub phone: Option<String>,
    pub registration_number: Option<String>,
    pub tax_number: Option<String>,
    pub website: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by_id: DieselUlid,
    pub updated_by_id: DieselUlid,
    pub deleted_by_id: Option<DieselUlid>,
}

impl Organization {
    pub fn new(create_org: CreateOrganization, created_by: Option<DieselUlid>) -> Self {
        let now = Utc::now();
        Organization {
            id: DieselUlid::new(),
            domain_id: create_org.domain_id,
            parent_id: create_org.parent_id,
            type_id: create_org.type_id,
            code: create_org.code,
            name: create_org.name,
            address: create_org.address,
            authorized_capital: create_org.authorized_capital,
            business_activities: create_org.business_activities,
            contact_persons: create_org.contact_persons,
            description: create_org.description,
            email: create_org.email,
            establishment_date: create_org.establishment_date,
            governance_structure: create_org.governance_structure,
            legal_status: create_org.legal_status,
            paid_capital: create_org.paid_capital,
            path: create_org.path,
            phone: create_org.phone,
            registration_number: create_org.registration_number,
            tax_number: create_org.tax_number,
            website: create_org.website,
            is_active: true,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by_id: created_by.unwrap_or_else(|| DieselUlid::from_string("01SYSTEM0SEEDER00000000000").unwrap()),
            updated_by_id: created_by.unwrap_or_else(|| DieselUlid::from_string("01SYSTEM0SEEDER00000000000").unwrap()),
            deleted_by_id: None,
        }
    }

    pub fn to_response(&self) -> OrganizationResponse {
        OrganizationResponse {
            id: self.id,
            domain_id: self.domain_id,
            parent_id: self.parent_id,
            type_id: self.type_id,
            code: self.code.clone(),
            name: self.name.clone(),
            address: self.address.clone(),
            authorized_capital: self.authorized_capital.clone(),
            business_activities: self.business_activities.clone(),
            contact_persons: self.contact_persons.clone(),
            description: self.description.clone(),
            email: self.email.clone(),
            establishment_date: self.establishment_date,
            governance_structure: self.governance_structure.clone(),
            legal_status: self.legal_status.clone(),
            paid_capital: self.paid_capital.clone(),
            path: self.path.clone(),
            phone: self.phone.clone(),
            registration_number: self.registration_number.clone(),
            tax_number: self.tax_number.clone(),
            website: self.website.clone(),
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

impl HasModelType for Organization {
    fn model_type() -> &'static str {
        "Organization"
    }
}

impl HasRoles for Organization {
    fn model_id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::models::activity_log::HasId for Organization {
    fn id(&self) -> String {
        self.id.to_string()
    }
}


impl crate::app::query_builder::Queryable for Organization {
    fn table_name() -> &'static str {
        "organizations"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "domain_id",
            "parent_id",
            "type_id",
            "code",
            "name",
            "address",
            "email",
            "phone",
            "legal_status",
            "registration_number",
            "tax_number",
            "website",
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
            "domain_id",
            "parent_id",
            "type_id",
            "code",
            "name",
            "address",
            "email",
            "phone",
            "legal_status",
            "registration_number",
            "tax_number",
            "website",
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
            "domain_id",
            "parent_id",
            "type_id",
            "code",
            "name",
            "address",
            "authorized_capital",
            "business_activities",
            "contact_persons",
            "description",
            "email",
            "establishment_date",
            "governance_structure",
            "legal_status",
            "paid_capital",
            "path",
            "phone",
            "registration_number",
            "tax_number",
            "website",
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
            "domain",
            "type",
            "parent",
            "children",
            "levels",
            "positions",
            "users",
            "positions.level",
            "users.roles",
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

// Implement the enhanced filtering trait
impl crate::app::query_builder::Filterable for Organization {
    fn apply_basic_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
        match operator {
            "=" => format!("{} = {}", column, Self::format_filter_value(value)),
            "!=" => format!("{} != {}", column, Self::format_filter_value(value)),
            _ => format!("{} {} {}", column, operator, Self::format_filter_value(value))
        }
    }
}

// Implement the enhanced sorting trait
impl crate::app::query_builder::Sortable for Organization {
    fn apply_basic_sort(column: &str, direction: &str) -> String {
        format!("{} {}", column, direction)
    }
}

// Implement the relationship inclusion trait
impl crate::app::query_builder::Includable for Organization {
    fn load_relationships(ids: &[String], includes: &[String], _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<()> {
        for include in includes {
            match include.as_str() {
                "roles" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_roles("organizations", ids, _conn)?;
                },
                "permissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_permissions("organizations", ids, _conn)?;
                },
                "roles.permissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_roles_with_permissions("organizations", ids, _conn)?;
                },
                "permissions.roles" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_permissions_with_roles("organizations", ids, _conn)?;
                },
                "roles.organization" => {
                    crate::app::query_builder::RolePermissionLoader::load_roles_with_organization("organizations", ids, _conn)?;
                },
                "permissions.organization" => {
                    crate::app::query_builder::RolePermissionLoader::load_permissions_with_organization("organizations", ids, _conn)?;
                },
                "authorizationContext" => {
                    crate::app::query_builder::RolePermissionLoader::load_complete_authorization_context("organizations", ids, _conn)?;
                },
                "scopedRoles" => {
                    crate::app::query_builder::RolePermissionLoader::load_scoped_roles("organizations", ids, _conn)?;
                },
                "scopedPermissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_scoped_permissions("organizations", ids, _conn)?;
                },
                "parent" => {
                    tracing::debug!("Loading parent organization for organizations: {:?}", ids);
                },
                "children" => {
                    tracing::debug!("Loading child organizations for organizations: {:?}", ids);
                },
                "positions" => {
                    tracing::debug!("Loading positions for organizations: {:?}", ids);
                },
                "users" => {
                    tracing::debug!("Loading users for organizations: {:?}", ids);
                },
                "positions.level" => {
                    tracing::debug!("Loading positions.level for organizations: {:?}", ids);
                },
                "users.roles" => {
                    tracing::debug!("Loading users.roles for organizations: {:?}", ids);
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
            "parent" => Some("parent_id".to_string()),
            "children" => Some("parent_id".to_string()),
            "positions" => Some("organization_id".to_string()),
            "users" => Some("organization_id".to_string()),
            _ => None
        }
    }

    fn should_eager_load(relationship: &str) -> bool {
        // Load positions and users eagerly for organizational hierarchy
        matches!(relationship, "positions" | "users")
    }
}

// Implement the query builder service for Organization
crate::impl_query_builder_service!(Organization);
