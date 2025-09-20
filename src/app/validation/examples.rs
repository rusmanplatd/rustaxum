use crate::app::validation::{Validatable, ValidationRules, ValidationErrors, validate_json_async};
use crate::validation_rules;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

// Example struct implementing Validatable trait
#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub body: String,
    pub author_email: String,
    pub tags: Vec<String>,
}

impl Validatable for CreatePostRequest {
    fn validation_rules() -> ValidationRules {
        validation_rules! {
            "title" => ["required", "string", "max:255"],
            "body" => ["required", "string"],
            "author_email" => ["required", "email"],
            "tags" => ["array"],
        }
    }
}

// Example with nested validation
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub profile: UserProfile,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub bio: Option<String>,
    pub age: u32,
    pub social_links: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub action: String,
}

impl Validatable for CreateUserRequest {
    fn validation_rules() -> ValidationRules {
        validation_rules! {
            "name" => ["required", "string", "min:2", "max:100"],
            "email" => ["required", "email", "unique:users"],
            "profile.age" => ["required", "integer", "min:13", "max:120"],
            "profile.bio" => ["string", "max:500"],
            "permissions" => ["required", "array"],
            "permissions.*.resource" => ["required", "string"],
            "permissions.*.action" => ["required", "string"],
        }
    }
}

// Example usage functions
pub async fn example_basic_validation() -> Result<(), ValidationErrors> {
    let post_data = CreatePostRequest {
        title: "My Blog Post".to_string(),
        body: "This is the content of my blog post.".to_string(),
        author_email: "author@example.com".to_string(),
        tags: vec!["rust".to_string(), "web".to_string()],
    };

    // Validate the struct
    post_data.validate().await?;
    println!("Post validation successful!");

    Ok(())
}

pub async fn example_nested_validation() -> Result<(), ValidationErrors> {
    let user_data = CreateUserRequest {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        profile: UserProfile {
            bio: Some("Software developer".to_string()),
            age: 30,
            social_links: HashMap::from([
                ("twitter".to_string(), "@johndoe".to_string()),
                ("github".to_string(), "johndoe".to_string()),
            ]),
        },
        permissions: vec![
            Permission {
                resource: "posts".to_string(),
                action: "read".to_string(),
            },
            Permission {
                resource: "users".to_string(),
                action: "create".to_string(),
            },
        ],
    };

    // Validate with nested rules
    user_data.validate().await?;
    println!("User validation successful!");

    Ok(())
}

pub async fn example_json_validation() -> Result<(), ValidationErrors> {
    let data = json!({
        "team_name": "Development Team",
        "members": [
            {
                "name": "Alice",
                "role": "Lead Developer"
            },
            {
                "name": "Bob",
                "role": "Backend Developer"
            }
        ],
        "authorization": {
            "role": "admin"
        }
    });

    let rules = validation_rules! {
        "team_name" => ["required", "string", "min:1"],
        "members" => ["required", "array"],
        "members.*.name" => ["required", "string"],
        "members.*.role" => ["required", "string"],
        "authorization.role" => ["required", "string"],
    };

    validate_json_async(data, rules).await?;
    println!("JSON validation successful!");

    Ok(())
}

pub async fn example_validation_errors() {
    let data = json!({
        "team_name": 123, // Should be string
        "authorization": {
            "role": "" // Should not be empty
        }
    });

    let rules = validation_rules! {
        "team_name" => ["required", "string", "min:1"],
        "authorization.role" => ["required"],
    };

    match validate_json_async(data, rules).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(errors) => {
            println!("Validation failed as expected:");
            println!("Message: {}", errors.message);
            println!("Errors: {:#?}", errors.errors);

            // Example output:
            // Message: "The team_name must be a string. (and 1 more errors)"
            // Errors: {
            //     "team_name": {
            //         "string": "The team_name must be a string.",
            //         "min": "The team_name must be at least 1 characters."
            //     },
            //     "authorization.role": {
            //         "required": "authorization.role is required."
            //     }
            // }
        }
    }
}