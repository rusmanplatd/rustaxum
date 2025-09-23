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
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::organizations)]
pub struct Organization {
    /// Unique identifier for the organization
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    /// Parent organization ID for hierarchical structure
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub parent_id: Option<DieselUlid>,
    /// Type of organization (company, boc, bod, division, department, branch, subbranch, section)
    #[schema(example = "department")]
    #[diesel(column_name = type_)]
    pub organization_type: String,
    /// Optional organization code
    #[schema(example = "ENG-001")]
    pub code: Option<String>,
    /// Organization level in hierarchy
    #[schema(example = 2)]
    pub level: i32,
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
    pub created_by: Option<DieselUlid>,
    /// User who last updated this organization
    pub updated_by: Option<DieselUlid>,
    /// User who deleted this organization
    pub deleted_by: Option<DieselUlid>,
}

/// Create organization payload for service layer
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateOrganization {
    pub name: String,
    pub organization_type: String,
    pub parent_id: Option<DieselUlid>,
    pub code: Option<String>,
    pub level: Option<i32>,
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

/// Insertable struct for organizations
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::organizations)]
pub struct NewOrganization {
    pub id: DieselUlid,
    pub parent_id: Option<DieselUlid>,
    #[diesel(column_name = type_)]
    pub organization_type: String,
    pub code: Option<String>,
    pub level: i32,
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
    pub created_by: Option<DieselUlid>,
    pub updated_by: Option<DieselUlid>,
    pub deleted_by: Option<DieselUlid>,
}

/// Update organization payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrganization {
    pub name: Option<String>,
    pub organization_type: Option<String>,
    pub parent_id: Option<Option<DieselUlid>>,
    pub code: Option<Option<String>>,
    pub level: Option<i32>,
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
    pub parent_id: Option<DieselUlid>,
    pub organization_type: String,
    pub code: Option<String>,
    pub level: i32,
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
    pub created_by: Option<DieselUlid>,
    pub updated_by: Option<DieselUlid>,
    pub deleted_by: Option<DieselUlid>,
}

impl NewOrganization {
    pub fn new(create_org: CreateOrganization, created_by: Option<DieselUlid>) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            parent_id: create_org.parent_id,
            organization_type: create_org.organization_type,
            code: create_org.code,
            level: create_org.level.unwrap_or(0),
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
            created_by,
            updated_by: created_by,
            deleted_by: None,
        }
    }
}

impl Organization {
    pub fn to_response(&self) -> OrganizationResponse {
        OrganizationResponse {
            id: self.id,
            parent_id: self.parent_id,
            organization_type: self.organization_type.clone(),
            code: self.code.clone(),
            level: self.level,
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
            created_by: self.created_by,
            updated_by: self.updated_by,
            deleted_by: self.deleted_by,
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
            "parent_id",
            "organization_type",
            "code",
            "level",
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
            "created_by",
            "updated_by",
            "deleted_by",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "parent_id",
            "organization_type",
            "code",
            "level",
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
            "created_by",
            "updated_by",
            "deleted_by",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "parent_id",
            "organization_type",
            "code",
            "level",
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
            "parent",
            "children",
            "positions",
            "users",
        ]
    }
}

// Implement the query builder service for Organization
crate::impl_query_builder_service!(Organization);
