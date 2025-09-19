use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use utoipa::ToSchema;

use crate::app::http::form_request::FormRequest;
use crate::app::validation::{Rule, required, string, min, max, in_list, ulid_format};
use crate::impl_form_request_extractor;

/// Create job position form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateJobPositionRequest {
    /// Job position name (2-100 characters)
    #[schema(example = "Senior Software Engineer")]
    pub name: String,
    /// Job position code (optional, 2-20 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "SSE001")]
    pub code: Option<String>,
    /// Job level ID (ULID format)
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub job_level_id: String,
    /// Job position description (optional, max 500 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Senior software engineer responsible for system architecture")]
    pub description: Option<String>,
}

#[async_trait]
impl FormRequest for CreateJobPositionRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("name", vec![required(), string(), min(2), max(100)]);
        rules.insert("code", vec![string(), min(2), max(20)]);
        rules.insert("job_level_id", vec![required(), string(), ulid_format()]);
        rules.insert("description", vec![string(), max(500)]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("name.required", "Job position name is required");
        messages.insert("name.min", "Job position name must be at least 2 characters");
        messages.insert("name.max", "Job position name cannot exceed 100 characters");
        messages.insert("code.min", "Job position code must be at least 2 characters");
        messages.insert("code.max", "Job position code cannot exceed 20 characters");
        messages.insert("job_level_id.required", "Job level ID is required");
        messages.insert("job_level_id.ulid_format", "Job level ID must be a valid ULID");
        messages.insert("description.max", "Description cannot exceed 500 characters");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("name", "job position name");
        attributes.insert("code", "job position code");
        attributes.insert("job_level_id", "job level");
        attributes.insert("description", "job position description");
        attributes
    }
}

impl_form_request_extractor!(CreateJobPositionRequest);

/// Update job position form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateJobPositionRequest {
    /// Job position name (optional, 2-100 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Senior Software Engineer")]
    pub name: Option<String>,
    /// Job position code (optional, 2-20 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "SSE001")]
    pub code: Option<String>,
    /// Job level ID (optional, ULID format)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub job_level_id: Option<String>,
    /// Job position description (optional, max 500 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Senior software engineer responsible for system architecture")]
    pub description: Option<String>,
    /// Active status
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = true)]
    pub is_active: Option<bool>,
}

#[async_trait]
impl FormRequest for UpdateJobPositionRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("name", vec![string(), min(2), max(100)]);
        rules.insert("code", vec![string(), min(2), max(20)]);
        rules.insert("job_level_id", vec![string(), ulid_format()]);
        rules.insert("description", vec![string(), max(500)]);
        rules.insert("is_active", vec![crate::app::validation::boolean()]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("name.min", "Job position name must be at least 2 characters");
        messages.insert("name.max", "Job position name cannot exceed 100 characters");
        messages.insert("code.min", "Job position code must be at least 2 characters");
        messages.insert("code.max", "Job position code cannot exceed 20 characters");
        messages.insert("job_level_id.ulid_format", "Job level ID must be a valid ULID");
        messages.insert("description.max", "Description cannot exceed 500 characters");
        messages.insert("is_active.boolean", "Active status must be true or false");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("name", "job position name");
        attributes.insert("code", "job position code");
        attributes.insert("job_level_id", "job level");
        attributes.insert("description", "job position description");
        attributes.insert("is_active", "active status");
        attributes
    }
}

impl_form_request_extractor!(UpdateJobPositionRequest);

/// Index/list job positions form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct IndexJobPositionRequest {
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
    /// Filter by job level ID (ULID format)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub job_level_id: Option<String>,
    /// Filter by name (partial match)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "engineer")]
    pub name_search: Option<String>,
}

#[async_trait]
impl FormRequest for IndexJobPositionRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("page", vec![crate::app::validation::numeric(), min(1)]);
        rules.insert("per_page", vec![crate::app::validation::numeric(), min(1), max(100)]);
        rules.insert("sort_by", vec![string(), in_list(vec!["name", "code", "created_at", "updated_at"])]);
        rules.insert("sort_direction", vec![string(), in_list(vec!["asc", "desc"])]);
        rules.insert("is_active", vec![crate::app::validation::boolean()]);
        rules.insert("job_level_id", vec![string(), ulid_format()]);
        rules.insert("name_search", vec![string(), min(2), max(100)]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("page.min", "Page must be at least 1");
        messages.insert("per_page.min", "Per page must be at least 1");
        messages.insert("per_page.max", "Per page cannot exceed 100");
        messages.insert("sort_by.in", "Sort by must be one of: name, code, created_at, updated_at");
        messages.insert("sort_direction.in", "Sort direction must be either asc or desc");
        messages.insert("is_active.boolean", "Active status must be true or false");
        messages.insert("job_level_id.ulid_format", "Job level ID must be a valid ULID");
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

impl_form_request_extractor!(IndexJobPositionRequest);

/// Job positions by job level form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct JobPositionsByLevelRequest {
    /// Job level ID (ULID format)
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub job_level_id: String,
    /// Include inactive positions
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = false)]
    pub include_inactive: Option<bool>,
}

#[async_trait]
impl FormRequest for JobPositionsByLevelRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("job_level_id", vec![required(), string(), ulid_format()]);
        rules.insert("include_inactive", vec![crate::app::validation::boolean()]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("job_level_id.required", "Job level ID is required");
        messages.insert("job_level_id.ulid_format", "Job level ID must be a valid ULID");
        messages.insert("include_inactive.boolean", "Include inactive must be true or false");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("job_level_id", "job level");
        attributes.insert("include_inactive", "include inactive flag");
        attributes
    }

    fn prepare_for_validation(&mut self) {
        if self.include_inactive.is_none() {
            self.include_inactive = Some(false);
        }
    }
}

impl_form_request_extractor!(JobPositionsByLevelRequest);