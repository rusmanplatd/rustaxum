use rustaxum::app::utils::validator::*;
use rustaxum::app::utils::validation_macros::*;
use rustaxum::app::utils::validation_helpers::*;
use std::collections::HashMap;
use serde_json::{json, Value};

#[cfg(test)]
mod validator_core_tests {
    use super::*;

    fn create_test_data() -> HashMap<String, Value> {
        let mut data = HashMap::new();
        data.insert("name".to_string(), Value::String("John Doe".to_string()));
        data.insert("email".to_string(), Value::String("john@example.com".to_string()));
        data.insert("age".to_string(), Value::Number(serde_json::Number::from(25)));
        data.insert("active".to_string(), Value::Bool(true));
        data
    }

    #[test]
    fn test_validator_creation() {
        let data = create_test_data();
        let validator = Validator::new(data);
        assert!(validator.validate().is_ok());
    }

    #[test]
    fn test_validation_error_creation() {
        let error = ValidationError::new("name", "required", "Name is required");
        assert_eq!(error.field, "name");
        assert_eq!(error.rule, "required");
        assert_eq!(error.message, "Name is required");
        assert!(error.value.is_none());

        let error_with_value = error.with_value("test");
        assert_eq!(error_with_value.value, Some("test".to_string()));
    }

    #[test]
    fn test_validation_errors_collection() {
        let mut errors = ValidationErrors::new();
        assert!(!errors.has_errors());

        errors.add(ValidationError::new("name", "required", "Name is required"));
        errors.add(ValidationError::new("email", "email", "Email is invalid"));

        assert!(errors.has_errors());
        assert_eq!(errors.errors.len(), 2);

        let messages = errors.get_messages();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages.get("name").unwrap()[0], "Name is required");
        assert_eq!(messages.get("email").unwrap()[0], "Email is invalid");

        assert_eq!(errors.first("name"), Some(&"Name is required".to_string()));
        assert_eq!(errors.first("nonexistent"), None);
    }
}

#[cfg(test)]
mod validation_rules_tests {
    use super::*;

    #[test]
    fn test_required_rule() {
        let mut data = HashMap::new();

        // Test required field missing
        let validator = Validator::make(data.clone(), validation_rules! {
            "name" => "required"
        });
        assert!(validator.validate().is_err());

        // Test required field with null value
        data.insert("name".to_string(), Value::Null);
        let validator = Validator::make(data.clone(), validation_rules! {
            "name" => "required"
        });
        assert!(validator.validate().is_err());

        // Test required field with empty string
        data.insert("name".to_string(), Value::String("".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "name" => "required"
        });
        assert!(validator.validate().is_err());

        // Test required field with valid value
        data.insert("name".to_string(), Value::String("John".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "name" => "required"
        });
        assert!(validator.validate().is_ok());
    }

    #[test]
    fn test_string_rule() {
        let mut data = HashMap::new();

        // Valid string
        data.insert("name".to_string(), Value::String("John".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "name" => "string"
        });
        assert!(validator.validate().is_ok());

        // Invalid - number
        data.insert("name".to_string(), Value::Number(serde_json::Number::from(123)));
        let validator = Validator::make(data.clone(), validation_rules! {
            "name" => "string"
        });
        assert!(validator.validate().is_err());

        // Invalid - boolean
        data.insert("name".to_string(), Value::Bool(true));
        let validator = Validator::make(data.clone(), validation_rules! {
            "name" => "string"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_numeric_rule() {
        let mut data = HashMap::new();

        // Valid number
        data.insert("age".to_string(), Value::Number(serde_json::Number::from(25)));
        let validator = Validator::make(data.clone(), validation_rules! {
            "age" => "numeric"
        });
        assert!(validator.validate().is_ok());

        // Valid numeric string
        data.insert("age".to_string(), Value::String("25.5".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "age" => "numeric"
        });
        assert!(validator.validate().is_ok());

        // Invalid - non-numeric string
        data.insert("age".to_string(), Value::String("not-a-number".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "age" => "numeric"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_integer_rule() {
        let mut data = HashMap::new();

        // Valid integer
        data.insert("age".to_string(), Value::Number(serde_json::Number::from(25)));
        let validator = Validator::make(data.clone(), validation_rules! {
            "age" => "integer"
        });
        assert!(validator.validate().is_ok());

        // Valid integer string
        data.insert("age".to_string(), Value::String("25".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "age" => "integer"
        });
        assert!(validator.validate().is_ok());

        // Invalid - float
        data.insert("age".to_string(), Value::String("25.5".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "age" => "integer"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_boolean_rule() {
        let mut data = HashMap::new();

        // Valid boolean
        data.insert("active".to_string(), Value::Bool(true));
        let validator = Validator::make(data.clone(), validation_rules! {
            "active" => "boolean"
        });
        assert!(validator.validate().is_ok());

        // Valid boolean strings
        for value in ["true", "false", "1", "0", "yes", "no", "on", "off"] {
            data.insert("active".to_string(), Value::String(value.to_string()));
            let validator = Validator::make(data.clone(), validation_rules! {
                "active" => "boolean"
            });
            assert!(validator.validate().is_ok(), "Failed for value: {}", value);
        }

        // Valid boolean numbers
        data.insert("active".to_string(), Value::Number(serde_json::Number::from(1)));
        let validator = Validator::make(data.clone(), validation_rules! {
            "active" => "boolean"
        });
        assert!(validator.validate().is_ok());

        // Invalid boolean
        data.insert("active".to_string(), Value::String("maybe".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "active" => "boolean"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_email_rule() {
        let mut data = HashMap::new();

        // Valid emails
        let valid_emails = [
            "user@example.com",
            "test.email+tag@example.co.uk",
            "user123@test-domain.org",
        ];

        for email in valid_emails {
            data.insert("email".to_string(), Value::String(email.to_string()));
            let validator = Validator::make(data.clone(), validation_rules! {
                "email" => "email"
            });
            assert!(validator.validate().is_ok(), "Failed for email: {}", email);
        }

        // Invalid emails
        let invalid_emails = [
            "invalid-email",
            "@example.com",
            "user@",
            "user..name@example.com",
            "user@.com",
        ];

        for email in invalid_emails {
            data.insert("email".to_string(), Value::String(email.to_string()));
            let validator = Validator::make(data.clone(), validation_rules! {
                "email" => "email"
            });
            assert!(validator.validate().is_err(), "Should fail for email: {}", email);
        }
    }

    #[test]
    fn test_url_rule() {
        let mut data = HashMap::new();

        // Valid URLs
        let valid_urls = [
            "https://example.com",
            "http://test.org/path",
            "https://subdomain.example.com/path?query=value",
        ];

        for url in valid_urls {
            data.insert("website".to_string(), Value::String(url.to_string()));
            let validator = Validator::make(data.clone(), validation_rules! {
                "website" => "url"
            });
            assert!(validator.validate().is_ok(), "Failed for URL: {}", url);
        }

        // Invalid URLs
        let invalid_urls = [
            "not-a-url",
            "ftp://example.com",
            "example.com",
            "http://",
        ];

        for url in invalid_urls {
            data.insert("website".to_string(), Value::String(url.to_string()));
            let validator = Validator::make(data.clone(), validation_rules! {
                "website" => "url"
            });
            assert!(validator.validate().is_err(), "Should fail for URL: {}", url);
        }
    }

    #[test]
    fn test_min_rule() {
        let mut data = HashMap::new();

        // String length
        data.insert("name".to_string(), Value::String("John".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "name" => "min:3"
        });
        assert!(validator.validate().is_ok());

        data.insert("name".to_string(), Value::String("Jo".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "name" => "min:3"
        });
        assert!(validator.validate().is_err());

        // Numeric value
        data.insert("age".to_string(), Value::Number(serde_json::Number::from(25)));
        let validator = Validator::make(data.clone(), validation_rules! {
            "age" => "min:18"
        });
        assert!(validator.validate().is_ok());

        data.insert("age".to_string(), Value::Number(serde_json::Number::from(15)));
        let validator = Validator::make(data.clone(), validation_rules! {
            "age" => "min:18"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_max_rule() {
        let mut data = HashMap::new();

        // String length
        data.insert("name".to_string(), Value::String("John".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "name" => "max:10"
        });
        assert!(validator.validate().is_ok());

        data.insert("name".to_string(), Value::String("Very Long Name".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "name" => "max:10"
        });
        assert!(validator.validate().is_err());

        // Numeric value
        data.insert("age".to_string(), Value::Number(serde_json::Number::from(25)));
        let validator = Validator::make(data.clone(), validation_rules! {
            "age" => "max:100"
        });
        assert!(validator.validate().is_ok());

        data.insert("age".to_string(), Value::Number(serde_json::Number::from(150)));
        let validator = Validator::make(data.clone(), validation_rules! {
            "age" => "max:100"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_between_rule() {
        let mut data = HashMap::new();

        // Valid range
        data.insert("age".to_string(), Value::Number(serde_json::Number::from(25)));
        let validator = Validator::make(data.clone(), validation_rules! {
            "age" => "between:18,65"
        });
        assert!(validator.validate().is_ok());

        // Below range
        data.insert("age".to_string(), Value::Number(serde_json::Number::from(15)));
        let validator = Validator::make(data.clone(), validation_rules! {
            "age" => "between:18,65"
        });
        assert!(validator.validate().is_err());

        // Above range
        data.insert("age".to_string(), Value::Number(serde_json::Number::from(70)));
        let validator = Validator::make(data.clone(), validation_rules! {
            "age" => "between:18,65"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_size_rule() {
        let mut data = HashMap::new();

        // String exact size
        data.insert("code".to_string(), Value::String("ABCD".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "code" => "size:4"
        });
        assert!(validator.validate().is_ok());

        data.insert("code".to_string(), Value::String("ABC".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "code" => "size:4"
        });
        assert!(validator.validate().is_err());

        // Array size
        data.insert("items".to_string(), Value::Array(vec![
            Value::String("item1".to_string()),
            Value::String("item2".to_string()),
        ]));
        let validator = Validator::make(data.clone(), validation_rules! {
            "items" => "size:2"
        });
        assert!(validator.validate().is_ok());
    }

    #[test]
    fn test_in_rule() {
        let mut data = HashMap::new();

        // Valid value in list
        data.insert("status".to_string(), Value::String("active".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "status" => "in:active,inactive,pending"
        });
        assert!(validator.validate().is_ok());

        // Invalid value not in list
        data.insert("status".to_string(), Value::String("unknown".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "status" => "in:active,inactive,pending"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_not_in_rule() {
        let mut data = HashMap::new();

        // Valid value not in forbidden list
        data.insert("username".to_string(), Value::String("johndoe".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "username" => "not_in:admin,root,test"
        });
        assert!(validator.validate().is_ok());

        // Invalid value in forbidden list
        data.insert("username".to_string(), Value::String("admin".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "username" => "not_in:admin,root,test"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_alpha_rule() {
        let mut data = HashMap::new();

        // Valid alphabetic string
        data.insert("name".to_string(), Value::String("JohnDoe".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "name" => "alpha"
        });
        assert!(validator.validate().is_ok());

        // Invalid - contains numbers
        data.insert("name".to_string(), Value::String("John123".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "name" => "alpha"
        });
        assert!(validator.validate().is_err());

        // Invalid - contains special characters
        data.insert("name".to_string(), Value::String("John-Doe".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "name" => "alpha"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_alpha_num_rule() {
        let mut data = HashMap::new();

        // Valid alphanumeric string
        data.insert("username".to_string(), Value::String("User123".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "username" => "alpha_num"
        });
        assert!(validator.validate().is_ok());

        // Invalid - contains special characters
        data.insert("username".to_string(), Value::String("User-123".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "username" => "alpha_num"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_alpha_dash_rule() {
        let mut data = HashMap::new();

        // Valid string with alphanumeric, dash, and underscore
        data.insert("slug".to_string(), Value::String("my-slug_123".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "slug" => "alpha_dash"
        });
        assert!(validator.validate().is_ok());

        // Invalid - contains spaces
        data.insert("slug".to_string(), Value::String("my slug".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "slug" => "alpha_dash"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_regex_rule() {
        let mut data = HashMap::new();

        // Valid phone number pattern
        data.insert("phone".to_string(), Value::String("123-456-7890".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "phone" => "regex:^[0-9]{3}-[0-9]{3}-[0-9]{4}$"
        });
        assert!(validator.validate().is_ok());

        // Invalid phone number pattern
        data.insert("phone".to_string(), Value::String("123-45-6789".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "phone" => "regex:^[0-9]{3}-[0-9]{3}-[0-9]{4}$"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_confirmed_rule() {
        let mut data = HashMap::new();

        // Valid confirmation
        data.insert("password".to_string(), Value::String("secret123".to_string()));
        data.insert("password_confirmation".to_string(), Value::String("secret123".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "password" => "confirmed"
        });
        assert!(validator.validate().is_ok());

        // Invalid confirmation
        data.insert("password_confirmation".to_string(), Value::String("different".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "password" => "confirmed"
        });
        assert!(validator.validate().is_err());

        // Missing confirmation
        data.remove("password_confirmation");
        let validator = Validator::make(data.clone(), validation_rules! {
            "password" => "confirmed"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_same_rule() {
        let mut data = HashMap::new();

        // Valid same values
        data.insert("field1".to_string(), Value::String("value".to_string()));
        data.insert("field2".to_string(), Value::String("value".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "field1" => "same:field2"
        });
        assert!(validator.validate().is_ok());

        // Invalid different values
        data.insert("field2".to_string(), Value::String("different".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "field1" => "same:field2"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_different_rule() {
        let mut data = HashMap::new();

        // Valid different values
        data.insert("field1".to_string(), Value::String("value1".to_string()));
        data.insert("field2".to_string(), Value::String("value2".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "field1" => "different:field2"
        });
        assert!(validator.validate().is_ok());

        // Invalid same values
        data.insert("field2".to_string(), Value::String("value1".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "field1" => "different:field2"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_date_rule() {
        let mut data = HashMap::new();

        // Valid date formats
        let valid_dates = [
            "2023-12-25",
            "2023-12-25 15:30:00",
            "2023/12/25",
            "25/12/2023",
            "12/25/2023",
        ];

        for date in valid_dates {
            data.insert("date".to_string(), Value::String(date.to_string()));
            let validator = Validator::make(data.clone(), validation_rules! {
                "date" => "date"
            });
            assert!(validator.validate().is_ok(), "Failed for date: {}", date);
        }

        // Invalid date
        data.insert("date".to_string(), Value::String("not-a-date".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "date" => "date"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_json_rule() {
        let mut data = HashMap::new();

        // Valid JSON
        data.insert("config".to_string(), Value::String(r#"{"key": "value"}"#.to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "config" => "json"
        });
        assert!(validator.validate().is_ok());

        // Invalid JSON
        data.insert("config".to_string(), Value::String(r#"{"key": invalid}"#.to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "config" => "json"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_array_rule() {
        let mut data = HashMap::new();

        // Valid array
        data.insert("items".to_string(), Value::Array(vec![
            Value::String("item1".to_string()),
            Value::String("item2".to_string()),
        ]));
        let validator = Validator::make(data.clone(), validation_rules! {
            "items" => "array"
        });
        assert!(validator.validate().is_ok());

        // Invalid - not an array
        data.insert("items".to_string(), Value::String("not-an-array".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "items" => "array"
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_nullable_rule() {
        let mut data = HashMap::new();

        // Null value should pass
        data.insert("optional".to_string(), Value::Null);
        let validator = Validator::make(data.clone(), validation_rules! {
            "optional" => "nullable|string"
        });
        assert!(validator.validate().is_ok());

        // Valid value should pass
        data.insert("optional".to_string(), Value::String("value".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "optional" => "nullable|string"
        });
        assert!(validator.validate().is_ok());
    }

    #[test]
    fn test_multiple_rules() {
        let mut data = HashMap::new();

        // Valid data passing multiple rules
        data.insert("email".to_string(), Value::String("user@example.com".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "email" => "required|string|email|max:255"
        });
        assert!(validator.validate().is_ok());

        // Invalid - fails one rule
        data.insert("email".to_string(), Value::String("invalid-email".to_string()));
        let validator = Validator::make(data.clone(), validation_rules! {
            "email" => "required|string|email|max:255"
        });
        assert!(validator.validate().is_err());
    }
}

#[cfg(test)]
mod macro_tests {
    use super::*;

    #[test]
    fn test_validation_rules_macro() {
        let rules = validation_rules! {
            "name" => "required|string|min:2|max:50",
            "email" => "required|email",
            "age" => "integer|min:18|max:120"
        };

        assert_eq!(rules.len(), 3);
        assert!(rules.contains_key("name"));
        assert!(rules.contains_key("email"));
        assert!(rules.contains_key("age"));
    }

    #[test]
    fn test_validation_messages_macro() {
        let messages = validation_messages! {
            "name.required" => "Please provide your name",
            "email.email" => "Please enter a valid email address"
        };

        assert_eq!(messages.len(), 2);
        assert_eq!(messages.get("name.required"), Some(&"Please provide your name".to_string()));
    }

    #[test]
    fn test_validate_macro() {
        let data = json!({
            "name": "John Doe",
            "email": "john@example.com",
            "age": 25
        });

        let result = validate!(data, {
            "name" => "required|string|min:2",
            "email" => "required|email",
            "age" => "required|integer|min:18"
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
            "name" => "required|string|min:2",
            "email" => "required|email"
        }, {
            "name.required" => "Custom name required message",
            "email.email" => "Custom email format message"
        });

        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors.has_errors());
        }
    }

    #[test]
    fn test_validator_macro() {
        let data = json!({
            "name": "John",
            "email": "john@example.com"
        });

        let mut validator = validator!(data);
        validator
            .rules("name", "required|string|min:2")
            .rules("email", "required|email");

        let result = validator.validate();
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_validator_builder() {
        let data = json!({
            "name": "John",
            "email": "john@example.com"
        });

        let result = ValidatorBuilder::from_json(data)
            .rules_from_string("name", "required|string|min:2")
            .rules_from_string("email", "required|email")
            .message("name.required", "Name is required")
            .validate();

        assert!(result.is_ok());
    }

    #[test]
    fn test_common_validators() {
        // Test user registration
        let registration_data = json!({
            "name": "John Doe",
            "email": "john@example.com",
            "password": "SecurePass123!",
            "password_confirmation": "SecurePass123!"
        });

        assert!(CommonValidators::user_registration(registration_data).is_ok());

        // Test user login
        let login_data = json!({
            "email": "john@example.com",
            "password": "password123"
        });

        assert!(CommonValidators::user_login(login_data).is_ok());

        // Test pagination
        let pagination_data = json!({
            "page": 1,
            "per_page": 20,
            "sort": "asc",
            "order_by": "created_at"
        });

        assert!(CommonValidators::pagination(pagination_data).is_ok());
    }

    #[test]
    fn test_custom_rules() {
        let username_rule = CustomRules::username();
        assert_eq!(username_rule.name, "regex");
        assert!(!username_rule.parameters.is_empty());

        let phone_rule = CustomRules::phone_number();
        assert_eq!(phone_rule.name, "regex");

        let email_rule = CustomRules::hex_color();
        assert_eq!(email_rule.name, "regex");
    }

    #[test]
    fn test_conditional_validation() {
        let rules_when_true = ConditionalValidation::when(|| true, vec![
            required(),
            string(),
        ]);
        assert_eq!(rules_when_true.len(), 2);

        let rules_when_false = ConditionalValidation::when(|| false, vec![
            required(),
            string(),
        ]);
        assert_eq!(rules_when_false.len(), 0);

        let rules_if_else = ConditionalValidation::when_else(
            || false,
            vec![required()],
            vec![string(), min(5)],
        );
        assert_eq!(rules_if_else.len(), 2);
    }

    #[test]
    fn test_validation_utils() {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new("name", "required", "Name is required"));
        errors.add(ValidationError::new("email", "email", "Email is invalid"));
        errors.add(ValidationError::new("name", "min", "Name too short"));

        assert!(ValidationUtils::has_error(&errors, "name", "required"));
        assert!(!ValidationUtils::has_error(&errors, "name", "max"));

        let field_errors = ValidationUtils::get_field_errors(&errors, "name");
        assert_eq!(field_errors.len(), 2);

        let summary = ValidationUtils::error_summary(&errors);
        assert_eq!(summary.get("name"), Some(&2));
        assert_eq!(summary.get("email"), Some(&1));

        let error_string = ValidationUtils::errors_to_string(&errors);
        assert!(error_string.contains("name: Name is required"));
        assert!(error_string.contains("email: Email is invalid"));
    }

    #[test]
    fn test_user_validation_trait() {
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

        let invalid_user = UserValidation {
            name: Some("J".to_string()), // Too short
            email: Some("invalid-email".to_string()),
            password: None,
            password_confirmation: None,
            age: Some(10), // Too young
        };

        assert!(invalid_user.validate().is_err());
        assert!(invalid_user.validate_for_creation().is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complex_validation_scenario() {
        let user_data = json!({
            "name": "John Doe",
            "email": "john@example.com",
            "password": "SecurePass123!",
            "password_confirmation": "SecurePass123!",
            "age": 25,
            "website": "https://johndoe.com",
            "bio": "Software developer",
            "tags": ["rust", "web", "api"],
            "preferences": {
                "theme": "dark",
                "notifications": true
            }
        });

        let result = validate!(user_data, {
            "name" => "required|string|min:2|max:100",
            "email" => "required|email|max:255",
            "password" => "required|string|min:8|confirmed",
            "age" => "required|integer|min:18|max:120",
            "website" => "nullable|url",
            "bio" => "nullable|string|max:1000",
            "tags" => "array",
            "preferences" => "required"
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_with_custom_messages() {
        let data = json!({
            "username": "a",
            "email": "invalid"
        });

        let result = validate!(data, {
            "username" => "required|string|min:3|max:30",
            "email" => "required|email"
        }, {
            "username.required" => "Username is required",
            "username.min" => "Username must be at least 3 characters",
            "email.email" => "Please provide a valid email address"
        });

        assert!(result.is_err());
        if let Err(errors) = result {
            let messages = errors.get_messages();
            assert!(messages.get("username").unwrap().iter().any(|msg| msg.contains("3 characters")));
            assert!(messages.get("email").unwrap().iter().any(|msg| msg.contains("valid email")));
        }
    }

    #[test]
    fn test_chained_validation_rules() {
        let data = json!({
            "product_code": "ABC-123",
            "price": 99.99,
            "status": "active",
            "tags": ["electronics", "gadget"]
        });

        let mut validator = validator!(data);
        validator
            .rules("product_code", "required|string|regex:^[A-Z]{3}-[0-9]{3}$")
            .rules("price", "required|numeric|min:0|max:9999.99")
            .rules("status", "required|in:active,inactive,draft")
            .rules("tags", "required|array|min:1|max:5")
            .message("product_code.regex", "Product code must be in format ABC-123")
            .message("price.max", "Price cannot exceed $9,999.99");

        let result = validator.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_response_format() {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new("name", "required", "Name is required"));
        errors.add(ValidationError::new("email", "email", "Email is invalid"));

        let messages = errors.get_messages();
        assert_eq!(messages.len(), 2);
        assert!(messages.contains_key("name"));
        assert!(messages.contains_key("email"));
    }
}