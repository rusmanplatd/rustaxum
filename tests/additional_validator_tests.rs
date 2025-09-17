use rustaxum::validation::*;
use std::collections::HashMap;
use serde_json::{json, Value};

#[cfg(test)]
mod additional_validation_rules_tests {
    use super::*;

    fn create_validator_with_data(data: Value, rules: HashMap<String, Vec<Rule>>) -> Validator {
        let data_map = match data {
            Value::Object(map) => map.into_iter().collect(),
            _ => HashMap::new(),
        };
        Validator::make(data_map, rules)
    }

    #[test]
    fn test_accepted_rule() {
        let mut rules = HashMap::new();
        rules.insert("terms".to_string(), vec![accepted()]);

        // Valid accepted values
        let valid_values = [
            json!(true),
            json!("yes"),
            json!("on"),
            json!("1"),
            json!("true"),
            json!(1),
        ];

        for value in valid_values {
            let data = json!({"terms": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_ok(), "Failed for value: {}", value);
        }

        // Invalid values
        let invalid_values = [
            json!(false),
            json!("no"),
            json!("off"),
            json!("0"),
            json!(0),
            json!("maybe"),
        ];

        for value in invalid_values {
            let data = json!({"terms": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_err(), "Should fail for value: {}", value);
        }
    }

    #[test]
    fn test_digits_rule() {
        let mut rules = HashMap::new();
        rules.insert("code".to_string(), vec![digits(4)]);

        // Valid 4-digit values
        let valid_values = [
            json!("1234"),
            json!("0000"),
            json!("9999"),
            json!(1234),
        ];

        for value in valid_values {
            let data = json!({"code": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_ok(), "Failed for value: {}", value);
        }

        // Invalid values
        let invalid_values = [
            json!("123"),     // Too short
            json!("12345"),   // Too long
            json!("abcd"),    // Not digits
            json!("12a4"),    // Mixed
        ];

        for value in invalid_values {
            let data = json!({"code": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_err(), "Should fail for value: {}", value);
        }
    }

    #[test]
    fn test_digits_between_rule() {
        let mut rules = HashMap::new();
        rules.insert("pin".to_string(), vec![digits_between(4, 6)]);

        // Valid values (4-6 digits)
        let valid_values = [
            json!("1234"),    // 4 digits
            json!("12345"),   // 5 digits
            json!("123456"),  // 6 digits
        ];

        for value in valid_values {
            let data = json!({"pin": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_ok(), "Failed for value: {}", value);
        }

        // Invalid values
        let invalid_values = [
            json!("123"),      // Too short
            json!("1234567"),  // Too long
            json!("abcd"),     // Not digits
        ];

        for value in invalid_values {
            let data = json!({"pin": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_err(), "Should fail for value: {}", value);
        }
    }

    #[test]
    fn test_distinct_rule() {
        let mut rules = HashMap::new();
        rules.insert("items".to_string(), vec![distinct()]);

        // Valid array with distinct values
        let data = json!({"items": ["a", "b", "c"]});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_ok());

        // Invalid array with duplicate values
        let data = json!({"items": ["a", "b", "a"]});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_err());

        // Non-array values should pass (rule doesn't apply)
        let data = json!({"items": "not an array"});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_ok());
    }

    #[test]
    fn test_ends_with_rule() {
        let mut rules = HashMap::new();
        rules.insert("filename".to_string(), vec![ends_with(vec!["jpg", "png", "gif"])]);

        // Valid endings
        let valid_values = [
            json!("photo.jpg"),
            json!("image.png"),
            json!("animation.gif"),
        ];

        for value in valid_values {
            let data = json!({"filename": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_ok(), "Failed for value: {}", value);
        }

        // Invalid endings
        let invalid_values = [
            json!("document.pdf"),
            json!("video.mp4"),
            json!("archive.zip"),
        ];

        for value in invalid_values {
            let data = json!({"filename": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_err(), "Should fail for value: {}", value);
        }
    }

    #[test]
    fn test_starts_with_rule() {
        let mut rules = HashMap::new();
        rules.insert("protocol".to_string(), vec![starts_with(vec!["http://", "https://", "ftp://"])]);

        // Valid prefixes
        let valid_values = [
            json!("http://example.com"),
            json!("https://secure.com"),
            json!("ftp://files.com"),
        ];

        for value in valid_values {
            let data = json!({"protocol": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_ok(), "Failed for value: {}", value);
        }

        // Invalid prefixes
        let invalid_values = [
            json!("smtp://mail.com"),
            json!("example.com"),
            json!("ssh://server.com"),
        ];

        for value in invalid_values {
            let data = json!({"protocol": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_err(), "Should fail for value: {}", value);
        }
    }

    #[test]
    fn test_filled_rule() {
        let mut rules = HashMap::new();
        rules.insert("optional_field".to_string(), vec![filled()]);

        // Valid: field not present (OK for filled rule)
        let data = json!({});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_ok());

        // Valid: field present with value
        let data = json!({"optional_field": "value"});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_ok());

        // Invalid: field present but empty
        let invalid_values = [
            json!(null),
            json!(""),
            json!([]),
            json!({}),
        ];

        for value in invalid_values {
            let data = json!({"optional_field": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_err(), "Should fail for value: {}", value);
        }
    }

    #[test]
    fn test_comparison_rules() {
        // Test gt, gte, lt, lte rules
        let data = json!({
            "min_age": 18,
            "max_age": 65,
            "user_age": 25
        });

        let mut rules = HashMap::new();
        rules.insert("user_age".to_string(), vec![gt("min_age"), lt("max_age")]);

        let validator = create_validator_with_data(data, rules);
        assert!(validator.validate().is_ok());

        // Test failing comparison
        let data = json!({
            "min_age": 18,
            "max_age": 65,
            "user_age": 15  // Less than min_age
        });

        let mut rules = HashMap::new();
        rules.insert("user_age".to_string(), vec![gt("min_age")]);

        let validator = create_validator_with_data(data, rules);
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_ip_validation_rules() {
        // Test IPv4
        let mut rules = HashMap::new();
        rules.insert("ip".to_string(), vec![ipv4()]);

        let valid_ipv4 = [
            json!("192.168.1.1"),
            json!("10.0.0.1"),
            json!("255.255.255.255"),
            json!("0.0.0.0"),
        ];

        for value in valid_ipv4 {
            let data = json!({"ip": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_ok(), "Failed for IPv4: {}", value);
        }

        let invalid_ipv4 = [
            json!("256.1.1.1"),
            json!("192.168.1"),
            json!("not.an.ip.address"),
            json!("192.168.1.1.1"),
        ];

        for value in invalid_ipv4 {
            let data = json!({"ip": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_err(), "Should fail for IPv4: {}", value);
        }

        // Test IPv6
        let mut rules = HashMap::new();
        rules.insert("ip".to_string(), vec![ipv6()]);

        let valid_ipv6 = [
            json!("2001:0db8:85a3:0000:0000:8a2e:0370:7334"),
            json!("::1"),
            json!("::"),
        ];

        for value in valid_ipv6 {
            let data = json!({"ip": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_ok(), "Failed for IPv6: {}", value);
        }
    }

    #[test]
    fn test_mac_address_rule() {
        let mut rules = HashMap::new();
        rules.insert("mac".to_string(), vec![mac_address()]);

        let valid_macs = [
            json!("00:1B:44:11:3A:B7"),
            json!("00-1B-44-11-3A-B7"),
            json!("FF:FF:FF:FF:FF:FF"),
        ];

        for value in valid_macs {
            let data = json!({"mac": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_ok(), "Failed for MAC: {}", value);
        }

        let invalid_macs = [
            json!("00:1B:44:11:3A"),      // Too short
            json!("GG:1B:44:11:3A:B7"),   // Invalid hex
            json!("00.1B.44.11.3A.B7"),   // Wrong separator
        ];

        for value in invalid_macs {
            let data = json!({"mac": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_err(), "Should fail for MAC: {}", value);
        }
    }

    #[test]
    fn test_multiple_of_rule() {
        let mut rules = HashMap::new();
        rules.insert("quantity".to_string(), vec![multiple_of(5)]);

        let valid_values = [
            json!(5),
            json!(10),
            json!(15),
            json!(0),
            json!("25"),
        ];

        for value in valid_values {
            let data = json!({"quantity": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_ok(), "Failed for value: {}", value);
        }

        let invalid_values = [
            json!(3),
            json!(7),
            json!(12),
            json!("13"),
        ];

        for value in invalid_values {
            let data = json!({"quantity": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_err(), "Should fail for value: {}", value);
        }
    }

    #[test]
    fn test_not_regex_rule() {
        let mut rules = HashMap::new();
        rules.insert("username".to_string(), vec![not_regex(r"^admin|root$")]);

        // Valid: doesn't match the pattern
        let valid_values = [
            json!("user123"),
            json!("johndoe"),
            json!("alice"),
        ];

        for value in valid_values {
            let data = json!({"username": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_ok(), "Failed for value: {}", value);
        }

        // Invalid: matches the forbidden pattern
        let invalid_values = [
            json!("admin"),
            json!("root"),
        ];

        for value in invalid_values {
            let data = json!({"username": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_err(), "Should fail for value: {}", value);
        }
    }

    #[test]
    fn test_present_rule() {
        let mut rules = HashMap::new();
        rules.insert("required_field".to_string(), vec![present()]);

        // Valid: field is present (even if empty)
        let data = json!({"required_field": ""});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_ok());

        let data = json!({"required_field": null});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_ok());

        // Invalid: field is not present
        let data = json!({});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_prohibited_rule() {
        let mut rules = HashMap::new();
        rules.insert("forbidden_field".to_string(), vec![prohibited()]);

        // Valid: field not present or empty
        let valid_values = [
            json!({}),  // Not present
            json!({"forbidden_field": null}),
            json!({"forbidden_field": ""}),
            json!({"forbidden_field": []}),
            json!({"forbidden_field": {}}),
        ];

        for data in valid_values {
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_ok());
        }

        // Invalid: field has a value
        let data = json!({"forbidden_field": "value"});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_conditional_required_rules() {
        // Test required_if
        let mut rules = HashMap::new();
        rules.insert("payment_method".to_string(), vec![required_if("type", "premium")]);

        // Valid: type is premium and payment_method is provided
        let data = json!({
            "type": "premium",
            "payment_method": "credit_card"
        });
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_ok());

        // Valid: type is not premium, payment_method not required
        let data = json!({"type": "free"});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_ok());

        // Invalid: type is premium but payment_method is missing
        let data = json!({"type": "premium"});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_required_with_rules() {
        let mut rules = HashMap::new();
        rules.insert("password_confirmation".to_string(), vec![required_with(vec!["password"])]);

        // Valid: password provided, confirmation provided
        let data = json!({
            "password": "secret123",
            "password_confirmation": "secret123"
        });
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_ok());

        // Valid: no password, no confirmation required
        let data = json!({});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_ok());

        // Invalid: password provided but confirmation missing
        let data = json!({"password": "secret123"});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_required_without_rules() {
        let mut rules = HashMap::new();
        rules.insert("email".to_string(), vec![required_without(vec!["phone"])]);

        // Valid: phone provided, email not required
        let data = json!({"phone": "555-1234"});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_ok());

        // Valid: no phone, but email provided
        let data = json!({"email": "user@example.com"});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_ok());

        // Invalid: no phone and no email
        let data = json!({});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_timezone_rule() {
        let mut rules = HashMap::new();
        rules.insert("tz".to_string(), vec![timezone()]);

        let valid_timezones = [
            json!("UTC"),
            json!("America/New_York"),
            json!("Europe/London"),
            json!("Asia/Tokyo"),
        ];

        for value in valid_timezones {
            let data = json!({"tz": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_ok(), "Failed for timezone: {}", value);
        }

        let invalid_timezones = [
            json!("Invalid/Timezone"),
            json!("NotATimezone"),
            json!("123"),
        ];

        for value in invalid_timezones {
            let data = json!({"tz": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_err(), "Should fail for timezone: {}", value);
        }
    }

    #[test]
    fn test_uuid_rule() {
        let mut rules = HashMap::new();
        rules.insert("id".to_string(), vec![uuid()]);

        let valid_uuids = [
            json!("550e8400-e29b-41d4-a716-446655440000"),
            json!("6ba7b810-9dad-11d1-80b4-00c04fd430c8"),
            json!("6ba7b811-9dad-11d1-80b4-00c04fd430c8"),
        ];

        for value in valid_uuids {
            let data = json!({"id": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_ok(), "Failed for UUID: {}", value);
        }

        let invalid_uuids = [
            json!("not-a-uuid"),
            json!("550e8400-e29b-41d4-a716"),  // Too short
            json!("550e8400-e29b-41d4-a716-446655440000-extra"),  // Too long
            json!("550e8400-e29b-41d4-g716-446655440000"),  // Invalid character
        ];

        for value in invalid_uuids {
            let data = json!({"id": value});
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_err(), "Should fail for UUID: {}", value);
        }
    }

    #[test]
    fn test_file_validation_rules() {
        let mut rules = HashMap::new();
        rules.insert("upload".to_string(), vec![file()]);

        // Valid file object
        let data = json!({
            "upload": {
                "filename": "document.pdf",
                "type": "application/pdf",
                "size": 1024
            }
        });
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_ok());

        // Invalid: not a file object
        let data = json!({"upload": "not a file"});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_err());

        // Test image rule
        let mut rules = HashMap::new();
        rules.insert("photo".to_string(), vec![image()]);

        let data = json!({
            "photo": {
                "filename": "photo.jpg",
                "type": "image/jpeg",
                "size": 2048
            }
        });
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_ok());

        // Invalid: not an image type
        let data = json!({
            "photo": {
                "filename": "document.pdf",
                "type": "application/pdf",
                "size": 1024
            }
        });
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_mimes_rule() {
        let mut rules = HashMap::new();
        rules.insert("file".to_string(), vec![mimes(vec!["jpg", "png", "pdf"])]);

        // Valid file extensions
        let valid_files = [
            json!({"file": {"filename": "photo.jpg"}}),
            json!({"file": {"filename": "image.png"}}),
            json!({"file": {"filename": "document.pdf"}}),
        ];

        for data in valid_files {
            let validator = create_validator_with_data(data, rules.clone());
            assert!(validator.validate().is_ok());
        }

        // Invalid file extension
        let data = json!({"file": {"filename": "video.mp4"}});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_array_validation_rules() {
        // Test array_min
        let mut rules = HashMap::new();
        rules.insert("tags".to_string(), vec![array(), array_min(2)]);

        let data = json!({"tags": ["tag1", "tag2", "tag3"]});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_ok());

        let data = json!({"tags": ["tag1"]});  // Too few items
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_err());

        // Test array_max
        let mut rules = HashMap::new();
        rules.insert("tags".to_string(), vec![array(), array_max(3)]);

        let data = json!({"tags": ["tag1", "tag2"]});
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_ok());

        let data = json!({"tags": ["tag1", "tag2", "tag3", "tag4"]});  // Too many items
        let validator = create_validator_with_data(data, rules.clone());
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_complex_validation_scenario() {
        // Test a complex scenario with multiple new rules
        let data = json!({
            "user_type": "premium",
            "email": "user@example.com",
            "payment_method": "credit_card",
            "ip_address": "192.168.1.100",
            "terms_accepted": true,
            "tags": ["web", "api", "rust"],
            "verification_code": "1234",
            "profile_url": "https://example.com/user",
            "user_id": "550e8400-e29b-41d4-a716-446655440000"
        });

        let mut rules = HashMap::new();
        rules.insert("payment_method".to_string(), vec![required_if("user_type", "premium")]);
        rules.insert("email".to_string(), vec![required(), email()]);
        rules.insert("ip_address".to_string(), vec![ipv4()]);
        rules.insert("terms_accepted".to_string(), vec![accepted()]);
        rules.insert("tags".to_string(), vec![array(), array_min(1), array_max(5), distinct()]);
        rules.insert("verification_code".to_string(), vec![digits(4)]);
        rules.insert("profile_url".to_string(), vec![starts_with(vec!["https://"])]);
        rules.insert("user_id".to_string(), vec![uuid()]);

        let validator = create_validator_with_data(data, rules);
        assert!(validator.validate().is_ok());
    }

    #[test]
    fn test_validation_with_new_macros() {
        let data = json!({
            "username": "johndoe123",
            "age": 25,
            "email": "john@example.com",
            "ip": "192.168.1.1",
            "tags": ["rust", "web", "api"],
            "terms": true
        });

        let result = validate!(data, {
            "username" => "required|alpha_dash|min:3|max:30",
            "age" => "required|integer|gte:18|lte:120",
            "email" => "required|email",
            "ip" => "required|ipv4",
            "tags" => "required|array|array_min:1|array_max:10|distinct",
            "terms" => "required|accepted"
        });

        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod custom_rules_tests {
    use super::*;
    use rustaxum::validation::CustomRules;

    #[test]
    fn test_custom_rules() {
        let data = json!({
            "username": "john_doe_123",
            "phone": "+1234567890",
            "color": "#FF5733",
            "uuid": "550e8400-e29b-41d4-a716-446655440000",
            "handle": "@johndoe"
        });

        let result = ValidatorBuilder::from_json(data)
            .rule("username", CustomRules::username())
            .rule("phone", CustomRules::phone_number())
            .rule("color", CustomRules::hex_color())
            .rule("uuid", CustomRules::uuid())
            .rule("handle", CustomRules::social_handle())
            .validate();

        assert!(result.is_ok());
    }

    #[test]
    fn test_password_strength() {
        let valid_passwords = [
            "StrongPass123!",
            "MySecure$Pass1",
            "Complex@Password9",
        ];

        for password in valid_passwords {
            let data = json!({"password": password});
            let result = ValidatorBuilder::from_json(data)
                .rule("password", CustomRules::password_strength())
                .validate();
            assert!(result.is_ok(), "Failed for password: {}", password);
        }

        let invalid_passwords = [
            "weakpass",      // No uppercase, numbers, or special chars
            "WEAKPASS",      // No lowercase, numbers, or special chars
            "WeakPass",      // No numbers or special chars
            "WeakPass123",   // No special chars
        ];

        for password in invalid_passwords {
            let data = json!({"password": password});
            let result = ValidatorBuilder::from_json(data)
                .rule("password", CustomRules::password_strength())
                .validate();
            assert!(result.is_err(), "Should fail for password: {}", password);
        }
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_multiple_validation_errors() {
        let data = json!({
            "email": "invalid-email",
            "age": 15,
            "username": "xy",
            "terms": false
        });

        let result = validate!(data, {
            "email" => "required|email",
            "age" => "required|integer|gte:18",
            "username" => "required|string|min:3",
            "terms" => "required|accepted"
        });

        assert!(result.is_err());

        if let Err(errors) = result {
            assert!(errors.has_errors());
            assert_eq!(errors.errors.len(), 4); // Should have 4 validation errors

            let messages = errors.get_messages();
            assert!(messages.contains_key("email"));
            assert!(messages.contains_key("age"));
            assert!(messages.contains_key("username"));
            assert!(messages.contains_key("terms"));
        }
    }

    #[test]
    fn test_validation_error_messages() {
        let data = json!({
            "code": "123"  // Should be 4 digits
        });

        let result = validate!(data, {
            "code" => "required|digits:4"
        }, {
            "code.digits" => "The verification code must be exactly 4 digits"
        });

        assert!(result.is_err());

        if let Err(errors) = result {
            let first_error = errors.first("code").unwrap();
            assert_eq!(first_error, "The verification code must be exactly 4 digits");
        }
    }
}