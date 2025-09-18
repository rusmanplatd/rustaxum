use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use crate::app::http::form_request::FormRequest;
use crate::app::utils::validator::{Rule, required, string, min, max, regex};
use crate::impl_form_request_extractor;

#[derive(Deserialize, Serialize)]
pub struct CreateCountryRequest {
    pub name: String,
    pub iso_code: String,
    pub phone_code: Option<String>,
}

#[async_trait]
impl FormRequest for CreateCountryRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("name", vec![required(), string(), min("2"), max("100")]);
        rules.insert("iso_code", vec![required(), string(), min("2"), max("3"), regex("^[A-Z]{2,3}$")]);
        rules.insert("phone_code", vec![string(), max("10"), regex("^\\+?[0-9]+$")]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("name.required", "Country name is required");
        messages.insert("name.min", "Country name must be at least 2 characters");
        messages.insert("name.max", "Country name cannot exceed 100 characters");
        messages.insert("iso_code.required", "ISO code is required");
        messages.insert("iso_code.min", "ISO code must be at least 2 characters");
        messages.insert("iso_code.max", "ISO code cannot exceed 3 characters");
        messages.insert("iso_code.regex", "ISO code must contain only uppercase letters");
        messages.insert("phone_code.max", "Phone code cannot exceed 10 characters");
        messages.insert("phone_code.regex", "Phone code must contain only numbers and optional + prefix");
        messages
    }

    fn prepare_for_validation(&mut self) {
        self.name = self.name.trim().to_string();
        self.iso_code = self.iso_code.trim().to_uppercase();
        if let Some(ref mut phone_code) = self.phone_code {
            *phone_code = phone_code.trim().to_string();
            if phone_code.is_empty() {
                self.phone_code = None;
            }
        }
    }
}

impl_form_request_extractor!(CreateCountryRequest);

#[derive(Deserialize, Serialize)]
pub struct UpdateCountryRequest {
    pub name: Option<String>,
    pub iso_code: Option<String>,
    pub phone_code: Option<String>,
}

#[async_trait]
impl FormRequest for UpdateCountryRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("name", vec![string(), min("2"), max("100")]);
        rules.insert("iso_code", vec![string(), min("2"), max("3"), regex("^[A-Z]{2,3}$")]);
        rules.insert("phone_code", vec![string(), max("10"), regex("^\\+?[0-9]+$")]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("name.min", "Country name must be at least 2 characters");
        messages.insert("name.max", "Country name cannot exceed 100 characters");
        messages.insert("iso_code.min", "ISO code must be at least 2 characters");
        messages.insert("iso_code.max", "ISO code cannot exceed 3 characters");
        messages.insert("iso_code.regex", "ISO code must contain only uppercase letters");
        messages.insert("phone_code.max", "Phone code cannot exceed 10 characters");
        messages.insert("phone_code.regex", "Phone code must contain only numbers and optional + prefix");
        messages
    }

    fn prepare_for_validation(&mut self) {
        if let Some(ref mut name) = self.name {
            *name = name.trim().to_string();
            if name.is_empty() {
                self.name = None;
            }
        }
        if let Some(ref mut iso_code) = self.iso_code {
            *iso_code = iso_code.trim().to_uppercase();
            if iso_code.is_empty() {
                self.iso_code = None;
            }
        }
        if let Some(ref mut phone_code) = self.phone_code {
            *phone_code = phone_code.trim().to_string();
            if phone_code.is_empty() {
                self.phone_code = None;
            }
        }
    }
}

impl_form_request_extractor!(UpdateCountryRequest);