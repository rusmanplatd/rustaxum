use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub jwt_secret: String,
    pub access_token_ttl: u64,
    pub refresh_token_ttl: u64,
    pub auth_code_ttl: u64,
}

impl OAuthConfig {
    pub fn from_env() -> Result<Self> {
        Ok(OAuthConfig {
            jwt_secret: env::var("OAUTH_JWT_SECRET")
                .unwrap_or_else(|_| "your-oauth2-jwt-secret-here".to_string()),
            access_token_ttl: env::var("OAUTH_ACCESS_TOKEN_TTL")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600),
            refresh_token_ttl: env::var("OAUTH_REFRESH_TOKEN_TTL")
                .unwrap_or_else(|_| "604800".to_string())
                .parse()
                .unwrap_or(604800),
            auth_code_ttl: env::var("OAUTH_AUTH_CODE_TTL")
                .unwrap_or_else(|_| "600".to_string())
                .parse()
                .unwrap_or(600),
        })
    }
}