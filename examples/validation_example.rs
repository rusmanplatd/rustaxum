use std::collections::HashMap;
use serde_json::{json, Value};

// Import validation components
use rustaxum::validation::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Laravel-like Validation System Demo\n");

    // Example 1: Basic validation using macros
    println!("=== Example 1: Basic Validation ===");
    let user_data = json!({
        "name": "John Doe",
        "email": "john@example.com",
        "age": 25,
        "website": "https://johndoe.com"
    });

    let result = validate!(user_data, {
        "name" => "required|string|min:2|max:50",
        "email" => "required|email",
        "age" => "required|integer|min:18|max:120",
        "website" => "nullable|url"
    });

    match result {
        Ok(_) => println!("✓ Validation passed!"),
        Err(errors) => {
            println!("✗ Validation failed:");
            for error in &errors.errors {
                println!("  - {}: {}", error.field, error.message);
            }
        }
    }

    // Example 2: Validation with custom messages
    println!("\n=== Example 2: Custom Messages ===");
    let invalid_data = json!({
        "username": "jo",
        "email": "invalid-email",
        "password": "weak"
    });

    let result = validate!(invalid_data, {
        "username" => "required|string|min:3|max:30",
        "email" => "required|email",
        "password" => "required|string|min:8"
    }, {
        "username.min" => "Username must be at least 3 characters long",
        "email.email" => "Please provide a valid email address",
        "password.min" => "Password must be at least 8 characters long"
    });

    if let Err(errors) = result {
        println!("Validation errors with custom messages:");
        let messages = errors.get_messages();
        for (field, field_errors) in messages {
            for error in field_errors {
                println!("  - {}: {}", field, error);
            }
        }
    }

    // Example 3: Builder pattern
    println!("\n=== Example 3: Builder Pattern ===");
    let registration_data = json!({
        "name": "Jane Smith",
        "email": "jane@example.com",
        "password": "SecurePass123!",
        "password_confirmation": "SecurePass123!",
        "terms": true,
        "age": 28
    });

    let result = ValidatorBuilder::from_json(registration_data)
        .rules_from_string("name", "required|string|min:2|max:100")
        .rules_from_string("email", "required|email|max:255")
        .rules_from_string("password", "required|string|min:8|confirmed")
        .rules_from_string("terms", "required|boolean")
        .rules_from_string("age", "required|integer|min:18|max:120")
        .message("terms.required", "You must accept the terms and conditions")
        .validate();

    match result {
        Ok(_) => println!("✓ Registration data is valid!"),
        Err(errors) => {
            println!("✗ Registration validation failed:");
            for error in &errors.errors {
                println!("  - {}: {}", error.field, error.message);
            }
        }
    }

    // Example 4: Common validators
    println!("\n=== Example 4: Common Validators ===");
    let login_data = json!({
        "email": "user@example.com",
        "password": "mypassword"
    });

    match CommonValidators::user_login(login_data) {
        Ok(_) => println!("✓ Login data is valid!"),
        Err(errors) => {
            println!("✗ Login validation failed:");
            for error in &errors.errors {
                println!("  - {}: {}", error.field, error.message);
            }
        }
    }

    // Example 5: Custom rules
    println!("\n=== Example 5: Custom Rules ===");
    let profile_data = json!({
        "username": "john_doe_123",
        "phone": "+1234567890",
        "color": "#FF5733",
        "social_handle": "@johndoe"
    });

    let result = ValidatorBuilder::from_json(profile_data)
        .rule("username", CustomRules::username())
        .rule("phone", CustomRules::phone_number())
        .rule("color", CustomRules::hex_color())
        .rule("social_handle", CustomRules::social_handle())
        .validate();

    match result {
        Ok(_) => println!("✓ Profile data is valid!"),
        Err(errors) => {
            println!("✗ Profile validation failed:");
            for error in &errors.errors {
                println!("  - {}: {}", error.field, error.message);
            }
        }
    }

    // Example 6: Complex validation scenario
    println!("\n=== Example 6: Complex Validation ===");
    let complex_data = json!({
        "user": {
            "name": "Admin User",
            "email": "admin@example.com"
        },
        "product": {
            "name": "Awesome Product",
            "price": 99.99,
            "category": "electronics",
            "tags": ["gadget", "tech", "new"]
        },
        "metadata": {
            "created_at": "2023-12-25",
            "settings": {
                "notifications": true,
                "theme": "dark"
            }
        }
    });

    // Flatten nested data for validation
    let flat_data = json!({
        "user_name": complex_data["user"]["name"],
        "user_email": complex_data["user"]["email"],
        "product_name": complex_data["product"]["name"],
        "product_price": complex_data["product"]["price"],
        "product_category": complex_data["product"]["category"],
        "product_tags": complex_data["product"]["tags"],
        "created_at": complex_data["metadata"]["created_at"],
        "notifications": complex_data["metadata"]["settings"]["notifications"],
        "theme": complex_data["metadata"]["settings"]["theme"]
    });

    let result = validate!(flat_data, {
        "user_name" => "required|string|min:2|max:100",
        "user_email" => "required|email",
        "product_name" => "required|string|min:3|max:200",
        "product_price" => "required|numeric|min:0|max:9999.99",
        "product_category" => "required|string|in:electronics,clothing,books,home",
        "product_tags" => "required|array|min:1|max:10",
        "created_at" => "required|date",
        "notifications" => "required|boolean",
        "theme" => "required|string|in:light,dark,auto"
    });

    match result {
        Ok(_) => println!("✓ Complex data validation passed!"),
        Err(errors) => {
            println!("✗ Complex data validation failed:");
            for error in &errors.errors {
                println!("  - {}: {}", error.field, error.message);
            }
        }
    }

    // Example 7: Validation utility functions
    println!("\n=== Example 7: Validation Utils ===");
    let mut errors = ValidationErrors::new();
    errors.add(ValidationError::new("name", "required", "Name is required"));
    errors.add(ValidationError::new("email", "email", "Email is invalid"));
    errors.add(ValidationError::new("name", "min", "Name too short"));

    println!("Error summary:");
    let summary = ValidationUtils::error_summary(&errors);
    for (field, count) in summary {
        println!("  - {}: {} error(s)", field, count);
    }

    println!("\nError string: {}", ValidationUtils::errors_to_string(&errors));

    println!("\nHas name required error: {}",
             ValidationUtils::has_error(&errors, "name", "required"));

    // Example 8: New validation rules showcase
    println!("\n=== Example 8: New Validation Rules ===");
    let advanced_data = json!({
        "user_type": "premium",
        "email": "admin@company.com",
        "verification_code": "1234",
        "ip_address": "192.168.1.100",
        "mac_address": "00:1B:44:11:3A:B7",
        "terms_accepted": "yes",
        "tags": ["web", "api", "rust", "backend"],
        "user_id": "550e8400-e29b-41d4-a716-446655440000",
        "timezone": "America/New_York",
        "profile_image": {
            "filename": "avatar.jpg",
            "type": "image/jpeg",
            "width": 200,
            "height": 200
        },
        "quantity": 15,
        "website": "https://company.com",
        "bio": "Software engineer passionate about Rust and web development.",
        "admin_field": null
    });

    let result = validate!(advanced_data, {
        "email" => "required|email|not_regex:^admin|root",
        "verification_code" => "required|digits:4",
        "ip_address" => "required|ipv4",
        "mac_address" => "required|mac_address",
        "terms_accepted" => "required|accepted",
        "tags" => "required|array|array_min:1|array_max:10|distinct",
        "user_id" => "required|uuid",
        "timezone" => "required|timezone",
        "profile_image" => "required|image|dimensions",
        "quantity" => "required|integer|multiple_of:5",
        "website" => "required|url|starts_with:https://",
        "bio" => "required|string|min:10|max:500|filled",
        "admin_field" => "prohibited"
    });

    match result {
        Ok(_) => println!("✓ Advanced validation passed!"),
        Err(errors) => {
            println!("✗ Advanced validation failed:");
            for error in &errors.errors {
                println!("  - {}: {}", error.field, error.message);
            }
        }
    }

    // Example 9: Conditional validation
    println!("\n=== Example 9: Conditional Validation ===");
    let conditional_data = json!({
        "account_type": "business",
        "company_name": "Tech Corp",
        "tax_id": "12-3456789",
        "personal_name": "",
        "contact_email": "contact@techcorp.com",
        "backup_email": ""
    });

    let result = validate!(conditional_data, {
        "company_name" => "required_if:account_type,business",
        "tax_id" => "required_if:account_type,business|digits_between:9,12",
        "personal_name" => "required_if:account_type,personal",
        "contact_email" => "required|email",
        "backup_email" => "required_with:contact_email"
    });

    match result {
        Ok(_) => println!("✓ Conditional validation passed!"),
        Err(errors) => {
            println!("✗ Conditional validation failed:");
            for error in &errors.errors {
                println!("  - {}: {}", error.field, error.message);
            }
        }
    }

    // Example 10: File and MIME type validation
    println!("\n=== Example 10: File Validation ===");
    let file_data = json!({
        "avatar": {
            "filename": "profile.png",
            "type": "image/png",
            "size": 2048,
            "width": 300,
            "height": 300
        },
        "document": {
            "filename": "contract.pdf",
            "type": "application/pdf",
            "size": 51200
        },
        "attachments": [
            {"filename": "doc1.docx", "type": "application/vnd.openxmlformats-officedocument.wordprocessingml.document"},
            {"filename": "sheet1.xlsx", "type": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"}
        ]
    });

    let flat_file_data = json!({
        "avatar": file_data["avatar"],
        "document": file_data["document"],
        "attachments": file_data["attachments"]
    });

    let result = validate!(flat_file_data, {
        "avatar" => "required|image|mimes:jpg,png,gif|dimensions",
        "document" => "required|file|mimes:pdf,doc,docx",
        "attachments" => "required|array|array_min:1|array_max:5"
    });

    match result {
        Ok(_) => println!("✓ File validation passed!"),
        Err(errors) => {
            println!("✗ File validation failed:");
            for error in &errors.errors {
                println!("  - {}: {}", error.field, error.message);
            }
        }
    }

    // Example 11: International validation patterns
    println!("\n=== Example 11: International Validation ===");
    let international_data = json!({
        "iban": "GB82WEST12345698765432",
        "phone": "+1-555-123-4567",
        "locale": "en_US",
        "currency": "USD",
        "country_code": "US"
    });

    let result = ValidatorBuilder::from_json(international_data)
        .rules_from_string("phone", "required|string")
        .rules_from_string("locale", "required|string|regex:^[a-z]{2}_[A-Z]{2}$")
        .rules_from_string("currency", "required|string|size:3|alpha")
        .rules_from_string("country_code", "required|string|size:2|alpha")
        .message("locale.regex", "Locale must be in format: language_COUNTRY (e.g., en_US)")
        .validate();

    match result {
        Ok(_) => println!("✓ International validation passed!"),
        Err(errors) => {
            println!("✗ International validation failed:");
            for error in &errors.errors {
                println!("  - {}: {}", error.field, error.message);
            }
        }
    }

    println!("\n=== All Examples Complete! ===");
    println!("🎉 Laravel-like validation system successfully demonstrated!");
    println!("\n📊 **Validation Rules Summary:**");
    println!("✅ 50+ validation rules implemented");
    println!("✅ Laravel-compatible syntax with pipe separation");
    println!("✅ Custom error messages support");
    println!("✅ Conditional validation rules");
    println!("✅ File and MIME type validation");
    println!("✅ Network and format validation (IP, MAC, UUID, etc.)");
    println!("✅ Array validation with size constraints");
    println!("✅ Builder pattern and macro support");
    println!("✅ Comprehensive error handling");

    Ok(())
}