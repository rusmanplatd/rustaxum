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
    pub jti: String,
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

                let user_id_for_log = user_id.clone();
                tokio::spawn(async move {
                    if let Err(e) = logger.log_custom(
                        &format!("User {} authenticated successfully via bearer token", user_id_for_log),
                        Some("auth.success"),
                        Some(properties)
                    ).await {
                        eprintln!("Failed to log authentication success: {}", e);
                    }
                });

                // Store user info in request extensions for later use
                request.extensions_mut().insert(user_id);

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

pub async fn is_valid_token(_pool: &DbPool, token: &str) -> Option<String> {
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

    // Check token blacklist in database
    let token_id = decoded.claims.jti;
    if is_token_revoked(_pool, &token_id).await.unwrap_or(false) {
        tracing::warn!("Attempted use of revoked token: {}", token_id);
        return None;
    }

    Some(decoded.claims.sub)
}

/// Validate JWT token without database lookup (synchronous version)
pub fn validate_jwt_token(token: &str) -> Option<String> {
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

    Some(decoded.claims.sub)
}

/// Check if the current user has admin privileges
pub async fn verify_admin_access(headers: &HeaderMap, pool: &DbPool) -> Result<String, StatusCode> {
    use diesel::prelude::*;
    use crate::schema::sys_users;
    use crate::app::models::user::User;

    // Extract token from Authorization header
    let token = extract_bearer_token(headers)?;

    // Validate the JWT token
    let user_id = validate_jwt_token(&token)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Get user from database and check if they are admin
    let mut conn = pool.get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user: User = sys_users::table
        .filter(sys_users::id.eq(&user_id))
        .select(User::as_select())
        .first(&mut conn)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Check if user has admin role
    // Note: This assumes you have an is_admin field or role-based system
    // Adjust based on your actual user model structure
    if user.email.ends_with("@admin.com") || user_id == "admin_user_id" {
        Ok(user_id)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

/// Extract bearer token from Authorization header
pub fn extract_bearer_token(headers: &HeaderMap) -> Result<String, StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        Ok(token.to_string())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// Check if token is revoked using OAuth token introspection
async fn is_token_revoked(pool: &DbPool, token_id: &str) -> Result<bool, anyhow::Error> {
    use diesel::prelude::*;

    // Query the access tokens table to check if token is revoked
    let mut conn = pool.get()?;

    let is_revoked: bool = crate::schema::oauth_access_tokens::table
        .filter(crate::schema::oauth_access_tokens::id.eq(token_id))
        .select(crate::schema::oauth_access_tokens::revoked)
        .first(&mut conn)
        .unwrap_or(true); // Default to revoked if not found

    Ok(is_revoked)
}