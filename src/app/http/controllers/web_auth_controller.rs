use axum::{
    extract::{State, Extension, Form, Path, Query},
    response::{IntoResponse, Redirect},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::database::DbPool;
use crate::app::services::session::SessionStore;
use crate::app::services::auth_service::AuthService;
use crate::app::services::user_service::UserService;
use crate::app::http::responses::template_response::TemplateResponse;
use crate::app::models::user::{LoginRequest, CreateUser, ForgotPasswordRequest, ResetPasswordRequest, ChangePasswordRequest};

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
    pub remember_me: Option<String>,
    pub redirect: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub password_confirmation: String,
    pub terms: Option<String>,
    pub newsletter: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordForm {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordForm {
    pub token: String,
    pub password: String,
    pub password_confirmation: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordForm {
    pub current_password: String,
    pub password: String,
    pub password_confirmation: String,
}

#[derive(Debug, Deserialize)]
pub struct RedirectQuery {
    pub redirect: Option<String>,
}

pub struct WebAuthController;

impl WebAuthController {
    // Authentication Pages

    pub async fn show_login(
        Extension(session): Extension<SessionStore>,
        Query(query): Query<RedirectQuery>,
    ) -> impl IntoResponse {
        // Check if already authenticated
        if session.get_bool("authenticated").await.unwrap_or(false) {
            return Redirect::to("/dashboard").into_response();
        }

        let data = json!({
            "title": "Sign In - RustAxum",
            "page_title": "Welcome Back",
            "subtitle": "Sign in to your account to continue",
            "header_icon": "fas fa-sign-in-alt",
            "redirect": query.redirect.unwrap_or_default(),
            "registration_enabled": true,
            "social_login_enabled": false,
            "errors": session.get("errors").await.unwrap_or(Value::Null),
            "old_input": session.get("old_input").await.unwrap_or(Value::Null),
            "success_message": session.get("success").await,
            "error_message": session.get("error").await,
            "csrf_token": session.token().await
        });

        // Clear flash data after displaying
        session.forget("errors").await;
        session.forget("old_input").await;
        session.forget("success").await;
        session.forget("error").await;

        TemplateResponse::new("auth/login", &data).with_layout("layouts/auth").into_response()
    }

    pub async fn show_register(
        Extension(session): Extension<SessionStore>,
    ) -> impl IntoResponse {
        // Check if already authenticated
        if session.get_bool("authenticated").await.unwrap_or(false) {
            return Redirect::to("/dashboard").into_response();
        }

        let data = json!({
            "title": "Create Account - RustAxum",
            "page_title": "Join Us Today",
            "subtitle": "Create your account to get started",
            "header_icon": "fas fa-user-plus",
            "errors": session.get("errors").await.unwrap_or(Value::Null),
            "old_input": session.get("old_input").await.unwrap_or(Value::Null),
            "success_message": session.get("success").await,
            "error_message": session.get("error").await,
            "csrf_token": session.token().await
        });

        // Clear flash data after displaying
        session.forget("errors").await;
        session.forget("old_input").await;
        session.forget("success").await;
        session.forget("error").await;

        TemplateResponse::new("auth/register", &data).with_layout("layouts/auth").into_response()
    }

    pub async fn show_forgot_password(
        Extension(session): Extension<SessionStore>,
    ) -> impl IntoResponse {
        let data = json!({
            "title": "Reset Password - RustAxum",
            "page_title": "Reset Your Password",
            "subtitle": "We'll send you a secure reset link",
            "header_icon": "fas fa-key",
            "errors": session.get("errors").await.unwrap_or(Value::Null),
            "old_input": session.get("old_input").await.unwrap_or(Value::Null),
            "success_message": session.get("success").await,
            "error_message": session.get("error").await,
            "csrf_token": session.token().await
        });

        // Clear flash data after displaying
        session.forget("errors").await;
        session.forget("old_input").await;
        session.forget("success").await;
        session.forget("error").await;

        TemplateResponse::new("auth/forgot-password", &data).with_layout("layouts/auth").into_response()
    }

    pub async fn show_reset_password(
        Path(token): Path<String>,
        Extension(session): Extension<SessionStore>,
        State(pool): State<DbPool>,
    ) -> impl IntoResponse {
        // Verify token exists and is valid
        if let Ok(Some(_user)) = UserService::find_by_reset_token(&pool, &token) {
            let data = json!({
                "title": "Reset Password - RustAxum",
                "page_title": "Create New Password",
                "subtitle": "Enter your new password below",
                "header_icon": "fas fa-lock",
                "token": token,
                "errors": session.get("errors").await.unwrap_or(Value::Null),
                "success_message": session.get("success").await,
                "error_message": session.get("error").await,
                "csrf_token": session.token().await
            });

            // Clear flash data
            session.forget("errors").await;
            session.forget("success").await;
            session.forget("error").await;

            TemplateResponse::new("auth/reset-password", &data).with_layout("layouts/auth").into_response()
        } else {
            session.flash("error", Value::String("Invalid or expired reset token.".to_string())).await;
            Redirect::to("/auth/forgot-password").into_response()
        }
    }

    pub async fn show_change_password(
        Extension(session): Extension<SessionStore>,
    ) -> impl IntoResponse {
        // Check if authenticated
        if !session.get_bool("authenticated").await.unwrap_or(false) {
            return Redirect::to("/auth/login").into_response();
        }

        let data = json!({
            "title": "Change Password - RustAxum",
            "page_title": "Change Your Password",
            "subtitle": "Update your account password",
            "header_icon": "fas fa-lock",
            "errors": session.get("errors").await.unwrap_or(Value::Null),
            "success_message": session.get("success").await,
            "error_message": session.get("error").await,
            "csrf_token": session.token().await
        });

        // Clear flash data
        session.forget("errors").await;
        session.forget("success").await;
        session.forget("error").await;

        TemplateResponse::new("auth/change-password", &data).with_layout("layouts/dashboard").into_response()
    }

    // Authentication Actions

    pub async fn login(
        State(pool): State<DbPool>,
        Extension(session): Extension<SessionStore>,
        Form(form): Form<LoginForm>,
    ) -> impl IntoResponse {
        // Validate form
        let mut errors = json!({});

        if form.email.is_empty() {
            errors["email"] = Value::Array(vec![Value::String("Email is required.".to_string())]);
        }
        if form.password.is_empty() {
            errors["password"] = Value::Array(vec![Value::String("Password is required.".to_string())]);
        }

        if !errors.as_object().unwrap().is_empty() {
            session.flash("errors", errors).await;
            session.flash("old_input", json!({"email": form.email})).await;
            return Redirect::to("/auth/login").into_response();
        }

        let login_request = LoginRequest {
            email: form.email.clone(),
            password: form.password,
        };

        match AuthService::login(&pool, login_request).await {
            Ok(response) => {
                // Store user authentication in session
                session.put("user_id", Value::String(response.user.id.to_string())).await;
                session.put("authenticated", Value::Bool(true)).await;
                session.put("user_name", Value::String(response.user.name.clone())).await;
                session.put("user_email", Value::String(response.user.email.clone())).await;

                // Handle remember me
                if form.remember_me.is_some() {
                    // Extend session lifetime for remember me
                    // This would be implemented based on your session configuration
                }

                // Regenerate session ID for security
                session.regenerate().await.ok();

                // Clear any old errors
                session.forget("errors").await;
                session.forget("error").await;

                // Redirect to intended destination or dashboard
                let redirect_url = form.redirect.unwrap_or_else(|| "/dashboard".to_string());
                Redirect::to(&redirect_url).into_response()
            }
            Err(e) => {
                session.flash("error", Value::String(e.to_string())).await;
                session.flash("old_input", json!({"email": form.email})).await;
                Redirect::to("/auth/login").into_response()
            }
        }
    }

    pub async fn register(
        State(pool): State<DbPool>,
        Extension(session): Extension<SessionStore>,
        Form(form): Form<RegisterForm>,
    ) -> impl IntoResponse {
        // Validate form
        let mut errors = json!({});

        if form.first_name.is_empty() {
            errors["first_name"] = Value::Array(vec![Value::String("First name is required.".to_string())]);
        }
        if form.last_name.is_empty() {
            errors["last_name"] = Value::Array(vec![Value::String("Last name is required.".to_string())]);
        }
        if form.email.is_empty() {
            errors["email"] = Value::Array(vec![Value::String("Email is required.".to_string())]);
        }
        if form.password.is_empty() {
            errors["password"] = Value::Array(vec![Value::String("Password is required.".to_string())]);
        } else if form.password.len() < 6 {
            errors["password"] = Value::Array(vec![Value::String("Password must be at least 6 characters.".to_string())]);
        }
        if form.password != form.password_confirmation {
            errors["password_confirmation"] = Value::Array(vec![Value::String("Passwords do not match.".to_string())]);
        }
        if form.terms.is_none() {
            errors["terms"] = Value::Array(vec![Value::String("You must accept the terms of service.".to_string())]);
        }

        if !errors.as_object().unwrap().is_empty() {
            session.flash("errors", errors).await;
            session.flash("old_input", json!({
                "first_name": form.first_name,
                "last_name": form.last_name,
                "email": form.email,
                "newsletter": form.newsletter
            })).await;
            return Redirect::to("/auth/register").into_response();
        }

        let create_user = CreateUser {
            name: format!("{} {}", form.first_name, form.last_name).trim().to_string(),
            email: form.email.clone(),
            password: form.password,
        };

        match AuthService::register(&pool, create_user).await {
            Ok(response) => {
                // Store user authentication in session
                session.put("user_id", Value::String(response.user.id.to_string())).await;
                session.put("authenticated", Value::Bool(true)).await;
                session.put("user_name", Value::String(response.user.name.clone())).await;
                session.put("user_email", Value::String(response.user.email.clone())).await;

                // Regenerate session ID for security
                session.regenerate().await.ok();

                // Clear any old errors
                session.forget("errors").await;
                session.forget("error").await;

                // Flash success message
                session.flash("success", Value::String("Registration successful! Welcome to RustAxum.".to_string())).await;

                Redirect::to("/dashboard").into_response()
            }
            Err(e) => {
                session.flash("error", Value::String(e.to_string())).await;
                session.flash("old_input", json!({
                    "first_name": form.first_name,
                    "last_name": form.last_name,
                    "email": form.email,
                    "newsletter": form.newsletter
                })).await;
                Redirect::to("/auth/register").into_response()
            }
        }
    }

    pub async fn forgot_password(
        State(pool): State<DbPool>,
        Extension(session): Extension<SessionStore>,
        Form(form): Form<ForgotPasswordForm>,
    ) -> impl IntoResponse {
        // Validate form
        if form.email.is_empty() {
            let errors = json!({"email": ["Email is required."]});
            session.flash("errors", errors).await;
            session.flash("old_input", json!({"email": form.email})).await;
            return Redirect::to("/auth/forgot-password").into_response();
        }

        let forgot_request = ForgotPasswordRequest {
            email: form.email.clone(),
        };

        match AuthService::forgot_password(&pool, forgot_request).await {
            Ok(_) => {
                session.flash("success", Value::String("If an account with that email exists, you'll receive a password reset link within the next few minutes.".to_string())).await;
                Redirect::to("/auth/forgot-password").into_response()
            }
            Err(e) => {
                session.flash("error", Value::String(e.to_string())).await;
                session.flash("old_input", json!({"email": form.email})).await;
                Redirect::to("/auth/forgot-password").into_response()
            }
        }
    }

    pub async fn reset_password(
        State(pool): State<DbPool>,
        Extension(session): Extension<SessionStore>,
        Form(form): Form<ResetPasswordForm>,
    ) -> impl IntoResponse {
        // Validate form
        let mut errors = json!({});

        if form.password.is_empty() {
            errors["password"] = Value::Array(vec![Value::String("Password is required.".to_string())]);
        } else if form.password.len() < 6 {
            errors["password"] = Value::Array(vec![Value::String("Password must be at least 6 characters.".to_string())]);
        }
        if form.password != form.password_confirmation {
            errors["password_confirmation"] = Value::Array(vec![Value::String("Passwords do not match.".to_string())]);
        }

        if !errors.as_object().unwrap().is_empty() {
            session.flash("errors", errors).await;
            return Redirect::to(&format!("/auth/reset-password/{}", form.token)).into_response();
        }

        let reset_request = ResetPasswordRequest {
            token: form.token.clone(),
            password: form.password,
            password_confirmation: form.password_confirmation,
        };

        match AuthService::reset_password(&pool, reset_request) {
            Ok(_) => {
                session.flash("success", Value::String("Your password has been reset successfully. You can now log in with your new password.".to_string())).await;
                Redirect::to("/auth/login").into_response()
            }
            Err(e) => {
                session.flash("error", Value::String(e.to_string())).await;
                Redirect::to(&format!("/auth/reset-password/{}", form.token)).into_response()
            }
        }
    }

    pub async fn change_password(
        State(pool): State<DbPool>,
        Extension(session): Extension<SessionStore>,
        Form(form): Form<ChangePasswordForm>,
    ) -> impl IntoResponse {
        // Check if authenticated
        if !session.get_bool("authenticated").await.unwrap_or(false) {
            return Redirect::to("/auth/login").into_response();
        }

        let user_id = match session.get_string("user_id").await {
            Some(id) => id,
            None => {
                session.flash("error", Value::String("Authentication error. Please log in again.".to_string())).await;
                return Redirect::to("/auth/login").into_response();
            }
        };

        // Validate form
        let mut errors = json!({});

        if form.current_password.is_empty() {
            errors["current_password"] = Value::Array(vec![Value::String("Current password is required.".to_string())]);
        }
        if form.password.is_empty() {
            errors["password"] = Value::Array(vec![Value::String("New password is required.".to_string())]);
        } else if form.password.len() < 6 {
            errors["password"] = Value::Array(vec![Value::String("Password must be at least 6 characters.".to_string())]);
        }
        if form.password != form.password_confirmation {
            errors["password_confirmation"] = Value::Array(vec![Value::String("Passwords do not match.".to_string())]);
        }

        if !errors.as_object().unwrap().is_empty() {
            session.flash("errors", errors).await;
            return Redirect::to("/auth/change-password").into_response();
        }

        let change_request = ChangePasswordRequest {
            current_password: form.current_password,
            new_password: form.password,
            password_confirmation: form.password_confirmation,
        };

        match AuthService::change_password(&pool, user_id, change_request) {
            Ok(_) => {
                session.flash("success", Value::String("Your password has been changed successfully.".to_string())).await;
                Redirect::to("/auth/change-password").into_response()
            }
            Err(e) => {
                session.flash("error", Value::String(e.to_string())).await;
                Redirect::to("/auth/change-password").into_response()
            }
        }
    }

    pub async fn logout(
        Extension(session): Extension<SessionStore>,
    ) -> impl IntoResponse {
        // Clear all session data
        session.flush().await;

        // Regenerate session ID
        session.regenerate().await.ok();

        // Flash success message
        session.flash("success", Value::String("You have been logged out successfully.".to_string())).await;

        Redirect::to("/auth/login")
    }

    // Dashboard and Profile

    pub async fn dashboard(
        State(pool): State<DbPool>,
        Extension(session): Extension<SessionStore>,
    ) -> impl IntoResponse {
        // Check if authenticated
        if !session.get_bool("authenticated").await.unwrap_or(false) {
            return Redirect::to("/auth/login").into_response();
        }

        let user_id = match session.get_string("user_id").await {
            Some(id) => id,
            None => return Redirect::to("/auth/login").into_response(),
        };

        match UserService::find_by_id(&pool, user_id) {
            Ok(Some(user)) => {
                let data = json!({
                    "title": "Dashboard - RustAxum",
                    "page_title": "Dashboard",
                    "user": user.to_response(),
                    "success_message": session.get("success").await,
                    "error_message": session.get("error").await,
                });

                // Clear flash data
                session.forget("success").await;
                session.forget("error").await;

                TemplateResponse::new("dashboard/index", &data).with_layout("layouts/dashboard").into_response()
            }
            Ok(None) => {
                session.flush().await;
                session.flash("error", Value::String("User account not found. Please log in again.".to_string())).await;
                Redirect::to("/auth/login").into_response()
            }
            Err(_) => {
                session.flash("error", Value::String("An error occurred. Please try again.".to_string())).await;
                Redirect::to("/auth/login").into_response()
            }
        }
    }
}