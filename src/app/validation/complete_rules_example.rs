// Complete Laravel validation rules implementation example
// This demonstrates all implemented validation rules

use crate::app::validation::{Validatable, ValidationRules, validation_rules, validate_json_async};
use crate::validation_rules;
use serde::{Deserialize, Serialize};
use serde_json::json;

// Comprehensive example showing all validation rules
pub async fn demonstrate_all_laravel_rules() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Demonstrating all Laravel validation rules in Rust!");

    // 1. STRING VALIDATION RULES
    println!("\n📝 STRING VALIDATION RULES");

    let string_data = json!({
        "name": "JohnDoe123",
        "username": "john_doe-2024",
        "code": "ABC123",
        "description": "This is a sample description",
        "phone": "1234567890"
    });

    let string_rules = validation_rules! {
        "name" => ["required", "string", "alpha_num"],
        "username" => ["required", "string", "alpha_dash"],
        "code" => ["required", "alpha"],
        "description" => ["string", "min:5", "max:100"],
        "phone" => ["digits:10"]
    };

    match validate_json_async(string_data, string_rules).await {
        Ok(_) => println!("✅ String validation passed"),
        Err(e) => println!("❌ String validation failed: {}", e),
    }

    // 2. NUMERIC VALIDATION RULES
    println!("\n🔢 NUMERIC VALIDATION RULES");

    let numeric_data = json!({
        "age": 25,
        "score": 85.5,
        "rating": 4,
        "percentage": 75.2,
        "count": 42
    });

    let numeric_rules = validation_rules! {
        "age" => ["required", "integer", "between:18,100"],
        "score" => ["numeric", "between:0,100"],
        "rating" => ["integer", "between:1,5"],
        "percentage" => ["numeric", "min:0", "max:100"],
        "count" => ["integer", "size:42"]
    };

    match validate_json_async(numeric_data, numeric_rules).await {
        Ok(_) => println!("✅ Numeric validation passed"),
        Err(e) => println!("❌ Numeric validation failed: {}", e),
    }

    // 3. DATE VALIDATION RULES
    println!("\n📅 DATE VALIDATION RULES");

    let date_data = json!({
        "birth_date": "1990-05-15",
        "start_date": "2024-01-01",
        "end_date": "2024-12-31",
        "formatted_date": "15/05/1990"
    });

    let date_rules = validation_rules! {
        "birth_date" => ["date", "before:today"],
        "start_date" => ["date", "after:2023-12-31"],
        "end_date" => ["date", "after:start_date"],
        "formatted_date" => ["date_format:%d/%m/%Y"]
    };

    match validate_json_async(date_data, date_rules).await {
        Ok(_) => println!("✅ Date validation passed"),
        Err(e) => println!("❌ Date validation failed: {}", e),
    }

    // 4. ARRAY AND CHOICE VALIDATION RULES
    println!("\n📋 ARRAY AND CHOICE VALIDATION RULES");

    let choice_data = json!({
        "status": "active",
        "priority": "high",
        "tags": ["rust", "web", "api"],
        "excluded_status": "pending"
    });

    let choice_rules = validation_rules! {
        "status" => ["required", "in:active,inactive,pending"],
        "priority" => ["in:low,medium,high"],
        "tags" => ["array", "min:1", "max:5"],
        "excluded_status" => ["not_in:deleted,archived"]
    };

    match validate_json_async(choice_data, choice_rules).await {
        Ok(_) => println!("✅ Choice validation passed"),
        Err(e) => println!("❌ Choice validation failed: {}", e),
    }

    // 5. FORMAT VALIDATION RULES
    println!("\n🌐 FORMAT VALIDATION RULES");

    let format_data = json!({
        "email": "user@example.com",
        "website": "https://example.com",
        "uuid": "550e8400-e29b-41d4-a716-446655440000",
        "ulid": "01F8MECHZX3TBDSZ7XR8G62H8V",
        "ip_address": "192.168.1.1",
        "json_data": "{\"key\": \"value\"}"
    });

    let format_rules = validation_rules! {
        "email" => ["required", "email"],
        "website" => ["url"],
        "uuid" => ["uuid"],
        "ulid" => ["ulid"],
        "ip_address" => ["ip"],
        "json_data" => ["json"]
    };

    match validate_json_async(format_data, format_rules).await {
        Ok(_) => println!("✅ Format validation passed"),
        Err(e) => println!("❌ Format validation failed: {}", e),
    }

    // 6. FILE VALIDATION RULES
    println!("\n📁 FILE VALIDATION RULES");

    let file_data = json!({
        "avatar": "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQ...",
        "document": "contract.pdf",
        "image": "photo.jpg"
    });

    let file_rules = validation_rules! {
        "avatar" => ["file", "image"],
        "document" => ["file", "mimes:application/pdf,text/plain"],
        "image" => ["image", "mimes:image/jpeg,image/png"]
    };

    match validate_json_async(file_data, file_rules).await {
        Ok(_) => println!("✅ File validation passed"),
        Err(e) => println!("❌ File validation failed: {}", e),
    }

    // 7. CONDITIONAL VALIDATION RULES
    println!("\n🔀 CONDITIONAL VALIDATION RULES");

    let conditional_data = json!({
        "user_type": "admin",
        "admin_key": "secret123",
        "password": "mypassword",
        "password_confirmation": "mypassword",
        "email": "admin@example.com",
        "backup_email": "backup@example.com"
    });

    let conditional_rules = validation_rules! {
        "user_type" => ["required", "in:admin,user,guest"],
        "admin_key" => ["required_if:user_type,admin"],
        "password" => ["required", "min:8", "confirmed"],
        "password_confirmation" => ["required"],
        "email" => ["required", "email"],
        "backup_email" => ["email", "different:email"]
    };

    match validate_json_async(conditional_data, conditional_rules).await {
        Ok(_) => println!("✅ Conditional validation passed"),
        Err(e) => println!("❌ Conditional validation failed: {}", e),
    }

    // 8. ADVANCED STRING RULES
    println!("\n🔍 ADVANCED STRING RULES");

    let advanced_data = json!({
        "phone_code": "+1",
        "filename": "document.pdf",
        "slug": "my-blog-post-2024",
        "pattern_field": "ABC-123-DEF"
    });

    let advanced_rules = validation_rules! {
        "phone_code" => ["starts_with:+"],
        "filename" => ["ends_with:.pdf"],
        "slug" => ["alpha_dash"],
        "pattern_field" => ["regex:^[A-Z]{3}-[0-9]{3}-[A-Z]{3}$"]
    };

    match validate_json_async(advanced_data, advanced_rules).await {
        Ok(_) => println!("✅ Advanced string validation passed"),
        Err(e) => println!("❌ Advanced string validation failed: {}", e),
    }

    // 9. NESTED VALIDATION EXAMPLE
    println!("\n🏗️ NESTED VALIDATION RULES");

    let nested_data = json!({
        "user": {
            "name": "John Doe",
            "email": "john@example.com",
            "profile": {
                "age": 30,
                "bio": "Software developer"
            }
        },
        "tags": [
            {"name": "rust", "priority": 1},
            {"name": "web", "priority": 2}
        ]
    });

    let nested_rules = validation_rules! {
        "user.name" => ["required", "string", "min:2"],
        "user.email" => ["required", "email"],
        "user.profile.age" => ["integer", "between:18,100"],
        "user.profile.bio" => ["string", "max:200"],
        "tags" => ["array", "min:1"],
        "tags.*.name" => ["required", "string", "alpha"],
        "tags.*.priority" => ["required", "integer", "min:1"]
    };

    match validate_json_async(nested_data, nested_rules).await {
        Ok(_) => println!("✅ Nested validation passed"),
        Err(e) => println!("❌ Nested validation failed: {}", e),
    }

    println!("\n🎉 All Laravel validation rules demonstrated!");
    Ok(())
}

// Complete list of all implemented Laravel validation rules
pub fn list_all_implemented_rules() {
    println!("📚 COMPLETE LIST OF IMPLEMENTED LARAVEL VALIDATION RULES:");

    println!("\n🔤 STRING RULES:");
    println!("  • required - Field must be present and not empty");
    println!("  • string - Field must be a string");
    println!("  • alpha - Field may only contain letters");
    println!("  • alpha_dash - Field may contain letters, numbers, dashes, and underscores");
    println!("  • alpha_num - Field may only contain letters and numbers");
    println!("  • min:value - Field must be at least the given length");
    println!("  • max:value - Field may not be greater than the given length");
    println!("  • size:value - Field must be exactly the given size");
    println!("  • starts_with:value - Field must start with the given value");
    println!("  • ends_with:value - Field must end with the given value");

    println!("\n🔢 NUMERIC RULES:");
    println!("  • numeric - Field must be numeric");
    println!("  • integer - Field must be an integer");
    println!("  • between:min,max - Field must be between the given min and max");
    println!("  • digits:value - Field must be exactly the given number of digits");
    println!("  • digits_between:min,max - Field must be between the given number of digits");

    println!("\n📅 DATE RULES:");
    println!("  • date - Field must be a valid date");
    println!("  • before:date - Field must be a date before the given date");
    println!("  • after:date - Field must be a date after the given date");
    println!("  • date_format:format - Field must match the given date format");

    println!("\n📋 ARRAY RULES:");
    println!("  • array - Field must be an array");
    println!("  • in:val1,val2,... - Field must be one of the given values");
    println!("  • not_in:val1,val2,... - Field must not be one of the given values");

    println!("\n🌐 FORMAT RULES:");
    println!("  • email - Field must be a valid email address");
    println!("  • url - Field must be a valid URL");
    println!("  • uuid - Field must be a valid UUID");
    println!("  • ulid - Field must be a valid ULID");
    println!("  • ip - Field must be a valid IP address");
    println!("  • json - Field must be valid JSON");
    println!("  • regex:pattern - Field must match the given regular expression");

    println!("\n📁 FILE RULES:");
    println!("  • file - Field must be a file");
    println!("  • image - Field must be an image");
    println!("  • mimes:type1,type2,... - Field must be one of the given MIME types");

    println!("\n🔀 CONDITIONAL RULES:");
    println!("  • required_if:field,value - Field is required if another field equals value");
    println!("  • required_unless:field,value - Field is required unless another field equals value");
    println!("  • confirmed - Field must have a matching confirmation field");
    println!("  • same:field - Field must be the same as another field");
    println!("  • different:field - Field must be different from another field");

    println!("\n🗄️ DATABASE RULES:");
    println!("  • unique:table[,column] - Field must be unique in the database");
    println!("  • exists:table[,column] - Field must exist in the database");

    println!("\n🆔 TYPE RULES:");
    println!("  • boolean - Field must be a boolean value");

    println!("\n✨ All rules support nested field validation with dot notation (e.g., 'user.email')");
    println!("✨ All rules support array validation with wildcard notation (e.g., 'users.*.name')");
}