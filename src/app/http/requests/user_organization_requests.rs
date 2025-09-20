use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use utoipa::ToSchema;
use chrono::{DateTime, Utc};

use crate::app::http::form_request::FormRequest;
use crate::app::validation::ValidationRules;
use crate::validation_rules;
use crate::impl_form_request_extractor;

/// Create user organization relationship form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateUserOrganizationRequest {
    /// User ID (ULID format)
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub user_id: String,
    /// Organization ID (ULID format)
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_id: String,
    /// Job Position ID (ULID format)
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub job_position_id: String,
    /// Start date of the relationship (defaults to current time)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub started_at: Option<DateTime<Utc>>,
}

#[async_trait]
impl FormRequest for CreateUserOrganizationRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "user_id" => ["required", "string", "ulid_format"],
            "organization_id" => ["required", "string", "ulid_format"],
            "job_position_id" => ["required", "string", "ulid_format"],
            "started_at" => ["date"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("user_id.required", "User ID is required");
        messages.insert("user_id.ulid_format", "User ID must be a valid ULID");
        messages.insert("organization_id.required", "Organization ID is required");
        messages.insert("organization_id.ulid_format", "Organization ID must be a valid ULID");
        messages.insert("job_position_id.required", "Job Position ID is required");
        messages.insert("job_position_id.ulid_format", "Job Position ID must be a valid ULID");
        messages.insert("started_at.date", "Started at must be a valid date");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("user_id", "user");
        attributes.insert("organization_id", "organization");
        attributes.insert("job_position_id", "job position");
        attributes.insert("started_at", "start date");
        attributes
    }
}

impl_form_request_extractor!(CreateUserOrganizationRequest);

/// Update user organization relationship form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateUserOrganizationRequest {
    /// Organization ID (ULID format) - optional for updates
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_id: Option<String>,
    /// Job Position ID (ULID format) - optional for updates
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub job_position_id: Option<String>,
    /// Active status of the relationship
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = true)]
    pub is_active: Option<bool>,
    /// Start date of the relationship
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub started_at: Option<DateTime<Utc>>,
    /// End date of the relationship
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "2024-12-31T23:59:59Z")]
    pub ended_at: Option<DateTime<Utc>>,
}

#[async_trait]
impl FormRequest for UpdateUserOrganizationRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "organization_id" => ["string", "ulid_format"],
            "job_position_id" => ["string", "ulid_format"],
            "is_active" => ["boolean"],
            "started_at" => ["date"],
            "ended_at" => ["date"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("organization_id.ulid_format", "Organization ID must be a valid ULID");
        messages.insert("job_position_id.ulid_format", "Job Position ID must be a valid ULID");
        messages.insert("is_active.boolean", "Active status must be true or false");
        messages.insert("started_at.date", "Started at must be a valid date");
        messages.insert("ended_at.date", "Ended at must be a valid date");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("organization_id", "organization");
        attributes.insert("job_position_id", "job position");
        attributes.insert("is_active", "active status");
        attributes.insert("started_at", "start date");
        attributes.insert("ended_at", "end date");
        attributes
    }

    fn prepare_for_validation(&mut self) {
        // Validation logic: if ended_at is set, is_active should be false
        if self.ended_at.is_some() && self.is_active.unwrap_or(true) {
            self.is_active = Some(false);
        }
    }
}

impl_form_request_extractor!(UpdateUserOrganizationRequest);

/// Index/list user organization relationships form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct IndexUserOrganizationRequest {
    /// Page number (default: 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 1)]
    pub page: Option<u32>,
    /// Items per page (1-100, default: 15)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 15)]
    pub per_page: Option<u32>,
    /// Sort field (created_at, updated_at, started_at, ended_at)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "created_at")]
    pub sort_by: Option<String>,
    /// Sort direction (asc, desc)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "desc")]
    pub sort_direction: Option<String>,
    /// Filter by user ID (ULID format)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub user_id: Option<String>,
    /// Filter by organization ID (ULID format)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_id: Option<String>,
    /// Filter by job position ID (ULID format)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub job_position_id: Option<String>,
    /// Filter by active status
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = true)]
    pub is_active: Option<bool>,
    /// Filter by organization type
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "department")]
    pub organization_type: Option<String>,
}

#[async_trait]
impl FormRequest for IndexUserOrganizationRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "page" => ["numeric", "min:1"],
            "per_page" => ["numeric", "min:1", "max:100"],
            "sort_by" => ["string", "in:created_at,updated_at,started_at,ended_at,user_id,organization_id"],
            "sort_direction" => ["string", "in:asc,desc"],
            "user_id" => ["string", "ulid_format"],
            "organization_id" => ["string", "ulid_format"],
            "job_position_id" => ["string", "ulid_format"],
            "is_active" => ["boolean"],
            "organization_type" => ["string", "in:company,boc,bod,division,department,branch,subbranch,section"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("page.min", "Page must be at least 1");
        messages.insert("per_page.min", "Per page must be at least 1");
        messages.insert("per_page.max", "Per page cannot exceed 100");
        messages.insert("sort_by.in", "Sort by must be one of: created_at, updated_at, started_at, ended_at, user_id, organization_id");
        messages.insert("sort_direction.in", "Sort direction must be either asc or desc");
        messages.insert("user_id.ulid_format", "User ID must be a valid ULID");
        messages.insert("organization_id.ulid_format", "Organization ID must be a valid ULID");
        messages.insert("job_position_id.ulid_format", "Job Position ID must be a valid ULID");
        messages.insert("is_active.boolean", "Active status must be true or false");
        messages.insert("organization_type.in", "Organization type must be one of: company, boc, bod, division, department, branch, subbranch, section");
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
            self.sort_by = Some("created_at".to_string());
        }
        if self.sort_direction.is_none() {
            self.sort_direction = Some("desc".to_string());
        }
    }
}

impl_form_request_extractor!(IndexUserOrganizationRequest);

/// Transfer user organization form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct TransferUserOrganizationRequest {
    /// New Organization ID (ULID format)
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_id: String,
    /// New Job Position ID (ULID format)
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub job_position_id: String,
    /// Transfer date (defaults to current time)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub transfer_date: Option<DateTime<Utc>>,
    /// Transfer reason
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Promotion")]
    pub reason: Option<String>,
}

#[async_trait]
impl FormRequest for TransferUserOrganizationRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "organization_id" => ["required", "string", "ulid_format"],
            "job_position_id" => ["required", "string", "ulid_format"],
            "transfer_date" => ["date"],
            "reason" => ["string", "max:500"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("organization_id.required", "New organization ID is required");
        messages.insert("organization_id.ulid_format", "Organization ID must be a valid ULID");
        messages.insert("job_position_id.required", "New job position ID is required");
        messages.insert("job_position_id.ulid_format", "Job Position ID must be a valid ULID");
        messages.insert("transfer_date.date", "Transfer date must be a valid date");
        messages.insert("reason.max", "Transfer reason cannot exceed 500 characters");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("organization_id", "new organization");
        attributes.insert("job_position_id", "new job position");
        attributes.insert("transfer_date", "transfer date");
        attributes.insert("reason", "transfer reason");
        attributes
    }
}

impl_form_request_extractor!(TransferUserOrganizationRequest);

/// Assign role to user organization form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct AssignRoleRequest {
    /// Role ID (ULID format)
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub role_id: String,
}

#[async_trait]
impl FormRequest for AssignRoleRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "role_id" => ["required", "string", "ulid_format"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("role_id.required", "Role ID is required");
        messages.insert("role_id.ulid_format", "Role ID must be a valid ULID");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("role_id", "role");
        attributes
    }
}

impl_form_request_extractor!(AssignRoleRequest);

/// Remove role from user organization form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct RemoveRoleRequest {
    /// Role ID (ULID format)
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub role_id: String,
}

#[async_trait]
impl FormRequest for RemoveRoleRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "role_id" => ["required", "string", "ulid_format"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("role_id.required", "Role ID is required");
        messages.insert("role_id.ulid_format", "Role ID must be a valid ULID");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("role_id", "role");
        attributes
    }
}

impl_form_request_extractor!(RemoveRoleRequest);