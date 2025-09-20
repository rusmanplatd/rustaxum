use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sha2::{Digest, Sha256};

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub async fn auth_middleware(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = &header[7..];
            if is_valid_token(&pool, token).await {
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

async fn is_valid_token(pool: &PgPool, token: &str) -> bool {
    // Load config for JWT secret
    let config = match Config::load() {
        Ok(config) => config,
        Err(_) => return false,
    };

    // Decode JWT token
    let validation = Validation::new(Algorithm::HS256);
    let _decoded = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.auth.jwt_secret.as_ref()),
        &validation,
    ) {
        Ok(decoded) => decoded,
        Err(_) => return false,
    };

    // Create token hash for blacklist check
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());
    return false
}