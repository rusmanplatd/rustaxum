use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::database::DbPool;
use sha2::{Digest, Sha256};

use crate::config::Config;
use crate::app::http::middleware::activity_logging_middleware::activity_logger_from_request;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub async fn auth_middleware(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok());

    // Create activity logger for this request
    let logger = activity_logger_from_request(&request, "authentication");

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = &header[7..];

            if let Some(user_id) = is_valid_token(&pool, token).await {
                // Log successful authentication
                let properties = json!({
                    "auth_method": "bearer_token",
                    "token_length": token.len(),
                    "path": request.uri().path(),
                    "method": request.method().as_str()
                });

                tokio::spawn(async move {
                    if let Err(e) = logger.log_custom(
                        &format!("User {} authenticated successfully via bearer token", user_id),
                        Some("auth.success"),
                        Some(properties)
                    ).await {
                        eprintln!("Failed to log authentication success: {}", e);
                    }
                });

                // Store user info in request extensions for later use
                request.extensions_mut().insert(user_id.clone());

                Ok(next.run(request).await)
            } else {
                // Log failed authentication
                let properties = json!({
                    "auth_method": "bearer_token",
                    "failure_reason": "invalid_token",
                    "path": request.uri().path(),
                    "method": request.method().as_str()
                });

                tokio::spawn(async move {
                    if let Err(e) = logger.log_custom(
                        "Authentication failed: invalid bearer token",
                        Some("auth.failure"),
                        Some(properties)
                    ).await {
                        eprintln!("Failed to log authentication failure: {}", e);
                    }
                });

                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => {
            // Log missing authentication
            let properties = json!({
                "failure_reason": "missing_or_invalid_auth_header",
                "path": request.uri().path(),
                "method": request.method().as_str()
            });

            tokio::spawn(async move {
                if let Err(e) = logger.log_custom(
                    "Authentication failed: missing or invalid authorization header",
                    Some("auth.failure"),
                    Some(properties)
                ).await {
                    eprintln!("Failed to log authentication failure: {}", e);
                }
            });

            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

async fn is_valid_token(_pool: &DbPool, token: &str) -> Option<String> {
    // Load config for JWT secret
    let config = match Config::load() {
        Ok(config) => config,
        Err(_) => return None,
    };

    // Decode JWT token
    let validation = Validation::new(Algorithm::HS256);
    let decoded = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.auth.jwt_secret.as_ref()),
        &validation,
    ) {
        Ok(decoded) => decoded,
        Err(_) => return None,
    };

    // Create token hash for blacklist check
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let _token_hash = format!("{:x}", hasher.finalize());

    // TODO: Check token blacklist in database
    // For now, return the user ID from token claims
    Some(decoded.claims.sub)
}