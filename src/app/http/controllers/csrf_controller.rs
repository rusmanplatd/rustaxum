use axum::{
    extract::Request,
    response::{Html, Json},
    Extension,
    http::StatusCode,
};
use serde_json::{json, Value};

use crate::app::services::csrf_service::CSRFService;
use crate::app::services::session::SessionStore;
use crate::app::helpers::csrf_helpers::CSRFHelpers;
use crate::app::services::template_service::TemplateService;

pub struct CSRFController;

impl CSRFController {
    /// Get CSRF token for the current session
    pub async fn token(
        Extension(session_store): Extension<SessionStore>,
    ) -> Result<Json<Value>, StatusCode> {
        let csrf_service = CSRFService::new();

        match csrf_service.token(&session_store).await {
            Ok(token) => Ok(Json(json!({
                "token": token,
                "token_name": csrf_service.token_name(),
                "header_name": csrf_service.header_name()
            }))),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    /// Show a test form with CSRF protection
    pub async fn form(
        Extension(session_store): Extension<SessionStore>,
    ) -> Result<Html<String>, StatusCode> {
        match CSRFHelpers::csrf_field(&session_store).await {
            Ok(csrf_field) => {
                let template_service = TemplateService::global();
                let context = json!({
                    "csrf_field": csrf_field
                });

                match template_service.render("csrf/form", &context).await {
                    Ok(html) => Ok(Html(html)),
                    Err(e) => {
                        tracing::error!("Template rendering error: {}", e);
                        Ok(Html("<h1>Template Error</h1><p>Unable to render CSRF form</p>".to_string()))
                    }
                }
            },
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    /// Handle test form submission
    pub async fn test_form(
        Extension(_session_store): Extension<SessionStore>,
        _request: Request,
    ) -> Result<Html<String>, StatusCode> {
        // If we reach here, CSRF validation passed
        let template_service = TemplateService::global();
        let context = json!({});

        match template_service.render("csrf/success", &context).await {
            Ok(html) => Ok(Html(html)),
            Err(e) => {
                tracing::error!("Template rendering error: {}", e);
                Ok(Html("<h1>Template Error</h1><p>Unable to render CSRF success page</p>".to_string()))
            }
        }
    }

    /// Test API endpoint with CSRF protection
    pub async fn test_api(
        Extension(_session_store): Extension<SessionStore>,
    ) -> Result<Json<Value>, StatusCode> {
        // If we reach here, CSRF validation passed
        Ok(Json(json!({
            "success": true,
            "message": "API request successful! CSRF protection is working.",
            "timestamp": chrono::Utc::now().to_rfc3339()
        })))
    }

    /// Regenerate CSRF token (useful after login/logout)
    pub async fn regenerate(
        Extension(session_store): Extension<SessionStore>,
    ) -> Result<Json<Value>, StatusCode> {
        let csrf_service = CSRFService::new();

        match csrf_service.regenerate_token(&session_store).await {
            Ok(new_token) => Ok(Json(json!({
                "success": true,
                "token": new_token,
                "message": "CSRF token regenerated successfully"
            }))),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}