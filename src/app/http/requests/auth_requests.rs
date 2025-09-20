use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use utoipa::ToSchema;

use crate::app::http::form_request::FormRequest;
use crate::app::validation::ValidationRules;
use crate::validation_rules;
use crate::impl_form_request_extractor;

/// Register user form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct RegisterRequest {
    /// User's full name
    #[schema(example = "John Doe")]
    pub name: String,
    /// User's email address
    #[schema(example = "john@example.com")]
    pub email: String,
    /// User's password (minimum 8 characters)
    #[schema(example = "SecurePass123!")]
    pub password: String,
    /// Password confirmation (must match password)
    #[schema(example = "SecurePass123!")]
    pub password_confirmation: String,
}

#[async_trait]
impl FormRequest for RegisterRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "name" => ["required", "string", "min:2"],
            "email" => ["required", "email"],
            "password" => ["required", "string", "min:8"],
            "password_confirmation" => ["required", "confirmed"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("name.required", "Name is required");
        messages.insert("name.min", "Name must be at least 2 characters");
        messages.insert("email.required", "Email is required");
        messages.insert("email.email", "Email must be a valid email address");
        messages.insert("password.required", "Password is required");
        messages.insert("password.min", "Password must be at least 8 characters");
        messages.insert("password_confirmation.required", "Password confirmation is required");
        messages.insert("password_confirmation.confirmed", "Password confirmation does not match");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("password_confirmation", "password confirmation");
        attributes
    }
}

impl_form_request_extractor!(RegisterRequest);

/// Login form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    /// User's email address
    #[schema(example = "john@example.com")]
    pub email: String,
    /// User's password
    #[schema(example = "SecurePass123!")]
    pub password: String,
    /// Remember me option for extended session
    #[serde(default)]
    #[schema(example = true)]
    pub remember: bool,
}

#[async_trait]
impl FormRequest for LoginRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "email" => ["required", "email"],
            "password" => ["required", "string"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("email.required", "Email is required");
        messages.insert("email.email", "Email must be a valid email address");
        messages.insert("password.required", "Password is required");
        messages
    }
}

impl_form_request_extractor!(LoginRequest);

/// Forgot password form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct ForgotPasswordRequest {
    /// User's email address to send reset link
    #[schema(example = "john@example.com")]
    pub email: String,
}

#[async_trait]
impl FormRequest for ForgotPasswordRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "email" => ["required", "email"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("email.required", "Email is required");
        messages.insert("email.email", "Email must be a valid email address");
        messages
    }
}

impl_form_request_extractor!(ForgotPasswordRequest);

/// Reset password form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct ResetPasswordRequest {
    /// Password reset token
    #[schema(example = "abc123def456")]
    pub token: String,
    /// User's email address
    #[schema(example = "john@example.com")]
    pub email: String,
    /// New password (minimum 8 characters)
    #[schema(example = "NewSecurePass123!")]
    pub password: String,
    /// Password confirmation (must match password)
    #[schema(example = "NewSecurePass123!")]
    pub password_confirmation: String,
}

#[async_trait]
impl FormRequest for ResetPasswordRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "token" => ["required", "string"],
            "email" => ["required", "email"],
            "password" => ["required", "string", "min:8"],
            "password_confirmation" => ["required", "confirmed"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("token.required", "Reset token is required");
        messages.insert("email.required", "Email is required");
        messages.insert("email.email", "Email must be a valid email address");
        messages.insert("password.required", "Password is required");
        messages.insert("password.min", "Password must be at least 8 characters");
        messages.insert("password_confirmation.required", "Password confirmation is required");
        messages.insert("password_confirmation.confirmed", "Password confirmation does not match");
        messages
    }
}

impl_form_request_extractor!(ResetPasswordRequest);

/// Change password form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct ChangePasswordRequest {
    /// Current password for verification
    #[schema(example = "CurrentPass123!")]
    pub current_password: String,
    /// New password (minimum 8 characters)
    #[schema(example = "NewSecurePass123!")]
    pub password: String,
    /// Password confirmation (must match password)
    #[schema(example = "NewSecurePass123!")]
    pub password_confirmation: String,
}

#[async_trait]
impl FormRequest for ChangePasswordRequest {
    fn rules() -> ValidationRules {
        validation_rules! {
            "current_password" => ["required", "string"],
            "password" => ["required", "string", "min:8"],
            "password_confirmation" => ["required", "confirmed"]
        }
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("current_password.required", "Current password is required");
        messages.insert("password.required", "New password is required");
        messages.insert("password.min", "Password must be at least 8 characters");
        messages.insert("password_confirmation.required", "Password confirmation is required");
        messages.insert("password_confirmation.confirmed", "Password confirmation does not match");
        messages
    }
}

impl_form_request_extractor!(ChangePasswordRequest);