use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use crate::app::http::form_request::FormRequest;
use crate::app::utils::validator::{Rule, required, string, min, max, uuid};
use crate::impl_form_request_extractor;

#[derive(Deserialize, Serialize)]
pub struct CreateProvinceRequest {
    pub country_id: String,
    pub name: String,
    pub code: Option<String>,
}

#[async_trait]
impl FormRequest for CreateProvinceRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("country_id", vec![required(), string(), uuid()]);
        rules.insert("name", vec![required(), string(), min("2"), max("100")]);
        rules.insert("code", vec![string(), max("10")]);
        rules
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

#[derive(Deserialize, Serialize)]
pub struct UpdateProvinceRequest {
    pub country_id: Option<String>,
    pub name: Option<String>,
    pub code: Option<String>,
}

#[async_trait]
impl FormRequest for UpdateProvinceRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("country_id", vec![string(), uuid()]);
        rules.insert("name", vec![string(), min("2"), max("100")]);
        rules.insert("code", vec![string(), max("10")]);
        rules
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