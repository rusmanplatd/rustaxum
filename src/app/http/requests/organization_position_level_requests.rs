use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use utoipa::ToSchema;

use crate::app::http::form_request::FormRequest;
use crate::app::validation::ValidationRules;
use crate::validation_rules;
use crate::impl_form_request_extractor;
use crate::app::models::DieselUlid;

/// Create organization position level form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateOrganizationPositionLevelRequest {
    /// Organization ID this position level belongs to
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_id: DieselUlid,
    /// Position level code (2-20 characters)
    #[schema(example = "SL5")]
    pub code: String,
    /// Position level name (2-100 characters)
    #[schema(example = "Senior Level")]
    pub name: String,
    /// Position level description (optional, max 500 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Senior level position with 5+ years experience")]
    pub description: Option<String>,
    /// Numeric level for hierarchy (1-20)
    #[schema(example = 5)]
    pub level: i32,
}

#[async_trait]
impl FormRequest for CreateOrganizationPositionLevelRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "organization_id" => ["required", "string", "regex:^[0-9A-HJKMNP-TV-Z]{26}$"],
            "code" => ["required", "string", "min:2", "max:20"],
            "name" => ["required", "string", "min:2", "max:100"],
            "description" => ["string", "max:500"],
            "level" => ["required", "numeric", "min:1", "max:20"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("organization_id.required", "Organization ID is required");
        messages.insert("organization_id.regex", "Organization ID must be a valid ULID");
        messages.insert("code.required", "Position level code is required");
        messages.insert("code.min", "Position level code must be at least 2 characters");
        messages.insert("code.max", "Position level code cannot exceed 20 characters");
        messages.insert("name.required", "Position level name is required");
        messages.insert("name.min", "Position level name must be at least 2 characters");
        messages.insert("name.max", "Position level name cannot exceed 100 characters");
        messages.insert("description.max", "Description cannot exceed 500 characters");
        messages.insert("level.required", "Level numeric value is required");
        messages.insert("level.min", "Level must be at least 1");
        messages.insert("level.max", "Level cannot exceed 20");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("organization_id", "organization ID");
        attributes.insert("code", "position level code");
        attributes.insert("name", "position level name");
        attributes.insert("description", "position level description");
        attributes.insert("level", "level number");
        attributes
    }
}

impl_form_request_extractor!(CreateOrganizationPositionLevelRequest);

/// Update organization position level form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateOrganizationPositionLevelRequest {
    /// Organization ID this position level belongs to (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_id: Option<DieselUlid>,
    /// Position level code (optional, 2-20 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "SL5")]
    pub code: Option<String>,
    /// Position level name (optional, 2-100 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Senior Level")]
    pub name: Option<String>,
    /// Position level description (optional, max 500 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Senior level position with 5+ years experience")]
    pub description: Option<Option<String>>,
    /// Numeric level for hierarchy (optional, 1-20)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 5)]
    pub level: Option<i32>,
    /// Active status
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = true)]
    pub is_active: Option<bool>,
}

#[async_trait]
impl FormRequest for UpdateOrganizationPositionLevelRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "organization_id" => ["string", "regex:^[0-9A-HJKMNP-TV-Z]{26}$"],
            "code" => ["string", "min:2", "max:20"],
            "name" => ["string", "min:2", "max:100"],
            "description" => ["string", "max:500"],
            "level" => ["numeric", "min:1", "max:20"],
            "is_active" => ["boolean"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("organization_id.regex", "Organization ID must be a valid ULID");
        messages.insert("code.min", "Position level code must be at least 2 characters");
        messages.insert("code.max", "Position level code cannot exceed 20 characters");
        messages.insert("name.min", "Position level name must be at least 2 characters");
        messages.insert("name.max", "Position level name cannot exceed 100 characters");
        messages.insert("description.max", "Description cannot exceed 500 characters");
        messages.insert("level.min", "Level must be at least 1");
        messages.insert("level.max", "Level cannot exceed 20");
        messages.insert("is_active.boolean", "Active status must be true or false");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("organization_id", "organization ID");
        attributes.insert("code", "position level code");
        attributes.insert("name", "position level name");
        attributes.insert("description", "position level description");
        attributes.insert("level", "level number");
        attributes.insert("is_active", "active status");
        attributes
    }
}

impl_form_request_extractor!(UpdateOrganizationPositionLevelRequest);

/// Index/list organization position levels form request
#[derive(Deserialize, Serialize, ToSchema, Clone)]
pub struct IndexOrganizationPositionLevelRequest {
    /// Page number (default: 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 1)]
    pub page: Option<u32>,
    /// Items per page (1-100, default: 15)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 15)]
    pub per_page: Option<u32>,
    /// Sort field (name, code, level, created_at, updated_at)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "level")]
    pub sort_by: Option<String>,
    /// Sort direction (asc, desc)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "asc")]
    pub sort_direction: Option<String>,
    /// Filter by active status
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = true)]
    pub is_active: Option<bool>,
    /// Filter by minimum level
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 1)]
    pub min_level: Option<i32>,
    /// Filter by maximum level
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 10)]
    pub max_level: Option<i32>,
}

#[async_trait]
impl FormRequest for IndexOrganizationPositionLevelRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "page" => ["numeric", "min:1"],
            "per_page" => ["numeric", "min:1", "max:100"],
            "sort_by" => ["string", "in:name,code,level,created_at,updated_at"],
            "sort_direction" => ["string", "in:asc,desc"],
            "is_active" => ["boolean"],
            "min_level" => ["numeric", "min:1", "max:20"],
            "max_level" => ["numeric", "min:1", "max:20"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("page.min", "Page must be at least 1");
        messages.insert("per_page.min", "Per page must be at least 1");
        messages.insert("per_page.max", "Per page cannot exceed 100");
        messages.insert("sort_by.in", "Sort by must be one of: name, code, level, created_at, updated_at");
        messages.insert("sort_direction.in", "Sort direction must be either asc or desc");
        messages.insert("is_active.boolean", "Active status must be true or false");
        messages.insert("min_level.min", "Minimum level must be at least 1");
        messages.insert("min_level.max", "Minimum level cannot exceed 20");
        messages.insert("max_level.min", "Maximum level must be at least 1");
        messages.insert("max_level.max", "Maximum level cannot exceed 20");
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
            self.sort_by = Some("level".to_string());
        }
        if self.sort_direction.is_none() {
            self.sort_direction = Some("asc".to_string());
        }
    }
}

impl_form_request_extractor!(IndexOrganizationPositionLevelRequest);