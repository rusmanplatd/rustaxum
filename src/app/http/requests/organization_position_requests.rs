use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use utoipa::ToSchema;

use crate::app::http::form_request::FormRequest;
use crate::app::validation::ValidationRules;
use crate::validation_rules;
use crate::impl_form_request_extractor;
use crate::app::models::{DieselUlid, DecimalWrapper};
use serde_json::Value as JsonValue;

/// Create organization position form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateOrganizationPositionRequest {
    /// Organization ID this position belongs to
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_id: DieselUlid,
    /// Position level ID (ULID format)
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_position_level_id: DieselUlid,
    /// Position code (2-20 characters)
    #[schema(example = "SSE001")]
    pub code: String,
    /// Position name (2-100 characters)
    #[schema(example = "Senior Software Engineer")]
    pub name: String,
    /// Position description (optional, max 500 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Senior software engineer responsible for system architecture")]
    pub description: Option<String>,
    /// Minimum salary for this position
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "75000.00")]
    pub min_salary: Option<DecimalWrapper>,
    /// Maximum salary for this position
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "120000.00")]
    pub max_salary: Option<DecimalWrapper>,
    /// Maximum number of incumbents allowed
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 5)]
    pub max_incumbents: Option<i32>,
    /// Required qualifications (JSON array)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qualifications: Option<JsonValue>,
    /// Position responsibilities (JSON array)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responsibilities: Option<JsonValue>,
}

#[async_trait]
impl FormRequest for CreateOrganizationPositionRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "organization_id" => ["required", "string", "regex:^[0-9A-HJKMNP-TV-Z]{26}$"],
            "organization_position_level_id" => ["required", "string", "regex:^[0-9A-HJKMNP-TV-Z]{26}$"],
            "code" => ["required", "string", "min:2", "max:20"],
            "name" => ["required", "string", "min:2", "max:100"],
            "description" => ["string", "max:500"],
            "min_salary" => ["numeric", "min:0"],
            "max_salary" => ["numeric", "min:0"],
            "max_incumbents" => ["numeric", "min:1"],
            "qualifications" => ["json"],
            "responsibilities" => ["json"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("organization_id.required", "Organization ID is required");
        messages.insert("organization_id.regex", "Organization ID must be a valid ULID");
        messages.insert("organization_position_level_id.required", "Position level ID is required");
        messages.insert("organization_position_level_id.regex", "Position level ID must be a valid ULID");
        messages.insert("code.required", "Position code is required");
        messages.insert("code.min", "Position code must be at least 2 characters");
        messages.insert("code.max", "Position code cannot exceed 20 characters");
        messages.insert("name.required", "Position name is required");
        messages.insert("name.min", "Position name must be at least 2 characters");
        messages.insert("name.max", "Position name cannot exceed 100 characters");
        messages.insert("description.max", "Description cannot exceed 500 characters");
        messages.insert("min_salary.min", "Minimum salary must be at least 0");
        messages.insert("max_salary.min", "Maximum salary must be at least 0");
        messages.insert("max_incumbents.min", "Maximum incumbents must be at least 1");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("name", "organization position name");
        attributes.insert("code", "organization position code");
        attributes.insert("organization_position_level_id", "organization position level");
        attributes.insert("description", "organization position description");
        attributes
    }
}

impl_form_request_extractor!(CreateOrganizationPositionRequest);

/// Update organization position form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateOrganizationPositionRequest {
    /// Organization ID this position belongs to (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_id: Option<DieselUlid>,
    /// Position level ID (optional, ULID format)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_position_level_id: Option<DieselUlid>,
    /// Position code (optional, 2-20 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "SSE001")]
    pub code: Option<String>,
    /// Position name (optional, 2-100 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Senior Software Engineer")]
    pub name: Option<String>,
    /// Position description (optional, max 500 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Senior software engineer responsible for system architecture")]
    pub description: Option<Option<String>>,
    /// Active status
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = true)]
    pub is_active: Option<bool>,
    /// Minimum salary for this position
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "75000.00")]
    pub min_salary: Option<DecimalWrapper>,
    /// Maximum salary for this position
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "120000.00")]
    pub max_salary: Option<DecimalWrapper>,
    /// Maximum number of incumbents allowed
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 5)]
    pub max_incumbents: Option<i32>,
    /// Required qualifications (JSON array)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qualifications: Option<JsonValue>,
    /// Position responsibilities (JSON array)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responsibilities: Option<JsonValue>,
}

#[async_trait]
impl FormRequest for UpdateOrganizationPositionRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "organization_id" => ["string", "regex:^[0-9A-HJKMNP-TV-Z]{26}$"],
            "organization_position_level_id" => ["string", "regex:^[0-9A-HJKMNP-TV-Z]{26}$"],
            "code" => ["string", "min:2", "max:20"],
            "name" => ["string", "min:2", "max:100"],
            "description" => ["string", "max:500"],
            "is_active" => ["boolean"],
            "min_salary" => ["numeric", "min:0"],
            "max_salary" => ["numeric", "min:0"],
            "max_incumbents" => ["numeric", "min:1"],
            "qualifications" => ["json"],
            "responsibilities" => ["json"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("organization_id.regex", "Organization ID must be a valid ULID");
        messages.insert("organization_position_level_id.regex", "Position level ID must be a valid ULID");
        messages.insert("code.min", "Position code must be at least 2 characters");
        messages.insert("code.max", "Position code cannot exceed 20 characters");
        messages.insert("name.min", "Position name must be at least 2 characters");
        messages.insert("name.max", "Position name cannot exceed 100 characters");
        messages.insert("description.max", "Description cannot exceed 500 characters");
        messages.insert("is_active.boolean", "Active status must be true or false");
        messages.insert("min_salary.min", "Minimum salary must be at least 0");
        messages.insert("max_salary.min", "Maximum salary must be at least 0");
        messages.insert("max_incumbents.min", "Maximum incumbents must be at least 1");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("organization_id", "organization ID");
        attributes.insert("organization_position_level_id", "position level ID");
        attributes.insert("code", "position code");
        attributes.insert("name", "position name");
        attributes.insert("description", "position description");
        attributes.insert("is_active", "active status");
        attributes.insert("min_salary", "minimum salary");
        attributes.insert("max_salary", "maximum salary");
        attributes.insert("max_incumbents", "maximum incumbents");
        attributes.insert("qualifications", "qualifications");
        attributes.insert("responsibilities", "responsibilities");
        attributes
    }
}

impl_form_request_extractor!(UpdateOrganizationPositionRequest);

/// Index/list organization positions form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct IndexOrganizationPositionRequest {
    /// Page number (default: 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 1)]
    pub page: Option<u32>,
    /// Items per page (1-100, default: 15)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 15)]
    pub per_page: Option<u32>,
    /// Sort field (name, code, created_at, updated_at)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "name")]
    pub sort_by: Option<String>,
    /// Sort direction (asc, desc)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "asc")]
    pub sort_direction: Option<String>,
    /// Filter by active status
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = true)]
    pub is_active: Option<bool>,
    /// Filter by organization position level ID (ULID format)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_position_level_id: Option<String>,
    /// Filter by name (partial match)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "engineer")]
    pub name_search: Option<String>,
}

#[async_trait]
impl FormRequest for IndexOrganizationPositionRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "page" => ["numeric", "min:1"],
            "per_page" => ["numeric", "min:1", "max:100"],
            "sort_by" => ["string", "in:name,code,created_at,updated_at"],
            "sort_direction" => ["string", "in:asc,desc"],
            "is_active" => ["boolean"],
            "organization_position_level_id" => ["string", "ulid_format"],
            "name_search" => ["string", "min:2", "max:100"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("page.min", "Page must be at least 1");
        messages.insert("per_page.min", "Per page must be at least 1");
        messages.insert("per_page.max", "Per page cannot exceed 100");
        messages.insert("sort_by.in", "Sort by must be one of: name, code, created_at, updated_at");
        messages.insert("sort_direction.in", "Sort direction must be either asc or desc");
        messages.insert("is_active.boolean", "Active status must be true or false");
        messages.insert("organization_position_level_id.ulid_format", "Job level ID must be a valid ULID");
        messages.insert("name_search.min", "Name search must be at least 2 characters");
        messages.insert("name_search.max", "Name search cannot exceed 100 characters");
        messages
    }

    fn prepare_for_validation(&mut self) {
        // Set default values
        if self.page.is_none() {
            self.page = Some(1);
        }
        if self.per_page.is_none() {
            self.per_page = Some(15);
        }
        if self.sort_by.is_none() {
            self.sort_by = Some("name".to_string());
        }
        if self.sort_direction.is_none() {
            self.sort_direction = Some("asc".to_string());
        }
    }
}

impl_form_request_extractor!(IndexOrganizationPositionRequest);

/// Organization positions by organization position level form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct OrganizationPositionsByLevelRequest {
    /// Job level ID (ULID format)
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_position_level_id: String,
    /// Include inactive positions
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = false)]
    pub include_inactive: Option<bool>,
}

#[async_trait]
impl FormRequest for OrganizationPositionsByLevelRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "organization_position_level_id" => ["required", "string", "ulid_format"],
            "include_inactive" => ["boolean"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("organization_position_level_id.required", "Job level ID is required");
        messages.insert("organization_position_level_id.ulid_format", "Job level ID must be a valid ULID");
        messages.insert("include_inactive.boolean", "Include inactive must be true or false");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("organization_position_level_id", "organization position level");
        attributes.insert("include_inactive", "include inactive flag");
        attributes
    }

    fn prepare_for_validation(&mut self) {
        if self.include_inactive.is_none() {
            self.include_inactive = Some(false);
        }
    }
}

impl_form_request_extractor!(OrganizationPositionsByLevelRequest);