use anyhow::{anyhow, Result};
use axum::extract::Request;
use axum::http::HeaderMap;
use base64::{engine::general_purpose, Engine as _};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::services::session::SessionStore;

#[derive(Clone)]
pub struct CSRFService {
    token_name: String,
    header_name: String,
    lifetime: u64, // in seconds
}

impl CSRFService {
    pub fn new() -> Self {
        Self {
            token_name: "_token".to_string(),
            header_name: "X-CSRF-TOKEN".to_string(),
            lifetime: 3600, // 1 hour default
        }
    }

    pub fn with_config(token_name: String, header_name: String, lifetime: u64) -> Self {
        Self {
            token_name,
            header_name,
            lifetime,
        }
    }

    /// Generate a new CSRF token
    pub fn generate_token(&self) -> String {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.gen();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut payload = Vec::new();
        payload.extend_from_slice(&timestamp.to_be_bytes());
        payload.extend_from_slice(&random_bytes);

        general_purpose::STANDARD.encode(payload)
    }

    /// Validate a CSRF token
    pub fn validate_token(&self, token: &str) -> bool {
        if let Ok(decoded) = general_purpose::STANDARD.decode(token) {
            if decoded.len() >= 8 {
                let timestamp_bytes = &decoded[0..8];
                let timestamp = u64::from_be_bytes([
                    timestamp_bytes[0], timestamp_bytes[1], timestamp_bytes[2], timestamp_bytes[3],
                    timestamp_bytes[4], timestamp_bytes[5], timestamp_bytes[6], timestamp_bytes[7],
                ]);

                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                // Check if token is not expired
                if current_time - timestamp <= self.lifetime {
                    return true;
                }
            }
        }
        false
    }

    /// Get or generate CSRF token for the session
    pub async fn token(&self, session_store: &SessionStore) -> Result<String> {
        let token_key = format!("csrf_{}", self.token_name);

        // Try to get existing token from session
        if let Some(token_value) = session_store.get(&token_key).await {
            if let Some(existing_token) = token_value.as_str() {
                if self.validate_token(existing_token) {
                    return Ok(existing_token.to_string());
                }
            }
        }

        // Generate new token if none exists or expired
        let new_token = self.generate_token();
        session_store.put(&token_key, serde_json::Value::String(new_token.clone())).await;

        Ok(new_token)
    }

    /// Verify CSRF token from request
    pub async fn verify_request(&self, request: &Request, session_store: &SessionStore) -> Result<bool> {
        let token = self.extract_token_from_request(request)?;
        self.verify_token(&token, session_store).await
    }

    /// Extract CSRF token from request (header or form data)
    fn extract_token_from_request(&self, request: &Request) -> Result<String> {
        // First try to get from header
        if let Some(header_value) = request.headers().get(&self.header_name) {
            if let Ok(token) = header_value.to_str() {
                return Ok(token.to_string());
            }
        }

        // If not in header, it might be in form data (this would need to be extracted by the caller)
        Err(anyhow!("CSRF token not found in request"))
    }

    /// Extract CSRF token from headers specifically
    pub fn extract_token_from_headers(&self, headers: &HeaderMap) -> Option<String> {
        headers
            .get(&self.header_name)
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_string())
    }

    /// Extract CSRF token from form data
    pub fn extract_token_from_form(&self, form_data: &str) -> Option<String> {
        // Simple form parsing for _token field
        for pair in form_data.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                if key == self.token_name {
                    return Some(urlencoding::decode(value).ok()?.to_string());
                }
            }
        }
        None
    }

    /// Verify token against session
    pub async fn verify_token(&self, token: &str, session_store: &SessionStore) -> Result<bool> {
        if !self.validate_token(token) {
            return Ok(false);
        }

        let token_key = format!("csrf_{}", self.token_name);

        // Get token from session
        if let Some(token_value) = session_store.get(&token_key).await {
            if let Some(session_token) = token_value.as_str() {
                return Ok(session_token == token);
            }
        }

        Ok(false)
    }

    /// Generate token hash for comparison (optional security enhancement)
    pub fn hash_token(&self, token: &str, session_id: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hasher.update(session_id.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Get token name for forms
    pub fn token_name(&self) -> &str {
        &self.token_name
    }

    /// Get header name
    pub fn header_name(&self) -> &str {
        &self.header_name
    }

    /// Check if request should be exempt from CSRF protection
    pub fn is_exempt_request(&self, request: &Request) -> bool {
        // Exempt GET, HEAD, OPTIONS requests
        matches!(request.method().as_str(), "GET" | "HEAD" | "OPTIONS")
    }

    /// Regenerate token (useful after authentication)
    pub async fn regenerate_token(&self, session_store: &SessionStore) -> Result<String> {
        let token_key = format!("csrf_{}", self.token_name);
        let new_token = self.generate_token();
        session_store.put(&token_key, serde_json::Value::String(new_token.clone())).await;
        Ok(new_token)
    }
}

impl Default for CSRFService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token() {
        let csrf = CSRFService::new();
        let token1 = csrf.generate_token();
        let token2 = csrf.generate_token();

        assert_ne!(token1, token2);
        assert!(!token1.is_empty());
        assert!(!token2.is_empty());
    }

    #[test]
    fn test_validate_token() {
        let csrf = CSRFService::new();
        let token = csrf.generate_token();

        assert!(csrf.validate_token(&token));
        assert!(!csrf.validate_token("invalid_token"));
    }

    #[test]
    fn test_extract_token_from_form() {
        let csrf = CSRFService::new();
        let form_data = "_token=abc123&name=John&email=john@example.com";

        let token = csrf.extract_token_from_form(form_data);
        assert_eq!(token, Some("abc123".to_string()));
    }

    #[test]
    fn test_hash_token() {
        let csrf = CSRFService::new();
        let hash1 = csrf.hash_token("token123", "session456");
        let hash2 = csrf.hash_token("token123", "session456");
        let hash3 = csrf.hash_token("token123", "session789");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}