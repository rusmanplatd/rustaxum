//! Validation module - Re-exports for convenient access
//!
//! This module provides convenient access to all validation functionality
//! including validators, rules, macros, and helpers.

// Re-export core validation types
pub use crate::app::utils::validator::{
    Validator, ValidationError, ValidationErrors, Rule,
    // Basic validation rules
    required, string, numeric, integer, boolean, email, url,
    min, max, between, size, in_list, not_in, alpha, alpha_num, alpha_dash,
    regex, confirmed, same, different, unique, exists, date, date_format,
    before, after, json, array, nullable,
    // Additional validation rules
    accepted, active_url, bail, digits, digits_between, distinct,
    ends_with, starts_with, filled, gt, gte, lt, lte,
    ip, ipv4, ipv6, mac_address, multiple_of, not_regex,
    present, prohibited, prohibited_if, prohibited_unless,
    required_if, required_unless, required_with, required_with_all,
    required_without, required_without_all, sometimes, timezone, uuid,
    // File validation rules
    file, image, mimes, mimetypes, dimensions,
    // Array validation rules
    array_max, array_min,
    // Date validation rules
    after_or_equal, before_or_equal, date_equals,
};

// Re-export validation macros
pub use crate::{validation_rules, validation_messages, validate, validator};

// Re-export validation helpers
pub use crate::app::utils::validation_helpers::{
    ValidationErrorResponse, ValidatorBuilder, CommonValidators, CustomRules,
    ConditionalValidation, ValidationUtils, Validatable, UserValidation,
};

// Re-export validation macros module for advanced usage
pub use crate::app::utils::validation_macros::{
    parse_rule_string, parse_rules, json_to_hashmap, ValidatorExt,
};

/// Quick validation function for common use cases using array format
///
/// # Examples
///
/// ```rust
/// use rustaxum::validation::quick_validate;
/// use serde_json::json;
///
/// let data = json!({
///     "name": "John Doe",
///     "email": "john@example.com"
/// });
///
/// let rules = vec![
///     ("name", vec!["required", "string", "min:2"]),
///     ("email", vec!["required", "email"]),
/// ];
///
/// let result = quick_validate(data, rules);
/// ```
pub fn quick_validate<T: AsRef<str>>(
    data: serde_json::Value,
    rules: Vec<(&str, Vec<T>)>,
) -> Result<(), ValidationErrors> {
    let mut validator = validator!(data);
    for (field, rule_array) in rules {
        let parsed_rules = parse_rules(rule_array);
        validator.rules(field, parsed_rules);
    }
    validator.validate()
}

/// Quick validation function for common use cases using string format (deprecated)
///
/// # Examples
///
/// ```rust
/// use rustaxum::validation::quick_validate_string;
/// use serde_json::json;
///
/// let data = json!({
///     "name": "John Doe",
///     "email": "john@example.com"
/// });
///
/// let rules = vec![
///     ("name", "required|string|min:2"),
///     ("email", "required|email"),
/// ];
///
/// let result = quick_validate_string(data, rules);
/// ```
#[deprecated(note = "Use quick_validate with array format instead")]
pub fn quick_validate_string(
    data: serde_json::Value,
    rules: Vec<(&str, &str)>,
) -> Result<(), ValidationErrors> {
    let mut validator = validator!(data);
    for (field, rule_string) in rules {
        let parsed_rules = parse_rule_string(rule_string);
        validator.rules(field, parsed_rules);
    }
    validator.validate()
}

/// Validate request data with common patterns
///
/// # Examples
///
/// ```rust
/// use rustaxum::validation::validate_request;
/// use serde_json::json;
///
/// let data = json!({
///     "name": "John Doe",
///     "email": "john@example.com",
///     "password": "SecurePass123!",
///     "password_confirmation": "SecurePass123!"
/// });
///
/// let result = validate_request("user_registration", data);
/// ```
pub fn validate_request(
    request_type: &str,
    data: serde_json::Value,
) -> Result<(), ValidationErrors> {
    match request_type {
        "user_registration" => CommonValidators::user_registration(data),
        "user_login" => CommonValidators::user_login(data),
        "user_profile_update" => CommonValidators::user_profile_update(data),
        "password_change" => CommonValidators::password_change(data),
        "pagination" => CommonValidators::pagination(data),
        "contact_form" => CommonValidators::contact_form(data),
        "file_upload" => CommonValidators::file_upload(data),
        _ => Err(ValidationErrors {
            errors: vec![ValidationError::new(
                "request_type",
                "unknown",
                format!("Unknown request type: {}", request_type),
            )],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_quick_validate() {
        let data = json!({
            "name": "John Doe",
            "email": "john@example.com"
        });

        let rules = vec![
            ("name", vec!["required", "string", "min:2"]),
            ("email", vec!["required", "email"]),
        ];

        let result = quick_validate(data, rules);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_request() {
        let data = json!({
            "name": "John Doe",
            "email": "john@example.com",
            "password": "SecurePass123!",
            "password_confirmation": "SecurePass123!"
        });

        let result = validate_request("user_registration", data);
        assert!(result.is_ok());

        // Test unknown request type
        let result = validate_request("unknown_type", json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_module_exports() {
        // Test that all important types are accessible
        let _rule = required();
        let _error = ValidationError::new("field", "rule", "message");
        let _errors = ValidationErrors::new();
        let _builder = ValidatorBuilder::new(std::collections::HashMap::new());
    }
}