use axum::{
    extract::{Json, State, Extension},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson, Html},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::database::DbPool;
use crate::app::services::mfa_service::MfaService;
use crate::app::services::mfa_email_service::MfaEmailService;
use crate::app::services::mfa_webauthn_service::MfaWebAuthnService;
use crate::app::services::mfa_biometric_service::MfaBiometricService;
use crate::app::models::mfa_email_code::{SendEmailCodeRequest, VerifyEmailCodeRequest};
use crate::app::models::mfa_webauthn::{
    WebAuthnRegistrationStartRequest, WebAuthnRegistrationFinishRequest,
    WebAuthnAuthenticationStartRequest, WebAuthnAuthenticationFinishRequest,
};
use crate::app::models::mfa_biometric::{BiometricRegistrationRequest, BiometricAuthenticationRequest};
use crate::config::Config;
use crate::app::http::middleware::auth_guard::AuthUser;
use crate::app::services::template_service::TemplateService;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct SuccessResponse {
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaSetupRequest {
    pub method_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaVerifyRequest {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaDisableRequest {
    pub current_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaRegenerateBackupCodesRequest {
    pub current_password: String,
}


/// Display the MFA setup page
pub async fn show_setup_page() -> impl IntoResponse {
    let context = json!({
        "page_title": "Multi-Factor Authentication Setup"
    });

    let template_service = TemplateService::global();
    match template_service.render("mfa/setup", &context).await {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Template rendering error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Template error").into_response()
        }
    }
}

/// API endpoint to setup TOTP MFA
pub async fn setup_mfa(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<MfaSetupRequest>,
) -> impl IntoResponse {

    if payload.method_type != "totp" {
        let error = ErrorResponse {
            error: "Only TOTP method is supported".to_string(),
        };
        return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
    }

    match MfaService::setup_totp(&pool, auth_user.user_id.clone(), "RustAxum").await {
        Ok(response) => (StatusCode::OK, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// API endpoint to verify TOTP code and enable MFA
pub async fn verify_mfa(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<MfaVerifyRequest>,
) -> impl IntoResponse {

    match MfaService::verify_totp(&pool, auth_user.user_id.clone(), &payload.code).await {
        Ok(is_valid) => {
            let response = json!({
                "verified": is_valid,
                "message": if is_valid { "MFA enabled successfully" } else { "Invalid verification code" }
            });
            (StatusCode::OK, ResponseJson(response)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// API endpoint to disable MFA
pub async fn disable_mfa(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<MfaDisableRequest>,
) -> impl IntoResponse {

    match MfaService::disable_mfa(&pool, auth_user.user_id.clone(), &payload.current_password).await {
        Ok(_) => {
            let response = SuccessResponse {
                message: "MFA disabled successfully".to_string(),
            };
            (StatusCode::OK, ResponseJson(response)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// API endpoint to regenerate backup codes
pub async fn regenerate_backup_codes(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<MfaRegenerateBackupCodesRequest>,
) -> impl IntoResponse {

    match MfaService::regenerate_backup_codes(&pool, auth_user.user_id.clone(), &payload.current_password).await {
        Ok(backup_codes) => {
            let response = json!({
                "backup_codes": backup_codes,
                "message": "Backup codes regenerated successfully"
            });
            (StatusCode::OK, ResponseJson(response)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// API endpoint to get user's MFA methods
pub async fn get_mfa_methods(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {

    match MfaService::get_mfa_methods(&pool, auth_user.user_id.clone()) {
        Ok(methods) => {
            let response = json!({
                "methods": methods
            });
            (StatusCode::OK, ResponseJson(response)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}