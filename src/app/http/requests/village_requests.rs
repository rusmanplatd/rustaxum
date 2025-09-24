use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use utoipa::ToSchema;
use rust_decimal::Decimal;

use crate::app::http::form_request::FormRequest;
use crate::app::validation::ValidationRules;
use crate::validation_rules;
use crate::impl_form_request_extractor;

/// Request payload for creating a new village
/// Contains all required and optional fields for village creation
/// @example {"district_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV", "name": "Green Valley", "code": "GV", "latitude": 40.7128, "longitude": -74.0060}
#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateVillageRequest {
    /// District ID this village belongs to
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub district_id: String,
    /// Village name (2-100 characters)
    #[schema(example = "Green Valley")]
    pub name: String,
    /// Optional village code (2-10 characters)
    #[schema(example = "GV")]
    pub code: Option<String>,
    /// Optional latitude coordinate
    #[schema(value_type = Option<f64>, example = 40.7128)]
    pub latitude: Option<Decimal>,
    /// Optional longitude coordinate
    #[schema(value_type = Option<f64>, example = -74.0060)]
    pub longitude: Option<Decimal>,
}

#[async_trait]
impl FormRequest for CreateVillageRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "district_id" => ["required", "string", "ulid"],
            "name" => ["required", "string", "min:2", "max:100"],
            "code" => ["string", "min:2", "max:10", "regex:^[A-Z0-9]+$"],
            "latitude" => ["numeric", "min:-90", "max:90"],
            "longitude" => ["numeric", "min:-180", "max:180"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("district_id.required", "District ID is required");
        messages.insert("district_id.ulid", "District ID must be a valid ULID format");
        messages.insert("name.required", "Village name is required");
        messages.insert("name.min", "Village name must be at least 2 characters");
        messages.insert("name.max", "Village name cannot exceed 100 characters");
        messages.insert("code.min", "Village code must be at least 2 characters");
        messages.insert("code.max", "Village code cannot exceed 10 characters");
        messages.insert("code.regex", "Village code must contain only uppercase letters and numbers");
        messages.insert("latitude.numeric", "Latitude must be a valid number");
        messages.insert("latitude.min", "Latitude must be between -90 and 90 degrees");
        messages.insert("latitude.max", "Latitude must be between -90 and 90 degrees");
        messages.insert("longitude.numeric", "Longitude must be a valid number");
        messages.insert("longitude.min", "Longitude must be between -180 and 180 degrees");
        messages.insert("longitude.max", "Longitude must be between -180 and 180 degrees");
        messages
    }
}

impl_form_request_extractor!(CreateVillageRequest);

/// Request payload for updating an existing village
/// All fields are optional for partial updates
/// @example {"name": "Updated Green Valley", "code": "UGV", "latitude": 40.7500, "longitude": -74.0100}
#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateVillageRequest {
    /// Updated district ID
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub district_id: Option<String>,
    /// Updated village name (2-100 characters)
    #[schema(example = "Updated Green Valley")]
    pub name: Option<String>,
    /// Updated village code (2-10 characters)
    #[schema(example = "UGV")]
    pub code: Option<String>,
    /// Updated latitude coordinate
    #[schema(value_type = Option<f64>, example = 40.7500)]
    pub latitude: Option<Decimal>,
    /// Updated longitude coordinate
    #[schema(value_type = Option<f64>, example = -74.0100)]
    pub longitude: Option<Decimal>,
}

#[async_trait]
impl FormRequest for UpdateVillageRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "district_id" => ["string", "ulid"],
            "name" => ["string", "min:2", "max:100"],
            "code" => ["string", "min:2", "max:10", "regex:^[A-Z0-9]+$"],
            "latitude" => ["numeric", "min:-90", "max:90"],
            "longitude" => ["numeric", "min:-180", "max:180"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("district_id.ulid", "District ID must be a valid ULID format");
        messages.insert("name.min", "Village name must be at least 2 characters");
        messages.insert("name.max", "Village name cannot exceed 100 characters");
        messages.insert("code.min", "Village code must be at least 2 characters");
        messages.insert("code.max", "Village code cannot exceed 10 characters");
        messages.insert("code.regex", "Village code must contain only uppercase letters and numbers");
        messages.insert("latitude.numeric", "Latitude must be a valid number");
        messages.insert("latitude.min", "Latitude must be between -90 and 90 degrees");
        messages.insert("latitude.max", "Latitude must be between -90 and 90 degrees");
        messages.insert("longitude.numeric", "Longitude must be a valid number");
        messages.insert("longitude.min", "Longitude must be between -180 and 180 degrees");
        messages.insert("longitude.max", "Longitude must be between -180 and 180 degrees");
        messages
    }
}

impl_form_request_extractor!(UpdateVillageRequest);