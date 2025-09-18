use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::app::http::form_request::FormRequest;
use crate::app::utils::validator::{Rule, required, string, min, max, uuid, numeric, between};
use crate::impl_form_request_extractor;

#[derive(Deserialize, Serialize)]
pub struct CreateCityRequest {
    pub province_id: String,
    pub name: String,
    pub code: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
}

#[async_trait]
impl FormRequest for CreateCityRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("province_id", vec![required(), string(), uuid()]);
        rules.insert("name", vec![required(), string(), min("2"), max("100")]);
        rules.insert("code", vec![string(), max("10")]);
        rules.insert("latitude", vec![numeric(), between("-90", "90")]);
        rules.insert("longitude", vec![numeric(), between("-180", "180")]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("province_id.required", "Province ID is required");
        messages.insert("province_id.uuid", "Province ID must be a valid UUID");
        messages.insert("name.required", "City name is required");
        messages.insert("name.min", "City name must be at least 2 characters");
        messages.insert("name.max", "City name cannot exceed 100 characters");
        messages.insert("code.max", "City code cannot exceed 10 characters");
        messages.insert("latitude.numeric", "Latitude must be a valid number");
        messages.insert("latitude.between", "Latitude must be between -90 and 90 degrees");
        messages.insert("longitude.numeric", "Longitude must be a valid number");
        messages.insert("longitude.between", "Longitude must be between -180 and 180 degrees");
        messages
    }

    fn prepare_for_validation(&mut self) {
        self.province_id = self.province_id.trim().to_string();
        self.name = self.name.trim().to_string();
        if let Some(ref mut code) = self.code {
            *code = code.trim().to_string();
            if code.is_empty() {
                self.code = None;
            }
        }
    }
}

impl_form_request_extractor!(CreateCityRequest);

#[derive(Deserialize, Serialize)]
pub struct UpdateCityRequest {
    pub province_id: Option<String>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
}

#[async_trait]
impl FormRequest for UpdateCityRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("province_id", vec![string(), uuid()]);
        rules.insert("name", vec![string(), min("2"), max("100")]);
        rules.insert("code", vec![string(), max("10")]);
        rules.insert("latitude", vec![numeric(), between("-90", "90")]);
        rules.insert("longitude", vec![numeric(), between("-180", "180")]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("province_id.uuid", "Province ID must be a valid UUID");
        messages.insert("name.min", "City name must be at least 2 characters");
        messages.insert("name.max", "City name cannot exceed 100 characters");
        messages.insert("code.max", "City code cannot exceed 10 characters");
        messages.insert("latitude.numeric", "Latitude must be a valid number");
        messages.insert("latitude.between", "Latitude must be between -90 and 90 degrees");
        messages.insert("longitude.numeric", "Longitude must be a valid number");
        messages.insert("longitude.between", "Longitude must be between -180 and 180 degrees");
        messages
    }

    fn prepare_for_validation(&mut self) {
        if let Some(ref mut province_id) = self.province_id {
            *province_id = province_id.trim().to_string();
            if province_id.is_empty() {
                self.province_id = None;
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

impl_form_request_extractor!(UpdateCityRequest);