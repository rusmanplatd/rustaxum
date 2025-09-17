use rustaxum::app::models::user::{User, CreateUser, UpdateUser, UserResponse};
use chrono::Utc;
use ulid::Ulid;

#[tokio::test]
async fn test_user_creation() {
    let name = "John Doe".to_string();
    let email = "john@example.com".to_string();
    let password = "hashedpassword".to_string();

    let user = User::new(name.clone(), email.clone(), password.clone());

    assert_eq!(user.name, name);
    assert_eq!(user.email, email);
    assert_eq!(user.password, password);
    assert_eq!(user.failed_login_attempts, 0);
    assert!(user.email_verified_at.is_none());
    assert!(user.remember_token.is_none());
    assert!(user.refresh_token.is_none());
    assert!(user.refresh_token_expires_at.is_none());
    assert!(user.password_reset_token.is_none());
    assert!(user.password_reset_expires_at.is_none());
    assert!(user.last_login_at.is_none());
    assert!(user.locked_until.is_none());
    assert!(user.created_at <= Utc::now());
    assert!(user.updated_at <= Utc::now());
    assert_eq!(user.created_at, user.updated_at);
}

#[tokio::test]
async fn test_user_to_response() {
    let user = User::new(
        "Jane Doe".to_string(),
        "jane@example.com".to_string(),
        "password".to_string(),
    );

    let response = user.to_response();

    assert_eq!(response.id, user.id.to_string());
    assert_eq!(response.name, user.name);
    assert_eq!(response.email, user.email);
    assert_eq!(response.email_verified_at, user.email_verified_at);
    assert_eq!(response.last_login_at, user.last_login_at);
    assert_eq!(response.created_at, user.created_at);
    assert_eq!(response.updated_at, user.updated_at);
}

#[tokio::test]
async fn test_user_lock_status() {
    let mut user = User::new(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "password".to_string(),
    );

    // User should not be locked initially
    assert!(!user.is_locked());

    // Set lock until future time
    user.locked_until = Some(Utc::now() + chrono::Duration::hours(1));
    assert!(user.is_locked());

    // Set lock until past time
    user.locked_until = Some(Utc::now() - chrono::Duration::hours(1));
    assert!(!user.is_locked());

    // Remove lock
    user.locked_until = None;
    assert!(!user.is_locked());
}

#[tokio::test]
async fn test_password_reset_token_validation() {
    let mut user = User::new(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "password".to_string(),
    );

    let reset_token = "test_reset_token_123";

    // Should be invalid when no token is set
    assert!(!user.is_password_reset_valid(reset_token));

    // Set valid token
    user.password_reset_token = Some(reset_token.to_string());
    user.password_reset_expires_at = Some(Utc::now() + chrono::Duration::hours(1));

    // Should be valid with correct token
    assert!(user.is_password_reset_valid(reset_token));

    // Should be invalid with wrong token
    assert!(!user.is_password_reset_valid("wrong_token"));

    // Should be invalid when expired
    user.password_reset_expires_at = Some(Utc::now() - chrono::Duration::hours(1));
    assert!(!user.is_password_reset_valid(reset_token));

    // Should be invalid when token is cleared but expiration remains
    user.password_reset_token = None;
    user.password_reset_expires_at = Some(Utc::now() + chrono::Duration::hours(1));
    assert!(!user.is_password_reset_valid(reset_token));
}

#[tokio::test]
async fn test_refresh_token_validation() {
    let mut user = User::new(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "password".to_string(),
    );

    let refresh_token = "test_refresh_token_123";

    // Should be invalid when no token is set
    assert!(!user.is_refresh_token_valid(refresh_token));

    // Set valid token
    user.refresh_token = Some(refresh_token.to_string());
    user.refresh_token_expires_at = Some(Utc::now() + chrono::Duration::days(7));

    // Should be valid with correct token
    assert!(user.is_refresh_token_valid(refresh_token));

    // Should be invalid with wrong token
    assert!(!user.is_refresh_token_valid("wrong_token"));

    // Should be invalid when expired
    user.refresh_token_expires_at = Some(Utc::now() - chrono::Duration::hours(1));
    assert!(!user.is_refresh_token_valid(refresh_token));

    // Should be invalid when token is cleared but expiration remains
    user.refresh_token = None;
    user.refresh_token_expires_at = Some(Utc::now() + chrono::Duration::days(7));
    assert!(!user.is_refresh_token_valid(refresh_token));
}

#[tokio::test]
async fn test_user_id_is_ulid() {
    let user = User::new(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "password".to_string(),
    );

    // Should be able to convert to string and back
    let id_string = user.id.to_string();
    let parsed_id = Ulid::from_string(&id_string).unwrap();
    assert_eq!(user.id, parsed_id);

    // Should be 26 characters long
    assert_eq!(id_string.len(), 26);
}

#[tokio::test]
async fn test_create_user_request_validation() {
    let create_user = CreateUser {
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password: "TestPassword123!".to_string(),
    };

    assert_eq!(create_user.name, "Test User");
    assert_eq!(create_user.email, "test@example.com");
    assert_eq!(create_user.password, "TestPassword123!");
}

#[tokio::test]
async fn test_update_user_request_optional_fields() {
    let update_user = UpdateUser {
        name: Some("New Name".to_string()),
        email: Some("new@example.com".to_string()),
    };

    assert_eq!(update_user.name, Some("New Name".to_string()));
    assert_eq!(update_user.email, Some("new@example.com".to_string()));

    let partial_update = UpdateUser {
        name: Some("Only Name".to_string()),
        email: None,
    };

    assert_eq!(partial_update.name, Some("Only Name".to_string()));
    assert_eq!(partial_update.email, None);

    let empty_update = UpdateUser {
        name: None,
        email: None,
    };

    assert_eq!(empty_update.name, None);
    assert_eq!(empty_update.email, None);
}

#[tokio::test]
async fn test_user_response_excludes_sensitive_data() {
    let mut user = User::new(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "hashed_password_secret".to_string(),
    );

    user.refresh_token = Some("secret_refresh_token".to_string());
    user.password_reset_token = Some("secret_reset_token".to_string());
    user.remember_token = Some("secret_remember_token".to_string());

    let response = user.to_response();

    // Verify sensitive fields are not included in response
    let response_json = serde_json::to_string(&response).unwrap();
    assert!(!response_json.contains("password"));
    assert!(!response_json.contains("refresh_token"));
    assert!(!response_json.contains("password_reset_token"));
    assert!(!response_json.contains("remember_token"));
    assert!(!response_json.contains("failed_login_attempts"));
    assert!(!response_json.contains("locked_until"));

    // Verify public fields are included
    assert!(response_json.contains(&user.name));
    assert!(response_json.contains(&user.email));
    assert!(response_json.contains(&user.id.to_string()));
}

#[tokio::test]
async fn test_user_timestamps_are_utc() {
    let user = User::new(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "password".to_string(),
    );

    // Timestamps should be UTC
    assert_eq!(user.created_at.timezone(), Utc);
    assert_eq!(user.updated_at.timezone(), Utc);

    // Should be very recent (within last second)
    let now = Utc::now();
    let diff = now.signed_duration_since(user.created_at);
    assert!(diff.num_seconds() < 1);
}

#[tokio::test]
async fn test_user_edge_cases() {
    // Test with empty strings
    let user = User::new(
        String::new(),
        String::new(),
        String::new(),
    );

    assert_eq!(user.name, "");
    assert_eq!(user.email, "");
    assert_eq!(user.password, "");

    // Test with very long strings
    let long_name = "a".repeat(1000);
    let long_email = format!("{}@example.com", "a".repeat(1000));
    let long_password = "a".repeat(1000);

    let user = User::new(long_name.clone(), long_email.clone(), long_password.clone());

    assert_eq!(user.name, long_name);
    assert_eq!(user.email, long_email);
    assert_eq!(user.password, long_password);
}

#[tokio::test]
async fn test_failed_login_attempts_tracking() {
    let mut user = User::new(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "password".to_string(),
    );

    // Should start with 0 failed attempts
    assert_eq!(user.failed_login_attempts, 0);

    // Simulate failed login attempts
    user.failed_login_attempts = 1;
    assert_eq!(user.failed_login_attempts, 1);

    user.failed_login_attempts = 5;
    assert_eq!(user.failed_login_attempts, 5);

    // Reset to 0
    user.failed_login_attempts = 0;
    assert_eq!(user.failed_login_attempts, 0);
}