use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use utoipa::ToSchema;

use crate::app::http::form_request::FormRequest;
use crate::app::validation::ValidationRules;
use crate::validation_rules;
use crate::impl_form_request_extractor;
use crate::app::models::DecimalWrapper;
use serde_json::Value as JsonValue;
use chrono::NaiveDate;

/// Request payload for creating a new organization
/// Contains all required and optional fields for organization creation
#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateOrganizationRequest {
    /// Organization name (2-100 characters)
    #[schema(example = "Engineering Department")]
    pub name: String,
    /// Type of organization (company, boc, bod, division, department, branch, subbranch, section)
    #[schema(example = "department")]
    pub organization_type: String,
    /// Parent organization ID (ULID format)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub parent_id: Option<String>,
    /// Optional organization code (2-20 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "ENG-001")]
    pub code: Option<String>,
    /// Organization level in hierarchy
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 2)]
    pub level: Option<i32>,
    /// Organization address
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "123 Main St, City, Country")]
    pub address: Option<String>,
    /// Authorized capital amount
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_capital: Option<DecimalWrapper>,
    /// Business activities description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_activities: Option<String>,
    /// Contact persons information (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_persons: Option<JsonValue>,
    /// Optional description of the organization
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Software engineering and development department")]
    pub description: Option<String>,
    /// Organization email
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "engineering@company.com")]
    pub email: Option<String>,
    /// Date of establishment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub establishment_date: Option<NaiveDate>,
    /// Governance structure (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub governance_structure: Option<JsonValue>,
    /// Legal status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legal_status: Option<String>,
    /// Paid capital amount
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid_capital: Option<DecimalWrapper>,
    /// Organization phone
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "+1234567890")]
    pub phone: Option<String>,
    /// Registration number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_number: Option<String>,
    /// Tax number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_number: Option<String>,
    /// Organization website
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "https://engineering.company.com")]
    pub website: Option<String>,
}

#[async_trait]
impl FormRequest for CreateOrganizationRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "name" => ["required", "string", "min:2", "max:100"],
            "organization_type" => ["required", "string", "min:2", "max:50"],
            "parent_id" => ["string", "regex:^[0-9A-HJKMNP-TV-Z]{26}$"],
            "code" => ["string", "min:2", "max:20", "regex:^[A-Z0-9-_]+$"],
            "description" => ["string", "max:500"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("name.required", "Organization name is required");
        messages.insert("name.min", "Organization name must be at least 2 characters");
        messages.insert("name.max", "Organization name cannot exceed 100 characters");
        messages.insert("organization_type.required", "Organization type is required");
        messages.insert("organization_type.min", "Organization type must be at least 2 characters");
        messages.insert("organization_type.max", "Organization type cannot exceed 50 characters");
        messages.insert("parent_id.regex", "Parent ID must be a valid ULID format");
        messages.insert("code.min", "Organization code must be at least 2 characters");
        messages.insert("code.max", "Organization code cannot exceed 20 characters");
        messages.insert("code.regex", "Organization code must contain only uppercase letters, numbers, hyphens, and underscores");
        messages.insert("description.max", "Description cannot exceed 500 characters");
        messages
    }

    fn prepare_for_validation(&mut self) {
        self.name = self.name.trim().to_string();
        self.organization_type = self.organization_type.trim().to_lowercase();
        if let Some(ref mut parent_id) = self.parent_id {
            *parent_id = parent_id.trim().to_string();
            if parent_id.is_empty() {
                self.parent_id = None;
            }
        }
        if let Some(ref mut code) = self.code {
            *code = code.trim().to_uppercase();
            if code.is_empty() {
                self.code = None;
            }
        }
        if let Some(ref mut description) = self.description {
            *description = description.trim().to_string();
            if description.is_empty() {
                self.description = None;
            }
        }
    }
}

impl_form_request_extractor!(CreateOrganizationRequest);

/// Request payload for updating an existing organization
/// All fields are optional for partial updates
/// @example {"name": "Software Engineering Department"}
#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateOrganizationRequest {
    /// Organization name (2-100 characters, optional)
    #[schema(example = "Software Engineering Department")]
    pub name: Option<String>,
    /// Type of organization (2-50 characters, optional)
    #[schema(example = "division")]
    pub organization_type: Option<String>,
    /// Parent organization ID (ULID format, optional)
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub parent_id: Option<String>,
    /// Organization code (2-20 characters, optional)
    #[schema(example = "SWE-001")]
    pub code: Option<String>,
    /// Description of the organization (optional)
    #[schema(example = "Advanced software engineering and research division")]
    pub description: Option<String>,
    /// Whether the organization is active (optional)
    #[schema(example = true)]
    pub is_active: Option<bool>,
}

#[async_trait]
impl FormRequest for UpdateOrganizationRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "name" => ["string", "min:2", "max:100"],
            "organization_type" => ["string", "min:2", "max:50"],
            "parent_id" => ["string", "regex:^[0-9A-HJKMNP-TV-Z]{26}$"],
            "code" => ["string", "min:2", "max:20", "regex:^[A-Z0-9-_]+$"],
            "description" => ["string", "max:500"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("name.min", "Organization name must be at least 2 characters");
        messages.insert("name.max", "Organization name cannot exceed 100 characters");
        messages.insert("organization_type.min", "Organization type must be at least 2 characters");
        messages.insert("organization_type.max", "Organization type cannot exceed 50 characters");
        messages.insert("parent_id.regex", "Parent ID must be a valid ULID format");
        messages.insert("code.min", "Organization code must be at least 2 characters");
        messages.insert("code.max", "Organization code cannot exceed 20 characters");
        messages.insert("code.regex", "Organization code must contain only uppercase letters, numbers, hyphens, and underscores");
        messages.insert("description.max", "Description cannot exceed 500 characters");
        messages
    }

    fn prepare_for_validation(&mut self) {
        if let Some(ref mut name) = self.name {
            *name = name.trim().to_string();
            if name.is_empty() {
                self.name = None;
            }
        }
        if let Some(ref mut organization_type) = self.organization_type {
            *organization_type = organization_type.trim().to_lowercase();
            if organization_type.is_empty() {
                self.organization_type = None;
            }
        }
        if let Some(ref mut parent_id) = self.parent_id {
            *parent_id = parent_id.trim().to_string();
            if parent_id.is_empty() {
                self.parent_id = None;
            }
        }
        if let Some(ref mut code) = self.code {
            *code = code.trim().to_uppercase();
            if code.is_empty() {
                self.code = None;
            }
        }
        if let Some(ref mut description) = self.description {
            *description = description.trim().to_string();
            if description.is_empty() {
                self.description = None;
            }
        }
    }
}

impl_form_request_extractor!(UpdateOrganizationRequest);