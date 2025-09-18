use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use crate::app::http::form_request::FormRequest;
use crate::app::validation::{Rule, required, email, min, confirmed, string};
use crate::impl_form_request_extractor;

/// Register user form request
#[derive(Deserialize, Serialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub password_confirmation: String,
}

#[async_trait]
impl FormRequest for RegisterRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("name", vec![required(), string(), min(2)]);
        rules.insert("email", vec![required(), email()]);
        rules.insert("password", vec![required(), string(), min(8)]);
        rules.insert("password_confirmation", vec![required(), confirmed()]);
        rules
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
#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub remember: bool,
}

#[async_trait]
impl FormRequest for LoginRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("email", vec![required(), email()]);
        rules.insert("password", vec![required(), string()]);
        rules
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
#[derive(Deserialize, Serialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[async_trait]
impl FormRequest for ForgotPasswordRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("email", vec![required(), email()]);
        rules
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
#[derive(Deserialize, Serialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub email: String,
    pub password: String,
    pub password_confirmation: String,
}

#[async_trait]
impl FormRequest for ResetPasswordRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("token", vec![required(), string()]);
        rules.insert("email", vec![required(), email()]);
        rules.insert("password", vec![required(), string(), min(8)]);
        rules.insert("password_confirmation", vec![required(), confirmed()]);
        rules
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
#[derive(Deserialize, Serialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub password: String,
    pub password_confirmation: String,
}

#[async_trait]
impl FormRequest for ChangePasswordRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("current_password", vec![required(), string()]);
        rules.insert("password", vec![required(), string(), min(8)]);
        rules.insert("password_confirmation", vec![required(), confirmed()]);
        rules
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