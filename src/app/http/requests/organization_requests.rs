use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use utoipa::ToSchema;

use crate::app::http::form_request::FormRequest;
use crate::app::utils::validator::{Rule, required, string, min, max, regex};
use crate::impl_form_request_extractor;

/// Request payload for creating a new organization
/// Contains all required and optional fields for organization creation
/// @example {"name": "Engineering Department", "organization_type": "department", "code": "ENG-001"}
#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateOrganizationRequest {
    /// Organization name (2-100 characters)
    #[schema(example = "Engineering Department")]
    pub name: String,
    /// Type of organization (2-50 characters)
    #[schema(example = "department")]
    pub organization_type: String,
    /// Parent organization ID (ULID format)
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub parent_id: Option<String>,
    /// Optional organization code (2-20 characters)
    #[schema(example = "ENG-001")]
    pub code: Option<String>,
    /// Optional description of the organization
    #[schema(example = "Software engineering and development department")]
    pub description: Option<String>,
}

#[async_trait]
impl FormRequest for CreateOrganizationRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("name", vec![required(), string(), min("2"), max("100")]);
        rules.insert("organization_type", vec![required(), string(), min("2"), max("50")]);
        rules.insert("parent_id", vec![string(), regex("^[0-9A-HJKMNP-TV-Z]{26}$")]);
        rules.insert("code", vec![string(), min("2"), max("20"), regex("^[A-Z0-9-_]+$")]);
        rules.insert("description", vec![string(), max("500")]);
        rules
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
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("name", vec![string(), min("2"), max("100")]);
        rules.insert("organization_type", vec![string(), min("2"), max("50")]);
        rules.insert("parent_id", vec![string(), regex("^[0-9A-HJKMNP-TV-Z]{26}$")]);
        rules.insert("code", vec![string(), min("2"), max("20"), regex("^[A-Z0-9-_]+$")]);
        rules.insert("description", vec![string(), max("500")]);
        rules
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