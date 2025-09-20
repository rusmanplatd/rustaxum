use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use utoipa::ToSchema;
use ulid::Ulid;

use crate::app::http::form_request::FormRequest;
use crate::app::utils::validator::{Rule, required, string, ulid_format};
use crate::impl_form_request_extractor;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateSysModelHasPermissionRequest {
    #[schema(example = "User")]
    pub model_type: String,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub model_id: Ulid,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub permission_id: Ulid,
    #[schema(example = "organization")]
    pub scope_type: Option<String>,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub scope_id: Option<Ulid>,
}

#[async_trait]
impl FormRequest for CreateSysModelHasPermissionRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("model_type", vec![required(), string()]);
        rules.insert("model_id", vec![required(), ulid_format()]);
        rules.insert("permission_id", vec![required(), ulid_format()]);
        rules.insert("scope_type", vec![string()]);
        rules.insert("scope_id", vec![ulid_format()]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("model_type.required", "Model type is required");
        messages.insert("model_id.required", "Model ID is required");
        messages.insert("model_id.ulid_format", "Model ID must be a valid ULID");
        messages.insert("permission_id.required", "Permission ID is required");
        messages.insert("permission_id.ulid_format", "Permission ID must be a valid ULID");
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

impl_form_request_extractor!(CreateSysModelHasPermissionRequest);

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateSysModelHasPermissionRequest {
    #[schema(example = "User")]
    pub model_type: Option<String>,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub model_id: Option<Ulid>,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub permission_id: Option<Ulid>,
    #[schema(example = "organization")]
    pub scope_type: Option<String>,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub scope_id: Option<Ulid>,
}

#[async_trait]
impl FormRequest for UpdateSysModelHasPermissionRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("model_type", vec![string()]);
        rules.insert("model_id", vec![ulid_format()]);
        rules.insert("permission_id", vec![ulid_format()]);
        rules.insert("scope_type", vec![string()]);
        rules.insert("scope_id", vec![ulid_format()]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("model_id.ulid_format", "Model ID must be a valid ULID");
        messages.insert("permission_id.ulid_format", "Permission ID must be a valid ULID");
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

impl_form_request_extractor!(UpdateSysModelHasPermissionRequest);