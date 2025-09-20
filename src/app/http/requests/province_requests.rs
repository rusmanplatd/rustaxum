use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use utoipa::ToSchema;

use crate::app::http::form_request::FormRequest;
use crate::app::validation::ValidationRules;
use crate::validation_rules;
use crate::impl_form_request_extractor;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateProvinceRequest {
    /// Country ID that this province belongs to
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub country_id: String,
    /// Province name (2-100 characters)
    #[schema(example = "California")]
    pub name: String,
    /// Optional province code
    #[schema(example = "CA")]
    pub code: Option<String>,
}

#[async_trait]
impl FormRequest for CreateProvinceRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "country_id" => ["required", "string", "uuid"],
            "name" => ["required", "string", "min:2", "max:100"],
            "code" => ["string", "max:10"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("country_id.required", "Country ID is required");
        messages.insert("country_id.uuid", "Country ID must be a valid UUID");
        messages.insert("name.required", "Province name is required");
        messages.insert("name.min", "Province name must be at least 2 characters");
        messages.insert("name.max", "Province name cannot exceed 100 characters");
        messages.insert("code.max", "Province code cannot exceed 10 characters");
        messages
    }

    fn prepare_for_validation(&mut self) {
        self.country_id = self.country_id.trim().to_string();
        self.name = self.name.trim().to_string();
        if let Some(ref mut code) = self.code {
            *code = code.trim().to_string();
            if code.is_empty() {
                self.code = None;
            }
        }
    }
}

impl_form_request_extractor!(CreateProvinceRequest);

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateProvinceRequest {
    /// Updated country ID
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub country_id: Option<String>,
    /// Updated province name
    #[schema(example = "California")]
    pub name: Option<String>,
    /// Updated province code
    #[schema(example = "CA")]
    pub code: Option<String>,
}

#[async_trait]
impl FormRequest for UpdateProvinceRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "country_id" => ["string", "uuid"],
            "name" => ["string", "min:2", "max:100"],
            "code" => ["string", "max:10"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("country_id.uuid", "Country ID must be a valid UUID");
        messages.insert("name.min", "Province name must be at least 2 characters");
        messages.insert("name.max", "Province name cannot exceed 100 characters");
        messages.insert("code.max", "Province code cannot exceed 10 characters");
        messages
    }

    fn prepare_for_validation(&mut self) {
        if let Some(ref mut country_id) = self.country_id {
            *country_id = country_id.trim().to_string();
            if country_id.is_empty() {
                self.country_id = None;
            }
        }
        if let Some(ref mut name) = self.name {
            *name = name.trim().to_string();
            if name.is_empty() {
                self.name = None;
            }
        }
        if let Some(ref mut code) = self.code {
            *code = code.trim().to_string();
            if code.is_empty() {
                self.code = None;
            }
        }
    }
}

impl_form_request_extractor!(UpdateProvinceRequest);