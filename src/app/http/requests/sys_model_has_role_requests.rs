use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use utoipa::ToSchema;

use crate::app::http::form_request::FormRequest;
use crate::app::validation::ValidationRules;
use crate::validation_rules;
use crate::impl_form_request_extractor;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateSysModelHasRoleRequest {
    #[schema(example = "User")]
    pub model_type: String,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub model_id: String,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub role_id: String,
    #[schema(example = "organization")]
    pub scope_type: Option<String>,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub scope_id: Option<String>,
}

#[async_trait]
impl FormRequest for CreateSysModelHasRoleRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "model_type" => ["required", "string"],
            "model_id" => ["required", "ulid_format"],
            "role_id" => ["required", "ulid_format"],
            "scope_type" => ["string"],
            "scope_id" => ["ulid_format"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("model_type.required", "Model type is required");
        messages.insert("model_id.required", "Model ID is required");
        messages.insert("model_id.ulid_format", "Model ID must be a valid ULID");
        messages.insert("role_id.required", "Role ID is required");
        messages.insert("role_id.ulid_format", "Role ID must be a valid ULID");
        messages.insert("scope_id.ulid_format", "Scope ID must be a valid ULID");
        messages
    }

    fn prepare_for_validation(&mut self) {
        self.model_type = self.model_type.trim().to_string();
        if let Some(ref mut scope_type) = self.scope_type {
            *scope_type = scope_type.trim().to_string();
            if scope_type.is_empty() {
                self.scope_type = None;
            }
        }
    }
}

impl_form_request_extractor!(CreateSysModelHasRoleRequest);

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateSysModelHasRoleRequest {
    #[schema(example = "User")]
    pub model_type: Option<String>,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub model_id: Option<String>,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub role_id: Option<String>,
    #[schema(example = "organization")]
    pub scope_type: Option<String>,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub scope_id: Option<String>,
}

#[async_trait]
impl FormRequest for UpdateSysModelHasRoleRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "model_type" => ["string"],
            "model_id" => ["ulid_format"],
            "role_id" => ["ulid_format"],
            "scope_type" => ["string"],
            "scope_id" => ["ulid_format"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("model_id.ulid_format", "Model ID must be a valid ULID");
        messages.insert("role_id.ulid_format", "Role ID must be a valid ULID");
        messages.insert("scope_id.ulid_format", "Scope ID must be a valid ULID");
        messages
    }

    fn prepare_for_validation(&mut self) {
        if let Some(ref mut model_type) = self.model_type {
            *model_type = model_type.trim().to_string();
            if model_type.is_empty() {
                self.model_type = None;
            }
        }
        if let Some(ref mut scope_type) = self.scope_type {
            *scope_type = scope_type.trim().to_string();
            if scope_type.is_empty() {
                self.scope_type = None;
            }
        }
    }
}

impl_form_request_extractor!(UpdateSysModelHasRoleRequest);