//! Validation helper functions and utilities
//!
//! This module provides additional helper functions for common validation patterns,
//! custom validators, and integration utilities.

use crate::app::utils::validator::{Validator, ValidationErrors, ValidationError, Rule};
use crate::{validate, validation_rules};
use std::collections::HashMap;
use serde_json::Value;
use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

/// Response format for validation errors in API responses
#[derive(Debug, Serialize)]
pub struct ValidationErrorResponse {
    pub message: String,
    pub errors: HashMap<String, Vec<String>>,
}

impl IntoResponse for ValidationErrors {
    fn into_response(self) -> Response {
        let response = ValidationErrorResponse {
            message: "The given data was invalid.".to_string(),
            errors: self.get_messages(),
        };

        (StatusCode::UNPROCESSABLE_ENTITY, Json(response)).into_response()
    }
}

/// Builder for creating validators with a fluent interface
pub struct ValidatorBuilder {
    data: HashMap<String, Value>,
    rules: HashMap<String, Vec<Rule>>,
    custom_messages: HashMap<String, String>,
}

impl ValidatorBuilder {
    pub fn new(data: HashMap<String, Value>) -> Self {
        Self {
            data,
            rules: HashMap::new(),
            custom_messages: HashMap::new(),
        }
    }

    pub fn from_json(value: Value) -> Self {
        let data = match value {
            Value::Object(map) => map.into_iter().collect(),
            _ => HashMap::new(),
        };
        Self::new(data)
    }

    pub fn rule(mut self, field: impl Into<String>, rule: Rule) -> Self {
        self.rules
            .entry(field.into())
            .or_insert_with(Vec::new)
            .push(rule);
        self
    }

    pub fn rules(mut self, field: impl Into<String>, rules: Vec<Rule>) -> Self {
        self.rules.insert(field.into(), rules);
        self
    }

    pub fn rules_from_string(mut self, field: impl Into<String>, rule_string: &str) -> Self {
        let rule_array: Vec<&str> = rule_string.split('|').collect();
        let rules = crate::app::utils::validation_macros::parse_rules(rule_array);
        self.rules.insert(field.into(), rules);
        self
    }

    pub fn rules_from_array<T: AsRef<str>>(mut self, field: impl Into<String>, rules_array: Vec<T>) -> Self {
        let rules = crate::app::utils::validation_macros::parse_rules(rules_array);
        self.rules.insert(field.into(), rules);
        self
    }

    pub fn message(mut self, key: impl Into<String>, message: impl Into<String>) -> Self {
        self.custom_messages.insert(key.into(), message.into());
        self
    }

    pub fn messages(mut self, messages: HashMap<String, String>) -> Self {
        self.custom_messages.extend(messages);
        self
    }

    pub fn build(self) -> Validator {
        let mut validator = Validator::make(self.data, self.rules);
        validator.messages(self.custom_messages);
        validator
    }

    pub fn validate(self) -> Result<(), ValidationErrors> {
        self.build().validate()
    }
}

/// Common validation patterns for specific use cases
pub struct CommonValidators;

impl CommonValidators {
    /// Validate user registration data
    pub fn user_registration(data: Value) -> Result<(), ValidationErrors> {
        validate!(data, {
            "name" => ["required", "string", "min:2", "max:255"],
            "email" => ["required", "email", "max:255"],
            "password" => ["required", "string", "min:8", "max:128", "confirmed"]
        })
    }

    /// Validate user login data
    pub fn user_login(data: Value) -> Result<(), ValidationErrors> {
        validate!(data, {
            "email" => ["required", "email"],
            "password" => ["required", "string"]
        })
    }

    /// Validate user profile update data
    pub fn user_profile_update(data: Value) -> Result<(), ValidationErrors> {
        validate!(data, {
            "name" => ["string", "min:2", "max:255"],
            "email" => ["email", "max:255"],
            "bio" => ["string", "max:1000"],
            "website" => ["url"],
            "age" => ["integer", "min:13", "max:120"]
        })
    }

    /// Validate password change data
    pub fn password_change(data: Value) -> Result<(), ValidationErrors> {
        validate!(data, {
            "current_password" => ["required", "string"],
            "password" => ["required", "string", "min:8", "max:128", "confirmed"]
        })
    }

    /// Validate API pagination parameters
    pub fn pagination(data: Value) -> Result<(), ValidationErrors> {
        validate!(data, {
            "page" => ["integer", "min:1"],
            "per_page" => ["integer", "min:1", "max:100"],
            "sort" => ["string", "in:asc,desc"],
            "order_by" => ["string", "alpha_dash"]
        })
    }

    /// Validate contact form data
    pub fn contact_form(data: Value) -> Result<(), ValidationErrors> {
        validate!(data, {
            "name" => ["required", "string", "min:2", "max:255"],
            "email" => ["required", "email", "max:255"],
            "subject" => ["required", "string", "min:5", "max:255"],
            "message" => ["required", "string", "min:10", "max:5000"]
        })
    }

    /// Validate file upload metadata
    pub fn file_upload(data: Value) -> Result<(), ValidationErrors> {
        validate!(data, {
            "filename" => ["required", "string", "max:255"],
            "mime_type" => ["required", "string", "max:127"],
            "size" => ["required", "integer", "min:1", "max:52428800"], // 50MB max
            "description" => ["string", "max:1000"]
        })
    }
}

/// Custom validation rule definitions
pub struct CustomRules;

impl CustomRules {
    /// Password strength validation rule
    pub fn password_strength() -> Rule {
        Rule::with_params("regex", vec![
            r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]+$".to_string()
        ])
    }

    /// Phone number validation rule (basic international format)
    pub fn phone_number() -> Rule {
        Rule::with_params("regex", vec![
            r"^\+?[1-9]\d{1,14}$".to_string()
        ])
    }

    /// Username validation rule (alphanumeric, underscore, dash, 3-30 chars)
    pub fn username() -> Rule {
        Rule::with_params("regex", vec![
            r"^[a-zA-Z0-9_-]{3,30}$".to_string()
        ])
    }

    /// Hex color validation rule
    pub fn hex_color() -> Rule {
        Rule::with_params("regex", vec![
            r"^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$".to_string()
        ])
    }

    /// Credit card number validation rule (basic Luhn algorithm check would be separate)
    pub fn credit_card() -> Rule {
        Rule::with_params("regex", vec![
            r"^[0-9]{13,19}$".to_string()
        ])
    }

    /// IP address validation rule (IPv4)
    pub fn ipv4() -> Rule {
        Rule::with_params("regex", vec![
            r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$".to_string()
        ])
    }

    /// UUID validation rule
    pub fn uuid() -> Rule {
        Rule::with_params("regex", vec![
            r"^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$".to_string()
        ])
    }

    /// Slug validation rule (URL-friendly strings)
    pub fn slug() -> Rule {
        Rule::with_params("regex", vec![
            r"^[a-z0-9]+(?:-[a-z0-9]+)*$".to_string()
        ])
    }

    /// YouTube URL validation rule
    pub fn youtube_url() -> Rule {
        Rule::with_params("regex", vec![
            r"^(https?://)?(www\.)?(youtube\.com/watch\?v=|youtu\.be/)[a-zA-Z0-9_-]{11}$".to_string()
        ])
    }

    /// Social media handle validation rule (Twitter-like)
    pub fn social_handle() -> Rule {
        Rule::with_params("regex", vec![
            r"^@?[a-zA-Z0-9_]{1,15}$".to_string()
        ])
    }

    /// IBAN validation rule (International Bank Account Number)
    pub fn iban() -> Rule {
        Rule::with_params("regex", vec![
            r"^[A-Z]{2}[0-9]{2}[A-Z0-9]{4}[0-9]{7}([A-Z0-9]?){0,16}$".to_string()
        ])
    }

    /// ASCII validation rule (only ASCII characters)
    pub fn ascii() -> Rule {
        Rule::with_params("regex", vec![
            r"^[\x00-\x7F]*$".to_string()
        ])
    }

    /// Base64 validation rule
    pub fn base64() -> Rule {
        Rule::with_params("regex", vec![
            r"^[A-Za-z0-9+/]*={0,2}$".to_string()
        ])
    }

    /// JSON validation rule (basic structure check)
    pub fn json_string() -> Rule {
        Rule::new("json")
    }

    /// Decimal validation rule (with optional precision)
    pub fn decimal(min_precision: Option<u8>, max_precision: Option<u8>) -> Rule {
        let pattern = match (min_precision, max_precision) {
            (Some(min), Some(max)) => format!(r"^\d+\.\d{{{},{}}}$", min, max),
            (Some(min), None) => format!(r"^\d+\.\d{{{}}}$", min),
            (None, Some(max)) => format!(r"^\d+\.\d{{1,{}}}$", max),
            (None, None) => r"^\d+\.\d+$".to_string(),
        };
        Rule::with_params("regex", vec![pattern])
    }

    /// Lowercase validation rule
    pub fn lowercase() -> Rule {
        Rule::with_params("regex", vec![
            r"^[a-z]*$".to_string()
        ])
    }

    /// Uppercase validation rule
    pub fn uppercase() -> Rule {
        Rule::with_params("regex", vec![
            r"^[A-Z]*$".to_string()
        ])
    }

    /// Contains validation rule (field must contain specific substring)
    pub fn contains(substring: impl ToString) -> Rule {
        let pattern = regex::escape(&substring.to_string());
        Rule::with_params("regex", vec![pattern])
    }

    /// Doesn't contain validation rule (field must not contain specific substring)
    pub fn doesnt_contain(substring: impl ToString) -> Rule {
        let pattern = format!("^(?!.*{}).*$", regex::escape(&substring.to_string()));
        Rule::with_params("regex", vec![pattern])
    }

    /// ISBN validation rule (ISBN-10 or ISBN-13)
    pub fn isbn() -> Rule {
        Rule::with_params("regex", vec![
            r"^(?:ISBN(?:-1[03])?:? )?(?=[0-9X]{10}$|(?=(?:[0-9]+[- ]){3})[- 0-9X]{13}$|97[89][0-9]{10}$|(?=(?:[0-9]+[- ]){4})[- 0-9]{17}$)(?:97[89][- ]?)?[0-9]{1,5}[- ]?[0-9]+[- ]?[0-9]+[- ]?[0-9X]$".to_string()
        ])
    }

    /// ISSN validation rule (International Standard Serial Number)
    pub fn issn() -> Rule {
        Rule::with_params("regex", vec![
            r"^[0-9]{4}-[0-9]{3}[0-9X]$".to_string()
        ])
    }

    /// ISIN validation rule (International Securities Identification Number)
    pub fn isin() -> Rule {
        Rule::with_params("regex", vec![
            r"^[A-Z]{2}[A-Z0-9]{9}[0-9]$".to_string()
        ])
    }

    /// EAN validation rule (European Article Number)
    pub fn ean() -> Rule {
        Rule::with_params("regex", vec![
            r"^[0-9]{8}$|^[0-9]{13}$".to_string()
        ])
    }

    /// IMEI validation rule (International Mobile Equipment Identity)
    pub fn imei() -> Rule {
        Rule::with_params("regex", vec![
            r"^[0-9]{15}$".to_string()
        ])
    }

    /// VIN validation rule (Vehicle Identification Number)
    pub fn vin() -> Rule {
        Rule::with_params("regex", vec![
            r"^[A-HJ-NPR-Z0-9]{17}$".to_string()
        ])
    }

    /// Semver validation rule (Semantic Versioning)
    pub fn semver() -> Rule {
        Rule::with_params("regex", vec![
            r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$".to_string()
        ])
    }
}

/// Conditional validation helpers
pub struct ConditionalValidation;

impl ConditionalValidation {
    /// Create a validator that only applies rules when a condition is met
    pub fn when<F>(condition: F, rules: Vec<Rule>) -> Vec<Rule>
    where
        F: FnOnce() -> bool,
    {
        if condition() {
            rules
        } else {
            vec![]
        }
    }

    /// Create a validator that applies different rules based on a condition
    pub fn when_else<F>(condition: F, if_rules: Vec<Rule>, else_rules: Vec<Rule>) -> Vec<Rule>
    where
        F: FnOnce() -> bool,
    {
        if condition() {
            if_rules
        } else {
            else_rules
        }
    }
}

/// Utility functions for working with validation
pub struct ValidationUtils;

impl ValidationUtils {
    /// Convert Axum JSON extraction to HashMap for validation
    pub fn from_json_extract<T>(json: Json<T>) -> HashMap<String, Value>
    where
        T: Serialize,
    {
        let value = serde_json::to_value(json.0).unwrap_or(Value::Null);
        match value {
            Value::Object(map) => map.into_iter().collect(),
            _ => HashMap::new(),
        }
    }

    /// Merge multiple validation error results
    pub fn merge_errors(errors_list: Vec<ValidationErrors>) -> ValidationErrors {
        let mut merged = ValidationErrors::new();
        for errors in errors_list {
            for error in errors.errors {
                merged.add(error);
            }
        }
        merged
    }

    /// Check if a field has a specific validation error
    pub fn has_error(errors: &ValidationErrors, field: &str, rule: &str) -> bool {
        errors.errors.iter().any(|e| e.field == field && e.rule == rule)
    }

    /// Get all errors for a specific field
    pub fn get_field_errors<'a>(errors: &'a ValidationErrors, field: &str) -> Vec<&'a ValidationError> {
        errors.errors.iter().filter(|e| e.field == field).collect()
    }

    /// Convert validation errors to a simple string format
    pub fn errors_to_string(errors: &ValidationErrors) -> String {
        errors
            .errors
            .iter()
            .map(|e| format!("{}: {}", e.field, e.message))
            .collect::<Vec<String>>()
            .join(", ")
    }

    /// Create a summary of validation errors by field
    pub fn error_summary(errors: &ValidationErrors) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        for error in &errors.errors {
            *summary.entry(error.field.clone()).or_insert(0) += 1;
        }
        summary
    }
}

/// Trait for adding validation to any struct
pub trait Validatable {
    fn validate(&self) -> Result<(), ValidationErrors>;
    fn validate_for_creation(&self) -> Result<(), ValidationErrors> {
        self.validate()
    }
    fn validate_for_update(&self) -> Result<(), ValidationErrors> {
        self.validate()
    }
}

/// Example implementation for a User struct
#[derive(Debug, Serialize, Deserialize)]
pub struct UserValidation {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub password_confirmation: Option<String>,
    pub age: Option<i32>,
}

impl Validatable for UserValidation {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let data = serde_json::to_value(self).unwrap();
        validate!(data, {
            "name" => ["string", "min:2", "max:255"],
            "email" => ["email", "max:255"],
            "age" => ["integer", "min:13", "max:120"]
        })
    }

    fn validate_for_creation(&self) -> Result<(), ValidationErrors> {
        let data = serde_json::to_value(self).unwrap();
        validate!(data, {
            "name" => ["required", "string", "min:2", "max:255"],
            "email" => ["required", "email", "max:255"],
            "password" => ["required", "string", "min:8", "confirmed"]
        })
    }

    fn validate_for_update(&self) -> Result<(), ValidationErrors> {
        let data = serde_json::to_value(self).unwrap();
        validate!(data, {
            "name" => ["string", "min:2", "max:255"],
            "email" => ["email", "max:255"],
            "password" => ["string", "min:8", "confirmed"]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validator_builder() {
        let data = json!({
            "name": "John",
            "email": "john@example.com"
        });

        let result = ValidatorBuilder::from_json(data)
            .rules_from_array("name", vec!["required", "string", "min:2"])
            .rules_from_array("email", vec!["required", "email"])
            .message("name.required", "Name is required")
            .validate();

        assert!(result.is_ok());
    }

    #[test]
    fn test_common_validators_user_registration() {
        let valid_data = json!({
            "name": "John Doe",
            "email": "john@example.com",
            "password": "SecurePass123!",
            "password_confirmation": "SecurePass123!"
        });

        let result = CommonValidators::user_registration(valid_data);
        assert!(result.is_ok());

        let invalid_data = json!({
            "name": "",
            "email": "invalid-email",
            "password": "weak"
        });

        let result = CommonValidators::user_registration(invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_rules() {
        let username_rule = CustomRules::username();
        assert_eq!(username_rule.name, "regex");
        assert!(!username_rule.parameters.is_empty());

        let phone_rule = CustomRules::phone_number();
        assert_eq!(phone_rule.name, "regex");
    }

    #[test]
    fn test_conditional_validation() {
        let rules_when_true = ConditionalValidation::when(|| true, vec![
            crate::app::utils::validator::required(),
            crate::app::utils::validator::string(),
        ]);
        assert_eq!(rules_when_true.len(), 2);

        let rules_when_false = ConditionalValidation::when(|| false, vec![
            crate::app::utils::validator::required(),
            crate::app::utils::validator::string(),
        ]);
        assert_eq!(rules_when_false.len(), 0);
    }

    #[test]
    fn test_validation_utils() {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new("name", "required", "Name is required"));
        errors.add(ValidationError::new("email", "email", "Email is invalid"));

        assert!(ValidationUtils::has_error(&errors, "name", "required"));
        assert!(!ValidationUtils::has_error(&errors, "name", "string"));

        let field_errors = ValidationUtils::get_field_errors(&errors, "name");
        assert_eq!(field_errors.len(), 1);

        let summary = ValidationUtils::error_summary(&errors);
        assert_eq!(summary.get("name"), Some(&1));
        assert_eq!(summary.get("email"), Some(&1));
    }

    #[test]
    fn test_user_validation() {
        let user = UserValidation {
            name: Some("John Doe".to_string()),
            email: Some("john@example.com".to_string()),
            password: Some("SecurePass123!".to_string()),
            password_confirmation: Some("SecurePass123!".to_string()),
            age: Some(25),
        };

        assert!(user.validate().is_ok());
        assert!(user.validate_for_creation().is_ok());
        assert!(user.validate_for_update().is_ok());
    }
}