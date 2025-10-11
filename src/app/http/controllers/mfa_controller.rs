use axum::{
    extract::{Json, State, Extension},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson, Html},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::database::DbPool;
use crate::app::services::mfa_service::MfaService;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaSetupSmsRequest {
    pub phone_number: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaDisableMethodRequest {
    pub method_type: String,
}


/// Display the MFA setup/settings page
pub async fn show_setup_page(
    Extension(session): Extension<crate::app::services::session::SessionStore>,
) -> impl IntoResponse {
    use crate::app::http::responses::template_response::TemplateResponse;
    use axum::response::Redirect;

    // Check if authenticated
    if !session.get_bool("authenticated").await.unwrap_or(false) {
        return Redirect::to("/auth/login").into_response();
    }

    let user_name = session.get_string("user_name").await.unwrap_or("User".to_string());

    let context = json!({
        "title": "MFA Settings",
        "page_title": "Multi-Factor Authentication",
        "user": {
            "name": user_name
        },
        "app_name": "RustAxum",
        "csrf_token": session.token().await
    });

    TemplateResponse::new("mfa/settings", &context)
        .with_layout("layouts/dashboard")
        .into_response()
}

/// Display the MFA verification page (for login flow)
pub async fn show_verify_page(
    Extension(session): Extension<crate::app::services::session::SessionStore>,
) -> impl IntoResponse {
    use axum::response::Redirect;

    // Check if MFA is required for this session
    let mfa_required = session.get_bool("mfa_required").await.unwrap_or(false);
    if !mfa_required {
        return Redirect::to("/auth/login").into_response();
    }

    let user_id = session.get_string("mfa_user_id").await.unwrap_or_default();

    let context = json!({
        "title": "MFA Verification",
        "user_id": user_id,
        "app_name": "RustAxum",
        "csrf_token": session.token().await
    });

    let template_service = TemplateService::global();
    match template_service.render("mfa/verify", &context).await {
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

/// API endpoint to setup Email OTP MFA
pub async fn setup_email_mfa(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    use crate::app::services::mfa_email_service::MfaEmailService;

    // Send test code
    match MfaEmailService::send_code(&pool, auth_user.user_id.clone(), None, None).await {
        Ok(_) => {
            let response = json!({
                "message": "Email OTP enabled. A test code has been sent to your email."
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

/// API endpoint to setup SMS OTP MFA
pub async fn setup_sms_mfa(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<MfaSetupSmsRequest>,
) -> impl IntoResponse {
    use crate::app::services::mfa_sms_service::MfaSmsService;

    // Send test code
    match MfaSmsService::send_code(&pool, auth_user.user_id.clone(), payload.phone_number, None, None).await {
        Ok(_) => {
            let response = json!({
                "message": "SMS OTP enabled. A test code has been sent to your phone."
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

/// API endpoint to disable a specific MFA method
pub async fn disable_method(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<MfaDisableMethodRequest>,
) -> impl IntoResponse {
    use crate::schema::mfa_methods;
    use crate::app::models::DieselUlid;
    use diesel::prelude::*;

    let user_id_ulid = match DieselUlid::from_string(&auth_user.user_id) {
        Ok(ulid) => ulid,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Invalid user ID: {}", e),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Database error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    // Disable the method
    match diesel::update(
        mfa_methods::table
            .filter(mfa_methods::user_id.eq(&user_id_ulid))
            .filter(mfa_methods::method_type.eq(&payload.method_type))
    )
    .set(mfa_methods::is_enabled.eq(false))
    .execute(&mut conn)
    {
        Ok(_) => {
            let response = json!({
                "message": format!("{} MFA disabled successfully", payload.method_type)
            });
            (StatusCode::OK, ResponseJson(response)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to disable method: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}