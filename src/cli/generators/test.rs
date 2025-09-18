use anyhow::Result;
use std::fs;
use std::path::Path;

pub async fn generate_test(name: &str, unit: bool) -> Result<()> {
    let test_name = if name.ends_with("Test") {
        name.to_string()
    } else {
        format!("{}Test", name)
    };

    let (dir_path, content) = if unit {
        ("tests/unit", generate_unit_test_template(&test_name))
    } else {
        ("tests/feature", generate_feature_test_template(&test_name))
    };

    fs::create_dir_all(dir_path)?;

    let file_path = format!("{}/{}.rs", dir_path, to_snake_case(&test_name));

    fs::write(&file_path, content)?;

    update_test_mod(&test_name, unit)?;

    println!("Test created successfully: {}", file_path);
    Ok(())
}

fn generate_unit_test_template(test_name: &str) -> String {
    let module_name = test_name.replace("Test", "").to_lowercase();
    format!(r#"use anyhow::Result;

// Import the module you want to test
// use rustaxum::app::models::SomeModel;
// use rustaxum::app::services::SomeService;

#[cfg(test)]
mod {} {{
    use super::*;

    #[tokio::test]
    async fn test_example() -> Result<()> {{
        // Arrange
        let input = "test input";

        // Act
        let result = example_function(input).await?;

        // Assert
        assert_eq!(result, "expected output");
        Ok(())
    }}

    #[tokio::test]
    async fn test_example_with_error() -> Result<()> {{
        // Arrange
        let invalid_input = "";

        // Act & Assert
        let result = example_function(invalid_input).await;
        assert!(result.is_err());
        Ok(())
    }}

    #[test]
    fn test_synchronous_function() {{
        // Arrange
        let input = 42;

        // Act
        let result = sync_example_function(input);

        // Assert
        assert_eq!(result, 84);
    }}

    #[tokio::test]
    async fn test_with_mock() -> Result<()> {{
        // Example of testing with mocks
        // You might want to use mockall or similar crate for mocking

        // Arrange
        let mock_data = "mocked data";

        // Act
        let result = function_using_external_service(mock_data).await?;

        // Assert
        assert!(result.contains("mocked"));
        Ok(())
    }}
}}

// Example functions to test (replace with your actual functions)
async fn example_function(input: &str) -> Result<String> {{
    if input.is_empty() {{
        return Err(anyhow::anyhow!("Input cannot be empty"));
    }}
    Ok(format!("Processed: {{}}", input))
}}

fn sync_example_function(input: i32) -> i32 {{
    input * 2
}}

async fn function_using_external_service(data: &str) -> Result<String> {{
    // Simulate an external service call
    Ok(format!("Service response for: {{}}", data))
}}
"#, module_name)
}

fn generate_feature_test_template(test_name: &str) -> String {
    let feature_name = test_name.replace("Test", "").to_lowercase();
    format!(r#"use anyhow::Result;
use axum::{{
    body::Body,
    http::{{Request, StatusCode}},
    response::Response,
}};
use axum_test::TestServer;
use serde_json::{{json, Value}};
use tokio_test;

// Import your application
use rustaxum::{{create_app, config::Config}};

#[cfg(test)]
mod {} {{
    use super::*;

    async fn setup_test_server() -> TestServer {{
        let config = Config::from_env();
        let app = create_app(config).await.expect("Failed to create app");
        TestServer::new(app).expect("Failed to create test server")
    }}

    #[tokio::test]
    async fn test_get_endpoint() -> Result<()> {{
        // Arrange
        let server = setup_test_server().await;

        // Act
        let response = server
            .get("/api/test-endpoint")
            .await;

        // Assert
        assert_eq!(response.status_code(), StatusCode::OK);

        let body: Value = response.json();
        assert!(body.get("success").unwrap().as_bool().unwrap());

        Ok(())
    }}

    #[tokio::test]
    async fn test_post_endpoint() -> Result<()> {{
        // Arrange
        let server = setup_test_server().await;
        let payload = json!({{
            "name": "Test Name",
            "email": "test@example.com"
        }});

        // Act
        let response = server
            .post("/api/test-endpoint")
            .json(&payload)
            .await;

        // Assert
        assert_eq!(response.status_code(), StatusCode::CREATED);

        let body: Value = response.json();
        assert_eq!(body["name"], "Test Name");
        assert_eq!(body["email"], "test@example.com");

        Ok(())
    }}

    #[tokio::test]
    async fn test_authentication_required() -> Result<()> {{
        // Arrange
        let server = setup_test_server().await;

        // Act
        let response = server
            .get("/api/protected-endpoint")
            .await;

        // Assert
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

        Ok(())
    }}

    #[tokio::test]
    async fn test_with_authentication() -> Result<()> {{
        // Arrange
        let server = setup_test_server().await;

        // First, authenticate to get a token
        let auth_payload = json!({{
            "email": "test@example.com",
            "password": "password123"
        }});

        let auth_response = server
            .post("/api/auth/login")
            .json(&auth_payload)
            .await;

        assert_eq!(auth_response.status_code(), StatusCode::OK);

        let auth_body: Value = auth_response.json();
        let token = auth_body["token"].as_str().unwrap();

        // Act - Use the token to access protected endpoint
        let response = server
            .get("/api/protected-endpoint")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {{}}", token).parse().unwrap())
            .await;

        // Assert
        assert_eq!(response.status_code(), StatusCode::OK);

        Ok(())
    }}

    #[tokio::test]
    async fn test_validation_error() -> Result<()> {{
        // Arrange
        let server = setup_test_server().await;
        let invalid_payload = json!({{
            "name": "", // Invalid: empty name
            "email": "invalid-email" // Invalid: not a valid email
        }});

        // Act
        let response = server
            .post("/api/test-endpoint")
            .json(&invalid_payload)
            .await;

        // Assert
        assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);

        let body: Value = response.json();
        assert!(body.get("errors").is_some());

        Ok(())
    }}

    #[tokio::test]
    async fn test_not_found() -> Result<()> {{
        // Arrange
        let server = setup_test_server().await;

        // Act
        let response = server
            .get("/api/non-existent-endpoint")
            .await;

        // Assert
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);

        Ok(())
    }}

    #[tokio::test]
    async fn test_database_interaction() -> Result<()> {{
        // Arrange
        let server = setup_test_server().await;

        // Create test data
        let create_payload = json!({{
            "name": "Test Item",
            "description": "Test Description"
        }});

        // Act - Create
        let create_response = server
            .post("/api/items")
            .json(&create_payload)
            .await;

        // Assert - Create
        assert_eq!(create_response.status_code(), StatusCode::CREATED);
        let created_item: Value = create_response.json();
        let item_id = created_item["id"].as_str().unwrap();

        // Act - Read
        let get_response = server
            .get(&format!("/api/items/{{}}", item_id))
            .await;

        // Assert - Read
        assert_eq!(get_response.status_code(), StatusCode::OK);
        let retrieved_item: Value = get_response.json();
        assert_eq!(retrieved_item["name"], "Test Item");

        // Act - Update
        let update_payload = json!({{
            "name": "Updated Test Item",
            "description": "Updated Description"
        }});

        let update_response = server
            .put(&format!("/api/items/{{}}", item_id))
            .json(&update_payload)
            .await;

        // Assert - Update
        assert_eq!(update_response.status_code(), StatusCode::OK);
        let updated_item: Value = update_response.json();
        assert_eq!(updated_item["name"], "Updated Test Item");

        // Act - Delete
        let delete_response = server
            .delete(&format!("/api/items/{{}}", item_id))
            .await;

        // Assert - Delete
        assert_eq!(delete_response.status_code(), StatusCode::NO_CONTENT);

        // Verify deletion
        let get_deleted_response = server
            .get(&format!("/api/items/{{}}", item_id))
            .await;

        assert_eq!(get_deleted_response.status_code(), StatusCode::NOT_FOUND);

        Ok(())
    }}
}}
"#, feature_name)
}

fn update_test_mod(test_name: &str, unit: bool) -> Result<()> {
    let mod_path = if unit {
        "tests/unit/mod.rs"
    } else {
        "tests/feature/mod.rs"
    };

    let module_name = to_snake_case(test_name);

    let mod_dir = if unit { "tests/unit" } else { "tests/feature" };
    if !Path::new(mod_dir).exists() {
        fs::create_dir_all(mod_dir)?;
    }

    let mod_content = if Path::new(mod_path).exists() {
        let existing = fs::read_to_string(mod_path)?;
        if existing.contains(&format!("mod {};", module_name)) {
            return Ok(());
        }
        format!("{}\nmod {};", existing.trim(), module_name)
    } else {
        format!("mod {};", module_name)
    };

    fs::write(mod_path, mod_content)?;

    // Also create/update the main tests directory mod.rs if it doesn't exist
    let main_test_mod = "tests/mod.rs";
    if !Path::new(main_test_mod).exists() {
        let main_content = r#"pub mod unit;
pub mod feature;"#;
        fs::write(main_test_mod, main_content)?;
    }

    Ok(())
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_is_upper = true;
        } else {
            result.push(c);
            prev_is_upper = false;
        }
    }

    result
}