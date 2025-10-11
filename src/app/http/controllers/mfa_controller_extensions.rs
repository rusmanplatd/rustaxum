use axum::{
    extract::{Json, State, Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::{Serialize, Deserialize};
use serde_json::json;
use crate::app::services::mfa_service::MfaService;
use crate::database::DbPool;
use crate::app::services::mfa_email_service::MfaEmailService;
use crate::app::services::mfa_webauthn_service::MfaWebAuthnService;
use crate::app::services::mfa_biometric_service::MfaBiometricService;
use crate::app::models::mfa_email_code::VerifyEmailCodeRequest;
use crate::app::models::mfa_webauthn::WebAuthnRegistrationStartRequest;
use crate::app::models::mfa_biometric::{BiometricRegistrationRequest, BiometricRegistrationStartRequest, BiometricAuthenticationRequest};
use crate::app::http::middleware::auth_guard::AuthUser;
use crate::config::Config;
use webauthn_rs::prelude::*;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct SuccessResponse {
    message: String,
}

// ===== EMAIL OTP ENDPOINTS =====

/// Send email OTP code
pub async fn send_email_code(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    match MfaEmailService::send_code(
        &pool,
        auth_user.user_id.clone(),
        None, // IP address would come from request
        None, // User agent would come from request
    )
    .await
    {
        Ok(_) => {
            let response = SuccessResponse {
                message: "Email code sent successfully".to_string(),
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

/// Verify email OTP code
pub async fn verify_email_code(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<VerifyEmailCodeRequest>,
) -> impl IntoResponse {
    match MfaEmailService::verify_code(&pool, auth_user.user_id.clone(), &payload.code).await {
        Ok(is_valid) => {
            let response = json!({
                "verified": is_valid,
                "message": if is_valid { "Email code verified successfully" } else { "Invalid email code" }
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

// ===== WEBAUTHN ENDPOINTS =====

/// Start WebAuthn registration
pub async fn webauthn_register_start(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<WebAuthnRegistrationStartRequest>,
) -> impl IntoResponse {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Config error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    // Extract domain from URL and ensure it's the proper format
    let url = match url::Url::parse(&config.app.url) {
        Ok(u) => u,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Invalid app URL: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    // For WebAuthn, the RP ID must be a registrable domain suffix of the origin
    let rp_id = match url.host_str() {
        Some(host) => host.trim_end_matches('.').trim_start_matches("www.").to_string(),
        None => {
            let error = ErrorResponse {
                error: "No host in app URL".to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    // Origin must be the full URL including protocol, but without path, query, or fragment
    let origin = format!(
        "{}://{}{}",
        url.scheme(),
        url.host_str().unwrap(),
        if let Some(port) = url.port() {
            format!(":{}", port)
        } else {
            "".to_string()
        }
    );

    let service = match MfaWebAuthnService::new(
        &origin,  // Full origin URL with protocol and port
        &rp_id,   // Domain only (no port) for RP ID
        &config.app.name,
    ) {
        Ok(s) => s,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("WebAuthn service error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    match service
        .start_registration(&pool, auth_user.user_id.clone(), payload.device_name)
        .await
    {
        Ok(challenge) => (StatusCode::OK, ResponseJson(challenge)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Finish WebAuthn registration
pub async fn webauthn_register_finish(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(credential): Json<RegisterPublicKeyCredential>,
) -> impl IntoResponse {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Config error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    // Extract domain from URL and ensure it's the proper format
    let url = match url::Url::parse(&config.app.url) {
        Ok(u) => u,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Invalid app URL: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    // For WebAuthn, the RP ID must be a registrable domain suffix of the origin
    let rp_id = match url.host_str() {
        Some(host) => host.trim_end_matches('.').trim_start_matches("www.").to_string(),
        None => {
            let error = ErrorResponse {
                error: "No host in app URL".to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    // Origin must be the full URL including protocol, but without path, query, or fragment
    let origin = format!(
        "{}://{}{}",
        url.scheme(),
        url.host_str().unwrap(),
        if let Some(port) = url.port() {
            format!(":{}", port)
        } else {
            "".to_string()
        }
    );

    let service = match MfaWebAuthnService::new(
        &origin,  // Full origin URL with protocol and port
        &rp_id,   // Domain only (no port) for RP ID
        &config.app.name,
    ) {
        Ok(s) => s,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("WebAuthn service error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    match service
        .finish_registration(&pool, auth_user.user_id.clone(), credential, None)
        .await
    {
        Ok(_) => {
            let response = SuccessResponse {
                message: "WebAuthn credential registered successfully".to_string(),
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

/// Start WebAuthn authentication
pub async fn webauthn_auth_start(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Config error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    // Extract domain from URL and ensure it's the proper format
    let url = match url::Url::parse(&config.app.url) {
        Ok(u) => u,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Invalid app URL: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    // For WebAuthn, the RP ID must be a registrable domain suffix of the origin
    let rp_id = match url.host_str() {
        Some(host) => host.trim_end_matches('.').trim_start_matches("www.").to_string(),
        None => {
            let error = ErrorResponse {
                error: "No host in app URL".to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    // Origin must be the full URL including protocol, but without path, query, or fragment
    let origin = format!(
        "{}://{}{}",
        url.scheme(),
        url.host_str().unwrap(),
        if let Some(port) = url.port() {
            format!(":{}", port)
        } else {
            "".to_string()
        }
    );

    let service = match MfaWebAuthnService::new(
        &origin,  // Full origin URL with protocol and port
        &rp_id,   // Domain only (no port) for RP ID
        &config.app.name,
    ) {
        Ok(s) => s,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("WebAuthn service error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    match service
        .start_authentication(&pool, auth_user.user_id.clone())
        .await
    {
        Ok(challenge) => (StatusCode::OK, ResponseJson(challenge)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Finish WebAuthn authentication
pub async fn webauthn_auth_finish(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(credential): Json<PublicKeyCredential>,
) -> impl IntoResponse {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Config error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    // Extract domain from URL and ensure it's the proper format
    let url = match url::Url::parse(&config.app.url) {
        Ok(u) => u,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Invalid app URL: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    // For WebAuthn, the RP ID must be a registrable domain suffix of the origin
    let rp_id = match url.host_str() {
        Some(host) => host.trim_end_matches('.').trim_start_matches("www.").to_string(),
        None => {
            let error = ErrorResponse {
                error: "No host in app URL".to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    // Origin must be the full URL including protocol, but without path, query, or fragment
    let origin = format!(
        "{}://{}{}",
        url.scheme(),
        url.host_str().unwrap(),
        if let Some(port) = url.port() {
            format!(":{}", port)
        } else {
            "".to_string()
        }
    );

    let service = match MfaWebAuthnService::new(
        &origin,  // Full origin URL with protocol and port
        &rp_id,   // Domain only (no port) for RP ID
        &config.app.name,
    ) {
        Ok(s) => s,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("WebAuthn service error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    match service
        .finish_authentication(&pool, auth_user.user_id.clone(), credential)
        .await
    {
        Ok(is_valid) => {
            let response = json!({
                "verified": is_valid,
                "message": "WebAuthn authentication successful"
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

/// List WebAuthn credentials
pub async fn webauthn_list_credentials(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    match MfaWebAuthnService::list_credentials(&pool, auth_user.user_id.clone()).await {
        Ok(credentials) => {
            let response_credentials: Vec<_> = credentials
                .iter()
                .map(|c| c.to_response())
                .collect();

            let response = json!({
                "credentials": response_credentials
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

/// Delete WebAuthn credential
pub async fn webauthn_delete_credential(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(credential_id): Path<String>,
) -> impl IntoResponse {
    match MfaWebAuthnService::delete_credential(&pool, &credential_id).await {
        Ok(_) => {
            let response = SuccessResponse {
                message: "WebAuthn credential deleted successfully".to_string(),
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

// ===== BACKUP CODES ENDPOINTS =====

/// Generate new backup codes
pub async fn generate_backup_codes(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    match MfaService::regenerate_backup_codes(&pool, auth_user.user_id.clone(), "").await {
        Ok(codes) => {
            let response = json!({
                "codes": codes,
                "message": "Backup codes generated successfully"
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

/// Verify backup code
pub async fn verify_backup_code(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<VerifyBackupCodeRequest>,
) -> impl IntoResponse {
    match MfaService::verify_backup_code(&pool, auth_user.user_id.clone(), &payload.code).await {
        Ok(is_valid) => {
            let response = json!({
                "verified": is_valid,
                "message": if is_valid { "Backup code verified" } else { "Invalid backup code" }
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

#[derive(Deserialize)]
pub struct VerifyBackupCodeRequest {
    pub code: String,
}

// ===== BIOMETRIC ENDPOINTS =====

/// Start biometric registration
pub async fn biometric_register_start(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<BiometricRegistrationStartRequest>,
) -> impl IntoResponse {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Config error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    let service = match MfaBiometricService::new(
        &config.app.url,
        &config.app.url.replace("https://", "").replace("http://", ""),
        &config.app.name,
    ) {
        Ok(s) => s,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Biometric service error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    match service
        .start_registration(
            &pool,
            auth_user.user_id.clone(),
            payload.biometric_type.clone(),
            payload.platform.clone(),
            payload.device_name.clone(),
        )
        .await
    {
        Ok(challenge) => (StatusCode::OK, ResponseJson(challenge)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Finish biometric registration
pub async fn biometric_register_finish(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<BiometricRegistrationRequest>,
) -> impl IntoResponse {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Config error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    let service = match MfaBiometricService::new(
        &config.app.url,
        &config.app.url.replace("https://", "").replace("http://", ""),
        &config.app.name,
    ) {
        Ok(s) => s,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Biometric service error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    // Parse credential from request
    // Note: In production, you would parse the actual RegisterPublicKeyCredential from the request
    // This is a placeholder implementation
    let error = ErrorResponse {
        error: "Not implemented - requires full WebAuthn credential parsing".to_string(),
    };
    (StatusCode::NOT_IMPLEMENTED, ResponseJson(error)).into_response()
}

/// Start biometric authentication
pub async fn biometric_auth_start(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Config error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    let service = match MfaBiometricService::new(
        &config.app.url,
        &config.app.url.replace("https://", "").replace("http://", ""),
        &config.app.name,
    ) {
        Ok(s) => s,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Biometric service error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    match service
        .start_authentication(&pool, auth_user.user_id.clone())
        .await
    {
        Ok(challenge) => (StatusCode::OK, ResponseJson(challenge)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Finish biometric authentication
pub async fn biometric_auth_finish(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<BiometricAuthenticationRequest>,
) -> impl IntoResponse {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Config error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    let service = match MfaBiometricService::new(
        &config.app.url,
        &config.app.url.replace("https://", "").replace("http://", ""),
        &config.app.name,
    ) {
        Ok(s) => s,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Biometric service error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    // Parse credential from request
    // Note: In production, you would parse the actual PublicKeyCredential from the request
    let error = ErrorResponse {
        error: "Not implemented - requires full WebAuthn credential parsing".to_string(),
    };
    (StatusCode::NOT_IMPLEMENTED, ResponseJson(error)).into_response()
}

/// List biometric credentials
pub async fn biometric_list_credentials(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    match MfaBiometricService::list_credentials(&pool, auth_user.user_id.clone()).await {
        Ok(credentials) => {
            let response_credentials: Vec<_> = credentials
                .iter()
                .map(|c| c.to_response())
                .collect();

            let response = json!({
                "credentials": response_credentials
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

/// Delete biometric credential
pub async fn biometric_delete_credential(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(credential_id): Path<String>,
) -> impl IntoResponse {
    match MfaBiometricService::delete_credential(&pool, &credential_id).await {
        Ok(_) => {
            let response = SuccessResponse {
                message: "Biometric credential deleted successfully".to_string(),
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
