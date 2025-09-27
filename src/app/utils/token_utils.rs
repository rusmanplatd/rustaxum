use anyhow::Result;
use sha2::{Sha256, Digest};
use uuid::Uuid;
use axum::http::HeaderMap;

pub struct TokenUtils;

impl TokenUtils {
    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn generate_reset_token() -> String {
        Uuid::new_v4().to_string().replace('-', "")
    }

    pub fn extract_token_from_header(auth_header: Option<&str>) -> Result<&str> {
        let header = auth_header.ok_or_else(|| anyhow::anyhow!("Authorization header missing"))?;

        if !header.starts_with("Bearer ") {
            anyhow::bail!("Invalid authorization header format");
        }

        Ok(&header[7..])
    }

    /// Extract user ID from Authorization header following Laravel patterns
    /// Returns None if no valid token or user ID found
    pub fn extract_user_id_from_headers(headers: &HeaderMap) -> Option<crate::app::models::DieselUlid> {
        use crate::app::services::auth_service::AuthService;

        // Extract token from Authorization header
        let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());

        let token = Self::extract_token_from_header(auth_header).ok()?;

        // Decode token to get user ID
        let claims = AuthService::decode_token(token).ok()?;

        // Convert string ID to DieselUlid
        crate::app::models::DieselUlid::from_string(&claims.sub).ok()
    }

    /// Decode JWT token and return claims
    /// This is a convenience method that wraps AuthService::decode_token
    pub fn decode_jwt_token(token: &str) -> Result<crate::app::services::auth_service::Claims> {
        use crate::app::services::auth_service::AuthService;
        AuthService::decode_token(token)
    }
}