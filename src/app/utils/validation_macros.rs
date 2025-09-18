//! Validation macros for convenient rule definitions
//!
//! This module provides macros to make validation rule definitions more convenient
//! and similar to Laravel's validation syntax.

use crate::app::utils::validator::{Rule, Validator};
use std::collections::HashMap;
use serde_json::Value;

/// Macro for creating validation rules in array format
///
/// # Examples
///
/// ```rust
/// use crate::validation_rules;
///
/// let rules = validation_rules! {
///     "name" => ["required", "string", "min:2", "max:50"],
///     "email" => ["required", "email"],
///     "age" => ["required", "integer", "min:18", "max:120"],
///     "password" => ["required", "string", "min:8", "confirmed"],
///     "tags" => ["array"],
///     "website" => ["nullable", "url"]
/// };
/// ```
#[macro_export]
macro_rules! validation_rules {
    ($($field:expr => [$($rule:expr),* $(,)?]),* $(,)?) => {
        {
            let mut rules_map: std::collections::HashMap<String, Vec<$crate::app::utils::validator::Rule>> = std::collections::HashMap::new();
            $(
                rules_map.insert($field.to_string(), $crate::app::utils::validation_macros::parse_rules(vec![$($rule),*]));
            )*
            rules_map
        }
    };
}

/// Macro for creating custom validation messages
///
/// # Examples
///
/// ```rust
/// use crate::validation_messages;
///
/// let messages = validation_messages! {
///     "name.required" => "Please provide your name",
///     "email.email" => "Please enter a valid email address",
///     "age.min" => "You must be at least 18 years old"
/// };
/// ```
#[macro_export]
macro_rules! validation_messages {
    ($($key:expr => $message:expr),* $(,)?) => {
        {
            let mut messages: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            $(
                messages.insert($key.to_string(), $message.to_string());
            )*
            messages
        }
    };
}

/// Macro for quick validator creation and validation using array format
///
/// # Examples
///
/// ```rust
/// use crate::{validate, validation_rules, validation_messages};
/// use serde_json::json;
///
/// let data = json!({
///     "name": "John Doe",
///     "email": "john@example.com",
///     "age": 25
/// });
///
/// let result = validate!(data, {
///     "name" => ["required", "string", "min:2"],
///     "email" => ["required", "email"],
///     "age" => ["required", "integer", "min:18"]
/// });
///
/// // With custom messages
/// let result = validate!(data, {
///     "name" => ["required", "string", "min:2"],
///     "email" => ["required", "email"]
/// }, {
///     "name.required" => "Name is required",
///     "email.email" => "Valid email required"
/// });
/// ```
#[macro_export]
macro_rules! validate {
    ($data:expr, { $($field:expr => [$($rule:expr),* $(,)?]),* $(,)? }) => {
        {
            let data_map = $crate::app::utils::validation_macros::json_to_hashmap($data);
            let rules = validation_rules! { $($field => [$($rule),*]),* };
            let validator = $crate::app::utils::validator::Validator::make(data_map, rules);
            validator.validate()
        }
    };

    ($data:expr, { $($field:expr => [$($rule:expr),* $(,)?]),* $(,)? }, { $($msg_key:expr => $msg_value:expr),* $(,)? }) => {
        {
            let data_map = $crate::app::utils::validation_macros::json_to_hashmap($data);
            let rules = validation_rules! { $($field => [$($rule),*]),* };
            let messages = validation_messages! { $($msg_key => $msg_value),* };
            let mut validator = $crate::app::utils::validator::Validator::make(data_map, rules);
            validator.messages(messages);
            validator.validate()
        }
    };
}

/// Macro for creating a validator instance with fluent API
///
/// # Examples
///
/// ```rust
/// use crate::validator;
/// use serde_json::json;
///
/// let data = json!({"name": "John", "email": "john@example.com"});
///
/// let mut v = validator!(data)
///     .rules("name", "required|string|min:2")
///     .rules("email", "required|email")
///     .message("name.required", "Name is required");
///
/// let result = v.validate();
/// ```
#[macro_export]
macro_rules! validator {
    ($data:expr) => {
        {
            let data_map = $crate::app::utils::validation_macros::json_to_hashmap($data);
            $crate::app::utils::validator::Validator::new(data_map)
        }
    };
}

/// Parse validation rules from an array format
///
/// # Examples
///
/// ```rust
/// let rules = parse_rules(vec!["required", "string", "min:2", "max:50"]);
/// ```
pub fn parse_rules<T: AsRef<str>>(rules: Vec<T>) -> Vec<Rule> {
    rules
        .into_iter()
        .map(|rule| {
            let rule_str = rule.as_ref();
            if rule_str.contains(':') {
                let parts: Vec<&str> = rule_str.splitn(2, ':').collect();
                let rule_name = parts[0];
                let params: Vec<String> = parts[1]
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                Rule::with_params(rule_name, params)
            } else {
                Rule::new(rule_str.trim())
            }
        })
        .collect()
}

/// Parse a rule string into a vector of Rule structs (deprecated - use parse_rules with array instead)
///
/// # Examples
///
/// ```rust
/// let rules = parse_rule_string("required|string|min:2|max:50");
/// ```
#[deprecated(note = "Use parse_rules with array format instead")]
pub fn parse_rule_string(rule_str: &str) -> Vec<Rule> {
    rule_str
        .split('|')
        .map(|rule| {
            if rule.contains(':') {
                let parts: Vec<&str> = rule.splitn(2, ':').collect();
                let rule_name = parts[0];
                let params: Vec<String> = parts[1]
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                Rule::with_params(rule_name, params)
            } else {
                Rule::new(rule.trim())
            }
        })
        .collect()
}

/// Convert serde_json::Value to HashMap<String, Value> for validator
pub fn json_to_hashmap(value: Value) -> HashMap<String, Value> {
    match value {
        Value::Object(map) => map.into_iter().collect(),
        _ => HashMap::new(),
    }
}

/// Extension trait for Validator to add fluent API methods
pub trait ValidatorExt {
    fn rules(&mut self, field: &str, rules: &str) -> &mut Self;
    fn rules_array<T: AsRef<str>>(&mut self, field: &str, rules: Vec<T>) -> &mut Self;
    fn message(&mut self, key: &str, message: &str) -> &mut Self;
}

impl ValidatorExt for Validator {
    fn rules(&mut self, field: &str, rules: &str) -> &mut Self {
        let parsed_rules = parse_rule_string(rules);
        self.rules(field, parsed_rules);
        self
    }

    fn rules_array<T: AsRef<str>>(&mut self, field: &str, rules: Vec<T>) -> &mut Self {
        let parsed_rules = parse_rules(rules);
        self.rules(field, parsed_rules);
        self
    }

    fn message(&mut self, key: &str, message: &str) -> &mut Self {
        let mut messages = HashMap::new();
        messages.insert(key.to_string(), message.to_string());
        self.messages(messages);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_rules() {
        let rules = parse_rules(vec!["required", "string", "min:2", "max:50"]);
        assert_eq!(rules.len(), 4);
        assert_eq!(rules[0].name, "required");
        assert_eq!(rules[1].name, "string");
        assert_eq!(rules[2].name, "min");
        assert_eq!(rules[2].parameters, vec!["2"]);
        assert_eq!(rules[3].name, "max");
        assert_eq!(rules[3].parameters, vec!["50"]);
    }

    #[test]
    fn test_parse_rules_with_multiple_params() {
        let rules = parse_rules(vec!["between:1,10", "in:active,inactive,pending"]);
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].name, "between");
        assert_eq!(rules[0].parameters, vec!["1", "10"]);
        assert_eq!(rules[1].name, "in");
        assert_eq!(rules[1].parameters, vec!["active", "inactive", "pending"]);
    }

    #[test]
    fn test_parse_rule_string() {
        let rules = parse_rule_string("required|string|min:2|max:50");
        assert_eq!(rules.len(), 4);
        assert_eq!(rules[0].name, "required");
        assert_eq!(rules[1].name, "string");
        assert_eq!(rules[2].name, "min");
        assert_eq!(rules[2].parameters, vec!["2"]);
        assert_eq!(rules[3].name, "max");
        assert_eq!(rules[3].parameters, vec!["50"]);
    }

    #[test]
    fn test_parse_rule_string_with_multiple_params() {
        let rules = parse_rule_string("between:1,10|in:active,inactive,pending");
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].name, "between");
        assert_eq!(rules[0].parameters, vec!["1", "10"]);
        assert_eq!(rules[1].name, "in");
        assert_eq!(rules[1].parameters, vec!["active", "inactive", "pending"]);
    }

    #[test]
    fn test_json_to_hashmap() {
        let json_data = json!({
            "name": "John",
            "age": 25,
            "active": true
        });

        let hashmap = json_to_hashmap(json_data);
        assert_eq!(hashmap.len(), 3);
        assert_eq!(hashmap.get("name"), Some(&Value::String("John".to_string())));
        assert_eq!(hashmap.get("age"), Some(&Value::Number(serde_json::Number::from(25))));
        assert_eq!(hashmap.get("active"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_validation_rules_macro() {
        let rules = validation_rules! {
            "name" => ["required", "string", "min:2"],
            "email" => ["required", "email"],
            "age" => ["integer", "min:18"]
        };

        assert_eq!(rules.len(), 3);
        assert!(rules.contains_key("name"));
        assert!(rules.contains_key("email"));
        assert!(rules.contains_key("age"));

        let name_rules = rules.get("name").unwrap();
        assert_eq!(name_rules.len(), 3);
        assert_eq!(name_rules[0].name, "required");
        assert_eq!(name_rules[1].name, "string");
        assert_eq!(name_rules[2].name, "min");
    }

    #[test]
    fn test_validation_messages_macro() {
        let messages = validation_messages! {
            "name.required" => "Name is required",
            "email.email" => "Please enter a valid email"
        };

        assert_eq!(messages.len(), 2);
        assert_eq!(messages.get("name.required"), Some(&"Name is required".to_string()));
        assert_eq!(messages.get("email.email"), Some(&"Please enter a valid email".to_string()));
    }

    #[test]
    fn test_validate_macro() {
        let data = json!({
            "name": "John",
            "email": "john@example.com",
            "age": 25
        });

        let result = validate!(data, {
            "name" => ["required", "string", "min:2"],
            "email" => ["required", "email"],
            "age" => ["required", "integer", "min:18"]
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_macro_with_messages() {
        let data = json!({
            "name": "",
            "email": "invalid-email"
        });

        let result = validate!(data, {
            "name" => ["required", "string", "min:2"],
            "email" => ["required", "email"]
        }, {
            "name.required" => "Custom name required message",
            "email.email" => "Custom email format message"
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_validator_macro() {
        let data = json!({
            "name": "John",
            "email": "john@example.com"
        });

        let mut validator = validator!(data);
        let parsed_rules1 = parse_rules(vec!["required", "string", "min:2"]);
        let parsed_rules2 = parse_rules(vec!["required", "email"]);
        validator
            .rules("name", parsed_rules1)
            .rules("email", parsed_rules2);

        let mut messages = HashMap::new();
        messages.insert("name.required".to_string(), "Custom name message".to_string());
        validator.messages(messages);

        let result = validator.validate();
        assert!(result.is_ok());
    }
}