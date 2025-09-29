use axum::{
    extract::{State, Path},
    response::Json,
    http::{StatusCode, HeaderMap},
};
use serde_json::{json, Value};
use base64::Engine;
use crate::database::DbPool;
use crate::app::services::oauth::{PARService, PushedAuthRequest};
// Remove unused middleware import - use direct authentication via headers

/// RFC 9126: OAuth 2.0 Pushed Authorization Requests Controller
///
/// This controller handles PAR endpoints allowing clients to push authorization
/// request parameters to the server before redirecting users to authorization endpoint.

/// Create pushed authorization request
/// POST /oauth/par
pub async fn create_pushed_request(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Json(request): Json<PushedAuthRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Extract client credentials from Authorization header or form data
    let client_id = match extract_client_id(&headers, &request.client_id) {
        Ok(id) => id,
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_request",
                    "error_description": err
                }))
            ));
        }
    };

    // Authenticate the client
    if let Err(err) = authenticate_client(&pool, &client_id, &headers).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "invalid_client",
                "error_description": err
            }))
        ));
    }

    // Create pushed authorization request
    match PARService::create_pushed_request(&pool, &client_id, request).await {
        Ok(response) => {
            tracing::info!("Created PAR request for client: {}", client_id);
            Ok(Json(json!({
                "request_uri": response.request_uri,
                "expires_in": response.expires_in
            })))
        }
        Err(err) => {
            tracing::error!("Failed to create PAR request: {}", err);
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

/// Get authorization URL with request_uri
/// GET /oauth/par/authorize?request_uri=urn:...&client_id=...
pub async fn create_authorization_url(
    State(_pool): State<DbPool>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let client_id = params.get("client_id")
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_request",
                "error_description": "client_id parameter is required"
            }))
        ))?;

    let request_uri = params.get("request_uri")
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_request",
                "error_description": "request_uri parameter is required"
            }))
        ))?;

    let state = params.get("state");

    // Create authorization URL from configuration
    let authorization_endpoint = std::env::var("OAUTH_AUTHORIZATION_ENDPOINT")
        .unwrap_or_else(|_| "https://auth.rustaxum.dev/oauth/authorize".to_string());
    let authorization_url = PARService::create_authorization_url(
        authorization_endpoint,
        client_id,
        request_uri,
        state.map(|s| s.as_str()),
    );

    Ok(Json(json!({
        "authorization_url": authorization_url
    })))
}

/// Clean up expired PAR requests
/// POST /oauth/par/cleanup (admin only)
pub async fn cleanup_expired_requests(
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check admin authentication
    if let Err(status) = crate::app::http::middleware::auth_middleware::verify_admin_access(&headers, &pool).await {
        return Err((
            status,
            Json(json!({
                "error": "access_denied",
                "error_description": "Admin privileges required"
            }))
        ));
    }

    match PARService::cleanup_expired_requests(&pool).await {
        Ok(cleaned_count) => {
            tracing::info!("Cleaned up {} expired PAR requests", cleaned_count);
            Ok(Json(json!({
                "message": "Cleanup completed",
                "cleaned_requests": cleaned_count
            })))
        }
        Err(err) => {
            tracing::error!("Failed to cleanup expired PAR requests: {}", err);
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

/// Check if PAR is required for a client
/// GET /oauth/par/required/{client_id}
pub async fn check_par_requirement(
    Path(client_id): Path<String>,
    State(_pool): State<DbPool>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let par_required = PARService::require_par_for_client(&client_id);

    Ok(Json(json!({
        "client_id": client_id,
        "par_required": par_required
    })))
}

// Helper functions

/// Extract client ID from request
fn extract_client_id(headers: &HeaderMap, form_client_id: &str) -> Result<String, String> {
    // First try Authorization header (for client_credentials auth)
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Basic ") {
                // Decode Basic auth and extract client_id
                let encoded = auth_str.strip_prefix("Basic ").unwrap();
                if let Ok(decoded_bytes) = base64::engine::general_purpose::STANDARD.decode(encoded) {
                    if let Ok(decoded_str) = String::from_utf8(decoded_bytes) {
                        if let Some(client_id) = decoded_str.split(':').next() {
                            return Ok(client_id.to_string());
                        }
                    }
                }
            }
        }
    }

    // Fallback to form parameter
    if form_client_id.is_empty() {
        return Err("client_id is required".to_string());
    }

    Ok(form_client_id.to_string())
}

/// Authenticate client with proper error handling and logging
async fn authenticate_client(
    pool: &DbPool,
    client_id: &str,
    headers: &HeaderMap,
) -> Result<(), String> {
    use crate::app::services::oauth::ClientService;

    // Find client
    let client = ClientService::find_by_id(pool, client_id.to_string())
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Client not found".to_string())?;

    if client.revoked {
        return Err("Client is revoked".to_string());
    }

    // For confidential clients, validate client secret
    if let Some(_secret) = &client.secret {
        // Validate client secret from Authorization header using ClientAuthService
        use crate::app::services::oauth::ClientAuthService;

        if let Some(auth_header) = headers.get("authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                // Parse Basic authentication or JWT client assertion
                match ClientAuthService::authenticate_client_from_header(pool, &client.id.to_string(), auth_str).await {
                    Ok(_) => {}, // Authentication successful
                    Err(_) => return Err("Invalid client credentials".to_string()),
                }
            } else {
                return Err("Invalid authorization header format".to_string());
            }
        } else {
            return Err("Client authentication required".to_string());
        }
    }

    Ok(())
}

