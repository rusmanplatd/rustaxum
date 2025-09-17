use axum::{
    http::{header::AUTHORIZATION, StatusCode},
    Router,
};
use axum_test::TestServer;
use serde_json::Value;
use serial_test::serial;

use rustaxum::app::models::user::{CreateUser, LoginRequest, ForgotPasswordRequest, ResetPasswordRequest, ChangePasswordRequest, RefreshTokenRequest};
use rustaxum::routes::api;

fn setup_test_server() -> TestServer {
    let app = Router::new()
        .merge(api::routes());

    TestServer::new(app).unwrap()
}

#[tokio::test]
#[serial]
async fn test_auth_endpoints_respond() {
    let server = setup_test_server();

    // Test registration endpoint exists
    let user_data = CreateUser {
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password: "TestPassword123!".to_string(),
    };

    let response = server
        .post("/api/auth/register")
        .json(&user_data)
        .await;

    assert_ne!(response.status_code(), StatusCode::NOT_FOUND);

    // Test login endpoint exists
    let login_data = LoginRequest {
        email: "test@example.com".to_string(),
        password: "TestPassword123!".to_string(),
    };

    let response = server
        .post("/api/auth/login")
        .json(&login_data)
        .await;

    assert_ne!(response.status_code(), StatusCode::NOT_FOUND);

    // Test forgot password endpoint exists
    let forgot_data = ForgotPasswordRequest {
        email: "test@example.com".to_string(),
    };

    let response = server
        .post("/api/auth/forgot-password")
        .json(&forgot_data)
        .await;

    assert_ne!(response.status_code(), StatusCode::NOT_FOUND);

    // Test refresh token endpoint exists
    let refresh_data = RefreshTokenRequest {
        refresh_token: "test_token".to_string(),
    };

    let response = server
        .post("/api/auth/refresh-token")
        .json(&refresh_data)
        .await;

    assert_ne!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[serial]
async fn test_protected_endpoints_require_auth() {
    let server = setup_test_server();

    // Test change password requires auth
    let change_data = ChangePasswordRequest {
        current_password: "old".to_string(),
        new_password: "NewPassword123!".to_string(),
        password_confirmation: "NewPassword123!".to_string(),
    };

    let response = server
        .put("/api/auth/change-password")
        .json(&change_data)
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

    // Test logout requires auth
    let response = server
        .post("/api/auth/logout")
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

    // Test revoke token requires auth
    let response = server
        .delete("/api/auth/revoke-token")
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

    // Test revoke all tokens requires auth
    let response = server
        .delete("/api/auth/revoke-all-tokens")
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn test_invalid_json_returns_bad_request() {
    let server = setup_test_server();

    let response = server
        .post("/api/auth/register")
        .text("invalid json")
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn test_weak_password_validation() {
    let server = setup_test_server();

    let user_data = CreateUser {
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password: "weak".to_string(),
    };

    let response = server
        .post("/api/auth/register")
        .json(&user_data)
        .await;

    // Should return bad request due to weak password
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    let body: Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("Password must be at least 8 characters"));
}

#[tokio::test]
#[serial]
async fn test_missing_password_complexity() {
    let server = setup_test_server();

    // Test password without uppercase
    let user_data = CreateUser {
        name: "Test User".to_string(),
        email: "test1@example.com".to_string(),
        password: "password123!".to_string(),
    };

    let response = server
        .post("/api/auth/register")
        .json(&user_data)
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    let body: Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("uppercase letter"));

    // Test password without numbers
    let user_data = CreateUser {
        name: "Test User".to_string(),
        email: "test2@example.com".to_string(),
        password: "Password!".to_string(),
    };

    let response = server
        .post("/api/auth/register")
        .json(&user_data)
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    let body: Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("number"));

    // Test password without special characters
    let user_data = CreateUser {
        name: "Test User".to_string(),
        email: "test3@example.com".to_string(),
        password: "Password123".to_string(),
    };

    let response = server
        .post("/api/auth/register")
        .json(&user_data)
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    let body: Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("special character"));
}