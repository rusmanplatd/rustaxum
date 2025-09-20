use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use utoipa::ToSchema;

use crate::app::http::form_request::FormRequest;
use crate::app::validation::{Rule, required, string, numeric, min, max, in_list};
use crate::impl_form_request_extractor;

/// Create job level form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateJobLevelRequest {
    /// Job level name (2-100 characters)
    #[schema(example = "Senior Level")]
    pub name: String,
    /// Job level code (optional, 2-20 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "SL5")]
    pub code: Option<String>,
    /// Numeric level for hierarchy (1-20)
    #[schema(example = 5)]
    pub level: i32,
    /// Job level description (optional, max 500 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Senior level position with 5+ years experience")]
    pub description: Option<String>,
}

#[async_trait]
impl FormRequest for CreateJobLevelRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("name", vec![required(), string(), min(2), max(100)]);
        rules.insert("code", vec![string(), min(2), max(20)]);
        rules.insert("level", vec![required(), numeric(), min(1), max(20)]);
        rules.insert("description", vec![string(), max(500)]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("name.required", "Job level name is required");
        messages.insert("name.min", "Job level name must be at least 2 characters");
        messages.insert("name.max", "Job level name cannot exceed 100 characters");
        messages.insert("code.min", "Job level code must be at least 2 characters");
        messages.insert("code.max", "Job level code cannot exceed 20 characters");
        messages.insert("level.required", "Job level numeric value is required");
        messages.insert("level.min", "Job level must be at least 1");
        messages.insert("level.max", "Job level cannot exceed 20");
        messages.insert("description.max", "Description cannot exceed 500 characters");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("name", "job level name");
        attributes.insert("code", "job level code");
        attributes.insert("level", "level number");
        attributes.insert("description", "job level description");
        attributes
    }
}

impl_form_request_extractor!(CreateJobLevelRequest);

/// Update job level form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateJobLevelRequest {
    /// Job level name (optional, 2-100 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Senior Level")]
    pub name: Option<String>,
    /// Job level code (optional, 2-20 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "SL5")]
    pub code: Option<String>,
    /// Numeric level for hierarchy (optional, 1-20)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 5)]
    pub level: Option<i32>,
    /// Job level description (optional, max 500 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Senior level position with 5+ years experience")]
    pub description: Option<String>,
    /// Active status
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = true)]
    pub is_active: Option<bool>,
}

#[async_trait]
impl FormRequest for UpdateJobLevelRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("name", vec![string(), min(2), max(100)]);
        rules.insert("code", vec![string(), min(2), max(20)]);
        rules.insert("level", vec![numeric(), min(1), max(20)]);
        rules.insert("description", vec![string(), max(500)]);
        rules.insert("is_active", vec![crate::app::validation::boolean()]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("name.min", "Job level name must be at least 2 characters");
        messages.insert("name.max", "Job level name cannot exceed 100 characters");
        messages.insert("code.min", "Job level code must be at least 2 characters");
        messages.insert("code.max", "Job level code cannot exceed 20 characters");
        messages.insert("level.min", "Job level must be at least 1");
        messages.insert("level.max", "Job level cannot exceed 20");
        messages.insert("description.max", "Description cannot exceed 500 characters");
        messages.insert("is_active.boolean", "Active status must be true or false");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("name", "job level name");
        attributes.insert("code", "job level code");
        attributes.insert("level", "level number");
        attributes.insert("description", "job level description");
        attributes.insert("is_active", "active status");
        attributes
    }
}

impl_form_request_extractor!(UpdateJobLevelRequest);

/// Index/list job levels form request
#[derive(Deserialize, Serialize, ToSchema, Clone)]
pub struct IndexJobLevelRequest {
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
impl FormRequest for IndexJobLevelRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("page", vec![numeric(), min(1)]);
        rules.insert("per_page", vec![numeric(), min(1), max(100)]);
        rules.insert("sort_by", vec![string(), in_list(vec!["name", "code", "level", "created_at", "updated_at"])]);
        rules.insert("sort_direction", vec![string(), in_list(vec!["asc", "desc"])]);
        rules.insert("is_active", vec![crate::app::validation::boolean()]);
        rules.insert("min_level", vec![numeric(), min(1), max(20)]);
        rules.insert("max_level", vec![numeric(), min(1), max(20)]);
        rules
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

impl_form_request_extractor!(IndexJobLevelRequest);