use axum::{
    extract::{State, Path},
    response::Json,
    http::{StatusCode, HeaderMap},
};
use serde::Deserialize;
use base64::Engine;
use serde_json::{json, Value};
use crate::database::DbPool;
use crate::app::services::oauth::{CIBAService, BackchannelAuthRequest, CIBAMode};

/// RFC 8955: Client Initiated Backchannel Authentication (CIBA) Controller
///
/// This controller handles CIBA endpoints for decoupled authentication scenarios
/// where users authenticate on different devices than the ones consuming the service.

#[derive(Debug, Deserialize)]
pub struct CIBAAuthFormRequest {
    pub scope: Option<String>,
    pub client_notification_token: Option<String>,
    pub acr_values: Option<String>,
    pub login_hint_token: Option<String>,
    pub id_token_hint: Option<String>,
    pub login_hint: Option<String>,
    pub binding_message: Option<String>,
    pub user_code: Option<String>,
    pub requested_expiry: Option<u32>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CIBATokenFormRequest {
    pub grant_type: String,
    pub auth_req_id: String,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

/// Backchannel authentication request endpoint
/// POST /oauth/ciba/auth
pub async fn create_backchannel_auth_request(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    axum::extract::Form(form): axum::extract::Form<CIBAAuthFormRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Extract and authenticate client
    let client_id = match extract_client_credentials(&headers, &form) {
        Ok(id) => id,
        Err(err) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "invalid_client",
                    "error_description": err
                }))
            ));
        }
    };

    // Create backchannel auth request
    let ciba_request = BackchannelAuthRequest {
        scope: form.scope,
        client_notification_token: form.client_notification_token,
        acr_values: form.acr_values,
        login_hint_token: form.login_hint_token,
        id_token_hint: form.id_token_hint,
        login_hint: form.login_hint,
        binding_message: form.binding_message,
        user_code: form.user_code,
        requested_expiry: form.requested_expiry,
    };

    // Determine CIBA mode (for demo, using Poll mode)
    let ciba_mode = if form.client_notification_token.is_some() {
        CIBAMode::Ping
    } else {
        CIBAMode::Poll
    };

    match CIBAService::create_backchannel_auth_request(&pool, &client_id, ciba_request).await {
        Ok(response) => {
            tracing::info!("Created CIBA auth request {} for client: {}", response.auth_req_id, client_id);
            Ok(Json(json!({
                "auth_req_id": response.auth_req_id,
                "expires_in": response.expires_in,
                "interval": response.interval
            })))
        }
        Err(err) => {
            tracing::error!("Failed to create CIBA auth request for client {}: {}", client_id, err);
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_request",
                    "error_description": err.to_string()
                }))
            ))
        }
    }
}

/// Complete user authentication (called by authentication device)
/// POST /oauth/ciba/complete/{auth_req_id}
pub async fn complete_user_authentication(
    Path(auth_req_id): Path<String>,
    State(pool): State<DbPool>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user_id = payload.get("user_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_request",
                "error_description": "user_id is required"
            }))
        ))?;

    let authorized = payload.get("authorized")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    match CIBAService::complete_user_authentication(&pool, &auth_req_id, user_id, authorized).await {
        Ok(_) => {
            let status = if authorized { "authorized" } else { "denied" };
            tracing::info!("User {} authentication for CIBA request {}: {}", user_id, auth_req_id, status);
            Ok(Json(json!({
                "message": "Authentication completed",
                "status": status
            })))
        }
        Err(err) => {
            tracing::error!("Failed to complete CIBA authentication {}: {}", auth_req_id, err);
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_request",
                    "error_description": err.to_string()
                }))
            ))
        }
    }
}

/// Token exchange for CIBA (polling endpoint)
/// POST /oauth/token (with grant_type=urn:ietf:params:oauth:grant-type:ciba)
pub async fn exchange_ciba_for_tokens(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    axum::extract::Form(form): axum::extract::Form<CIBATokenFormRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate grant type
    if form.grant_type != "urn:ietf:params:oauth:grant-type:ciba" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "unsupported_grant_type",
                "error_description": "Only CIBA grant type is supported"
            }))
        ));
    }

    // Extract and authenticate client
    let client_id = match extract_client_credentials_from_form(&headers, &form) {
        Ok(id) => id,
        Err(err) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "invalid_client",
                    "error_description": err
                }))
            ));
        }
    };

    match CIBAService::exchange_ciba_for_tokens(&pool, &client_id, &form.auth_req_id).await {
        Ok(response) => {
            tracing::info!("CIBA token exchange successful for auth_req_id: {}", form.auth_req_id);
            Ok(Json(json!({
                "access_token": response.access_token,
                "token_type": response.token_type,
                "expires_in": response.expires_in,
                "refresh_token": response.refresh_token,
                "scope": response.scope
            })))
        }
        Err(err) => {
            let error_msg = err.to_string();
            tracing::error!("CIBA token exchange failed for auth_req_id {}: {}", form.auth_req_id, error_msg);

            // Map specific CIBA errors to OAuth error codes
            let (error_code, status_code) = if error_msg.contains("authorization_pending") {
                ("authorization_pending", StatusCode::BAD_REQUEST)
            } else if error_msg.contains("slow_down") {
                ("slow_down", StatusCode::BAD_REQUEST)
            } else if error_msg.contains("access_denied") {
                ("access_denied", StatusCode::BAD_REQUEST)
            } else if error_msg.contains("expired_token") {
                ("expired_token", StatusCode::BAD_REQUEST)
            } else {
                ("invalid_grant", StatusCode::BAD_REQUEST)
            };

            Err((
                status_code,
                Json(json!({
                    "error": error_code,
                    "error_description": error_msg
                }))
            ))
        }
    }
}

/// Get CIBA authentication status
/// GET /oauth/ciba/status/{auth_req_id}
pub async fn get_auth_request_status(
    Path(auth_req_id): Path<String>,
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Extract client credentials for authorization
    let client_id = match extract_client_credentials_from_headers(&headers) {
        Ok(id) => id,
        Err(err) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "invalid_client",
                    "error_description": err
                }))
            ));
        }
    };

    match CIBAService::get_auth_request_status(&pool, &auth_req_id, &client_id).await {
        Ok(request) => {
            Ok(Json(json!({
                "auth_req_id": auth_req_id,
                "status": request.status,
                "expires_at": request.expires_at,
                "scope": request.scope,
                "binding_message": request.binding_message,
                "user_code": request.user_code
            })))
        }
        Err(err) => {
            tracing::error!("Failed to get CIBA status for {}: {}", auth_req_id, err);
            Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "invalid_request",
                    "error_description": "Authentication request not found"
                }))
            ))
        }
    }
}

/// Clean up expired CIBA requests
/// POST /oauth/ciba/cleanup (admin only)
pub async fn cleanup_expired_requests(
    State(pool): State<DbPool>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match CIBAService::cleanup_expired_requests(&pool).await {
        Ok(cleaned_count) => {
            tracing::info!("Cleaned up {} expired CIBA requests", cleaned_count);
            Ok(Json(json!({
                "message": "Cleanup completed",
                "cleaned_requests": cleaned_count
            })))
        }
        Err(err) => {
            tracing::error!("Failed to cleanup expired CIBA requests: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "server_error",
                    "error_description": "Failed to cleanup expired requests"
                }))
            ))
        }
    }
}

// Helper functions

/// Extract client credentials from various sources
fn extract_client_credentials(
    headers: &HeaderMap,
    form: &CIBAAuthFormRequest,
) -> Result<String, String> {
    // Try Authorization header first (Basic auth)
    if let Ok(client_id) = extract_client_credentials_from_headers(headers) {
        return Ok(client_id);
    }

    // Try form parameters
    if let Some(client_id) = &form.client_id {
        if let Some(_secret) = &form.client_secret {
            // TODO: In production, validate the client secret
            return Ok(client_id.clone());
        }
        // Public client (no secret required)
        return Ok(client_id.clone());
    }

    Err("Client authentication required".to_string())
}

/// Extract client credentials from CIBA token form
fn extract_client_credentials_from_form(
    headers: &HeaderMap,
    form: &CIBATokenFormRequest,
) -> Result<String, String> {
    // Try Authorization header first (Basic auth)
    if let Ok(client_id) = extract_client_credentials_from_headers(headers) {
        return Ok(client_id);
    }

    // Try form parameters
    if let Some(client_id) = &form.client_id {
        if let Some(_secret) = &form.client_secret {
            // TODO: In production, validate the client secret
            return Ok(client_id.clone());
        }
        // Public client (no secret required)
        return Ok(client_id.clone());
    }

    Err("Client authentication required".to_string())
}

/// Extract client credentials from Authorization header
fn extract_client_credentials_from_headers(headers: &HeaderMap) -> Result<String, String> {
    let auth_header = headers
        .get("authorization")
        .ok_or_else(|| "Authorization header missing".to_string())?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| "Invalid Authorization header".to_string())?;

    if !auth_str.starts_with("Basic ") {
        return Err("Only Basic authentication supported".to_string());
    }

    let encoded = auth_str.strip_prefix("Basic ").unwrap();
    let decoded_bytes = base64::engine::general_purpose::STANDARD.decode(encoded)
        .map_err(|_| "Invalid base64 in Authorization header".to_string())?;

    let decoded_str = String::from_utf8(decoded_bytes)
        .map_err(|_| "Invalid UTF-8 in Authorization header".to_string())?;

    let client_id = decoded_str
        .split(':')
        .next()
        .ok_or_else(|| "Invalid credential format".to_string())?;

    Ok(client_id.to_string())
}