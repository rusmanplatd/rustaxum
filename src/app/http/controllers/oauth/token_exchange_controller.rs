use axum::{
    extract::State,
    response::Json,
    http::{StatusCode, HeaderMap},
};
use serde::Deserialize;
use base64::Engine;
use serde_json::{json, Value};
use crate::database::DbPool;
use crate::app::services::oauth::{TokenExchangeService, TokenExchangeRequest};

/// RFC 8693: OAuth 2.0 Token Exchange Controller
///
/// This controller handles token exchange requests for delegation and impersonation scenarios,
/// enabling secure token-to-token exchanges for complex authorization flows.

#[derive(Debug, Deserialize)]
pub struct TokenExchangeFormRequest {
    pub grant_type: String,
    pub resource: Option<String>,
    pub audience: Option<String>,
    pub scope: Option<String>,
    pub requested_token_type: Option<String>,
    pub subject_token: String,
    pub subject_token_type: String,
    pub actor_token: Option<String>,
    pub actor_token_type: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

/// Token exchange endpoint
/// POST /oauth/token (with grant_type=urn:ietf:params:oauth:grant-type:token-exchange)
pub async fn exchange_token(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    axum::extract::Form(form): axum::extract::Form<TokenExchangeFormRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate grant type
    if form.grant_type != "urn:ietf:params:oauth:grant-type:token-exchange" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "unsupported_grant_type",
                "error_description": "Only token exchange grant type is supported"
            }))
        ));
    }

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

    // Create token exchange request
    let exchange_request = TokenExchangeRequest {
        grant_type: form.grant_type,
        resource: form.resource,
        audience: form.audience,
        scope: form.scope,
        requested_token_type: form.requested_token_type,
        subject_token: form.subject_token,
        subject_token_type: form.subject_token_type,
        actor_token: form.actor_token,
        actor_token_type: form.actor_token_type,
    };

    // Perform token exchange
    match TokenExchangeService::exchange_token(&pool, &client_id, exchange_request).await {
        Ok(response) => {
            tracing::info!("Token exchange successful for client: {}", client_id);
            Ok(Json(json!({
                "access_token": response.access_token,
                "issued_token_type": response.issued_token_type,
                "token_type": response.token_type,
                "expires_in": response.expires_in,
                "scope": response.scope,
                "refresh_token": response.refresh_token
            })))
        }
        Err(err) => {
            tracing::error!("Token exchange failed for client {}: {}", client_id, err);
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_grant",
                    "error_description": err.to_string()
                }))
            ))
        }
    }
}

/// Get supported token types
/// GET /oauth/token-exchange/supported-types
pub async fn get_supported_token_types(
    State(_pool): State<DbPool>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Ok(Json(json!({
        "supported_subject_token_types": [
            "urn:ietf:params:oauth:token-type:access_token",
            "urn:ietf:params:oauth:token-type:refresh_token",
            "urn:ietf:params:oauth:token-type:id_token",
            "urn:ietf:params:oauth:token-type:jwt"
        ],
        "supported_actor_token_types": [
            "urn:ietf:params:oauth:token-type:access_token",
            "urn:ietf:params:oauth:token-type:jwt"
        ],
        "supported_requested_token_types": [
            "urn:ietf:params:oauth:token-type:access_token"
        ],
        "supported_exchange_scenarios": [
            "delegation",
            "impersonation",
            "service_to_service"
        ]
    })))
}

/// Validate token exchange request (for testing/debugging)
/// POST /oauth/token-exchange/validate
pub async fn validate_exchange_request(
    State(_pool): State<DbPool>,
    headers: HeaderMap,
    Json(request): Json<TokenExchangeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Extract client credentials
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

    // Validate request structure
    let mut validation_results = Vec::new();

    // Validate grant type
    if request.grant_type != "urn:ietf:params:oauth:grant-type:token-exchange" {
        validation_results.push("Invalid grant_type");
    }

    // Validate subject token type
    let supported_subject_types = [
        "urn:ietf:params:oauth:token-type:access_token",
        "urn:ietf:params:oauth:token-type:refresh_token",
        "urn:ietf:params:oauth:token-type:id_token",
        "urn:ietf:params:oauth:token-type:jwt"
    ];
    if !supported_subject_types.contains(&request.subject_token_type.as_str()) {
        validation_results.push("Unsupported subject_token_type");
    }

    // Validate actor token type if present
    if let Some(actor_token_type) = &request.actor_token_type {
        let supported_actor_types = [
            "urn:ietf:params:oauth:token-type:access_token",
            "urn:ietf:params:oauth:token-type:jwt"
        ];
        if !supported_actor_types.contains(&actor_token_type.as_str()) {
            validation_results.push("Unsupported actor_token_type");
        }

        // Actor token must be present if actor token type is specified
        if request.actor_token.is_none() {
            validation_results.push("actor_token required when actor_token_type is specified");
        }
    }

    // Validate requested token type
    if let Some(requested_type) = &request.requested_token_type {
        if requested_type != "urn:ietf:params:oauth:token-type:access_token" {
            validation_results.push("Only access_token requested_token_type is supported");
        }
    }

    let is_valid = validation_results.is_empty();

    Ok(Json(json!({
        "valid": is_valid,
        "client_id": client_id,
        "validation_errors": validation_results,
        "request_summary": {
            "grant_type": request.grant_type,
            "subject_token_type": request.subject_token_type,
            "actor_token_type": request.actor_token_type,
            "requested_token_type": request.requested_token_type,
            "has_subject_token": !request.subject_token.is_empty(),
            "has_actor_token": request.actor_token.is_some(),
            "resource": request.resource,
            "audience": request.audience,
            "scope": request.scope
        }
    })))
}

// Helper functions

/// Extract client credentials from various sources
fn extract_client_credentials(
    headers: &HeaderMap,
    form: &TokenExchangeFormRequest,
) -> Result<String, String> {
    // Try Authorization header first (Basic auth)
    if let Ok(client_id) = extract_client_credentials_from_headers(headers) {
        return Ok(client_id);
    }

    // Try form parameters
    if let Some(client_id) = &form.client_id {
        if let Some(_secret) = &form.client_secret {
            // In production, validate the client secret
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