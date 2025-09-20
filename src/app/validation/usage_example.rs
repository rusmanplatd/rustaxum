// Example demonstrating the new validation system usage
// This replaces the old validation pattern with the new Laravel-style array-based rules

use crate::app::validation::{
    Validatable, ValidationRules, ValidationErrors,
    validate_json_async
};
use crate::validation_rules;
use serde::{Deserialize, Serialize};
use serde_json::json;

// Example 1: Basic validation with the Validatable trait
#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub body: String,
}

impl Validatable for CreatePostRequest {
    fn validation_rules() -> ValidationRules {
        validation_rules! {
            "title" => ["required", "string", "max:255"],
            "body" => ["required", "string"],
        }
    }
}

// Example 2: Complex nested validation
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub authorization: Authorization,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Authorization {
    pub role: String,
}

impl Validatable for CreateUserRequest {
    fn validation_rules() -> ValidationRules {
        validation_rules! {
            "name" => ["required", "string", "min:1"],
            "email" => ["required", "email", "unique:users"],
            "authorization.role" => ["required", "string"],
        }
    }
}

// Example usage functions
pub async fn example_usage() -> Result<(), ValidationErrors> {
    // Example 1: Using the trait
    let post = CreatePostRequest {
        title: "My Post".to_string(),
        body: "Post content".to_string(),
    };

    post.validate().await?;
    println!("Post validation passed!");

    // Example 2: Direct JSON validation (like the old way but with new syntax)
    let data = json!({
        "title": "Test",
        "body": "Content"
    });

    let rules = validation_rules! {
        "title" => ["required", "string", "max:255"],
        "body" => ["required", "string"],
    };

    validate_json_async(data, rules).await?;
    println!("JSON validation passed!");

    Ok(())
}

// Migration guide for existing code:
//
// OLD WAY (before):
// use crate::app::utils::validator::{Rule, required, string, max};
// let rules = vec![required(), string(), max(255)];
//
// NEW WAY (now):
// use crate::app::validation::{validation_rules, validate_json_async};
// let rules = validation_rules! {
//     "field" => ["required", "string", "max:255"]
// };
//
// The validation error format matches exactly what was requested:
// {
//     "message": "The team name must be a string. (and 1 more errors)",
//     "errors": {
//         "team_name": {
//             "string": "The team name must be a string.",
//             "min": "The team name must be at least 1 characters."
//         },
//         "authorization.role": {
//             "required": "authorization.role cannot be empty."
//         }
//     }
// }