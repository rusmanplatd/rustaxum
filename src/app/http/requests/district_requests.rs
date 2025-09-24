use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use utoipa::ToSchema;

use crate::app::http::form_request::FormRequest;
use crate::app::validation::ValidationRules;
use crate::validation_rules;
use crate::impl_form_request_extractor;

/// Request payload for creating a new district
/// Contains all required and optional fields for district creation
/// @example {"city_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV", "name": "Downtown", "code": "DT"}
#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateDistrictRequest {
    /// City ID this district belongs to
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub city_id: String,
    /// District name (2-100 characters)
    #[schema(example = "Downtown")]
    pub name: String,
    /// Optional district code (2-10 characters)
    #[schema(example = "DT")]
    pub code: Option<String>,
}

#[async_trait]
impl FormRequest for CreateDistrictRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "city_id" => ["required", "string", "ulid"],
            "name" => ["required", "string", "min:2", "max:100"],
            "code" => ["string", "min:2", "max:10", "regex:^[A-Z0-9]+$"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("city_id.required", "City ID is required");
        messages.insert("city_id.ulid", "City ID must be a valid ULID format");
        messages.insert("name.required", "District name is required");
        messages.insert("name.min", "District name must be at least 2 characters");
        messages.insert("name.max", "District name cannot exceed 100 characters");
        messages.insert("code.min", "District code must be at least 2 characters");
        messages.insert("code.max", "District code cannot exceed 10 characters");
        messages.insert("code.regex", "District code must contain only uppercase letters and numbers");
        messages
    }
}

impl_form_request_extractor!(CreateDistrictRequest);

/// Request payload for updating an existing district
/// All fields are optional for partial updates
/// @example {"name": "Updated Downtown", "code": "UDT"}
#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateDistrictRequest {
    /// Updated city ID
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub city_id: Option<String>,
    /// Updated district name (2-100 characters)
    #[schema(example = "Updated Downtown")]
    pub name: Option<String>,
    /// Updated district code (2-10 characters)
    #[schema(example = "UDT")]
    pub code: Option<String>,
}

#[async_trait]
impl FormRequest for UpdateDistrictRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "city_id" => ["string", "ulid"],
            "name" => ["string", "min:2", "max:100"],
            "code" => ["string", "min:2", "max:10", "regex:^[A-Z0-9]+$"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("city_id.ulid", "City ID must be a valid ULID format");
        messages.insert("name.min", "District name must be at least 2 characters");
        messages.insert("name.max", "District name cannot exceed 100 characters");
        messages.insert("code.min", "District code must be at least 2 characters");
        messages.insert("code.max", "District code cannot exceed 10 characters");
        messages.insert("code.regex", "District code must contain only uppercase letters and numbers");
        messages
    }
}

impl_form_request_extractor!(UpdateDistrictRequest);