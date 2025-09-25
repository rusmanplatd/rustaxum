use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use crate::database::DbPool;
use crate::app::models::DieselUlid;
use diesel::prelude::*;
use rand::{distributions::Alphanumeric, Rng};

/// RFC 9126: OAuth 2.0 Pushed Authorization Requests (PAR)
///
/// This service implements Pushed Authorization Requests, which allows clients to push
/// authorization request parameters to the authorization server via a direct HTTP request,
/// receiving a request URI that represents the authorization request data.
pub struct PARService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushedAuthRequest {
    // Standard OAuth parameters
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: Option<String>,
    pub state: Option<String>,

    // PKCE parameters (RFC 7636)
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,

    // Additional OAuth parameters
    pub response_mode: Option<String>,
    pub nonce: Option<String>,
    pub display: Option<String>,
    pub prompt: Option<String>,
    pub max_age: Option<u64>,
    pub ui_locales: Option<String>,
    pub id_token_hint: Option<String>,
    pub login_hint: Option<String>,
    pub acr_values: Option<String>,

    // Rich Authorization Requests (RFC 9396)
    pub authorization_details: Option<String>,

    // FAPI specific parameters
    pub request: Option<String>, // JWT request object
    pub request_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushedAuthResponse {
    pub request_uri: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = crate::schema::oauth_pushed_requests)]
pub struct StoredPushedRequest {
    pub id: DieselUlid,
    pub request_uri: String,
    pub client_id: String,
    pub request_data: String, // JSON serialized PushedAuthRequest
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PARService {
    /// Process a pushed authorization request
    /// RFC 9126: Accept authorization request parameters and return request_uri
    pub async fn create_pushed_request(
        pool: &DbPool,
        client_id: &str,
        request: PushedAuthRequest,
    ) -> Result<PushedAuthResponse> {
        // Validate client
        Self::validate_client(pool, client_id).await?;

        // Validate request parameters
        Self::validate_request_parameters(&request)?;

        // Generate request URI
        let request_uri = Self::generate_request_uri()?;

        // Store request data
        let expires_in = 600; // 10 minutes (RFC 9126 recommendation)
        let expires_at = Utc::now() + Duration::seconds(expires_in);

        let stored_request = StoredPushedRequest {
            id: DieselUlid::new(),
            request_uri: request_uri.clone(),
            client_id: client_id.to_string(),
            request_data: serde_json::to_string(&request)?,
            expires_at,
            used: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Self::store_pushed_request(pool, stored_request).await?;

        tracing::info!("Created pushed authorization request {} for client {}", request_uri, client_id);

        Ok(PushedAuthResponse {
            request_uri,
            expires_in,
        })
    }

    /// Retrieve and consume a pushed authorization request
    /// RFC 9126: Exchange request_uri for original authorization parameters
    pub async fn consume_pushed_request(
        pool: &DbPool,
        request_uri: &str,
        client_id: &str,
    ) -> Result<PushedAuthRequest> {
        let mut conn = pool.get()?;

        // Find the stored request
        let stored_request: StoredPushedRequest = oauth_pushed_requests::table
            .filter(oauth_pushed_requests::request_uri.eq(request_uri))
            .filter(oauth_pushed_requests::client_id.eq(client_id))
            .filter(oauth_pushed_requests::used.eq(false))
            .first(&mut conn)
            .map_err(|_| anyhow::anyhow!("Invalid or expired request_uri"))?;

        // Check if expired
        if Utc::now() > stored_request.expires_at {
            return Err(anyhow::anyhow!("Request URI has expired"));
        }

        // Mark as used (single-use)
        diesel::update(oauth_pushed_requests::table)
            .filter(oauth_pushed_requests::id.eq(stored_request.id))
            .set((
                oauth_pushed_requests::used.eq(true),
                oauth_pushed_requests::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        // Deserialize request data
        let request: PushedAuthRequest = serde_json::from_str(&stored_request.request_data)?;

        tracing::info!("Consumed pushed authorization request {} for client {}", request_uri, client_id);

        Ok(request)
    }

    /// Validate client is authorized to use PAR
    async fn validate_client(pool: &DbPool, client_id: &str) -> Result<()> {
        use crate::app::services::oauth::ClientService;

        let client = ClientService::find_by_id(pool, client_id.to_string())?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        // Check if client is revoked
        if client.revoked {
            return Err(anyhow::anyhow!("Client is revoked"));
        }

        // TODO: In production, made mandatory per client by adding a `require_par` field to the client configuration
        tracing::debug!("PAR validation passed for client {}", client_id);

        Ok(())
    }

    /// Validate authorization request parameters
    fn validate_request_parameters(request: &PushedAuthRequest) -> Result<()> {
        // Validate response_type
        if request.response_type != "code" {
            return Err(anyhow::anyhow!("Only 'code' response_type is supported in OAuth 2.1"));
        }

        // Validate client_id format
        if request.client_id.is_empty() {
            return Err(anyhow::anyhow!("client_id is required"));
        }

        // Validate redirect_uri
        if request.redirect_uri.is_empty() {
            return Err(anyhow::anyhow!("redirect_uri is required"));
        }

        // Validate redirect_uri format
        if !request.redirect_uri.starts_with("https://") && !request.redirect_uri.starts_with("http://localhost") {
            return Err(anyhow::anyhow!("redirect_uri must use HTTPS or be localhost"));
        }

        // OAuth 2.1: PKCE is mandatory
        if request.code_challenge.is_none() {
            return Err(anyhow::anyhow!("PKCE code_challenge is mandatory in OAuth 2.1"));
        }

        // Validate PKCE method
        if let Some(method) = &request.code_challenge_method {
            if method != "S256" && method != "plain" {
                return Err(anyhow::anyhow!("Invalid code_challenge_method"));
            }
        }

        // Validate scopes if present
        if let Some(scope) = &request.scope {
            if scope.contains("..") || scope.contains("  ") {
                return Err(anyhow::anyhow!("Invalid scope format"));
            }
        }

        // Validate request object if present (for FAPI compliance)
        if let Some(request_jwt) = &request.request {
            Self::validate_request_object(request_jwt)?;
        }

        Ok(())
    }

    /// Validate JWT request object (FAPI requirement)
    fn validate_request_object(request_jwt: &str) -> Result<()> {
        // For production implementation, you would:
        // 1. Verify JWT signature using client's public key or shared secret
        // 2. Validate standard claims (iat, exp, aud, iss)
        // 3. Check client_id matches the authenticated client
        // 4. Validate that request object parameters match PAR request
        // 5. Ensure no security-sensitive parameters are duplicated outside JWT

        // Basic JWT format validation
        let parts: Vec<&str> = request_jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid JWT format: expected 3 parts"));
        }

        // TODO: In production, implement full cryptographic validation
        tracing::debug!("JWT request object format validation passed");
        tracing::warn!("JWT request object signature validation not implemented - add proper JWT verification for production");

        if request_jwt.is_empty() {
            return Err(anyhow::anyhow!("Empty request object"));
        }

        // Basic JWT format validation
        let parts: Vec<&str> = request_jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid JWT format"));
        }

        Ok(())
    }

    /// Generate cryptographically secure request URI
    fn generate_request_uri() -> Result<String> {
        let random_part: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // RFC 9126: request_uri should use urn:ietf:params:oauth:request_uri: prefix
        Ok(format!("urn:ietf:params:oauth:request_uri:{}", random_part))
    }

    /// Store pushed request in database
    async fn store_pushed_request(
        pool: &DbPool,
        request: StoredPushedRequest,
    ) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::insert_into(oauth_pushed_requests::table)
            .values(request)
            .execute(&mut conn)?;

        Ok(())
    }

    /// Clean up expired pushed requests
    pub async fn cleanup_expired_requests(pool: &DbPool) -> Result<u64> {
        let mut conn = pool.get()?;

        let deleted = diesel::delete(oauth_pushed_requests::table)
            .filter(oauth_pushed_requests::expires_at.lt(Utc::now()))
            .execute(&mut conn)?;

        if deleted > 0 {
            tracing::info!("Cleaned up {} expired pushed authorization requests", deleted);
        }

        Ok(deleted as u64)
    }

    /// Validate that authorization request uses PAR when required
    pub fn require_par_for_client(client_id: &str) -> bool {
        // TODO: In production, this would check client configuration
        // High-security clients (FAPI) might require PAR
        client_id.contains("fapi") || client_id.contains("bank")
    }

    /// Create authorization URL with request_uri
    pub fn create_authorization_url(
        authorization_endpoint: &str,
        client_id: &str,
        request_uri: &str,
        state: Option<&str>,
    ) -> String {
        let mut url = format!(
            "{}?client_id={}&request_uri={}",
            authorization_endpoint,
            urlencoding::encode(client_id),
            urlencoding::encode(request_uri)
        );

        if let Some(state) = state {
            url.push_str(&format!("&state={}", urlencoding::encode(state)));
        }

        url
    }

    /// Extract request parameters for logging/audit
    pub fn extract_request_info_for_audit(
        request: &PushedAuthRequest,
    ) -> serde_json::Value {
        serde_json::json!({
            "client_id": request.client_id,
            "redirect_uri": request.redirect_uri,
            "response_type": request.response_type,
            "scope": request.scope,
            "has_pkce": request.code_challenge.is_some(),
            "has_request_object": request.request.is_some(),
            "has_authorization_details": request.authorization_details.is_some(),
        })
    }
}

// Use the oauth_pushed_requests table from schema.rs
use crate::schema::oauth_pushed_requests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_uri_generation() {
        let uri1 = PARService::generate_request_uri().unwrap();
        let uri2 = PARService::generate_request_uri().unwrap();

        assert_ne!(uri1, uri2);
        assert!(uri1.starts_with("urn:ietf:params:oauth:request_uri:"));
        assert!(uri2.starts_with("urn:ietf:params:oauth:request_uri:"));
    }

    #[test]
    fn test_request_validation() {
        let valid_request = PushedAuthRequest {
            response_type: "code".to_string(),
            client_id: "test_client".to_string(),
            redirect_uri: "https://example.com/callback".to_string(),
            scope: Some("read write".to_string()),
            state: Some("random_state".to_string()),
            code_challenge: Some("challenge".to_string()),
            code_challenge_method: Some("S256".to_string()),
            response_mode: None,
            nonce: None,
            display: None,
            prompt: None,
            max_age: None,
            ui_locales: None,
            id_token_hint: None,
            login_hint: None,
            acr_values: None,
            authorization_details: None,
            request: None,
            request_uri: None,
        };

        assert!(PARService::validate_request_parameters(&valid_request).is_ok());
    }

    #[test]
    fn test_invalid_request_validation() {
        let invalid_request = PushedAuthRequest {
            response_type: "token".to_string(), // Invalid for OAuth 2.1
            client_id: "test_client".to_string(),
            redirect_uri: "http://example.com/callback".to_string(), // Non-HTTPS
            scope: Some("read write".to_string()),
            state: Some("random_state".to_string()),
            code_challenge: None, // Missing PKCE
            code_challenge_method: None,
            response_mode: None,
            nonce: None,
            display: None,
            prompt: None,
            max_age: None,
            ui_locales: None,
            id_token_hint: None,
            login_hint: None,
            acr_values: None,
            authorization_details: None,
            request: None,
            request_uri: None,
        };

        assert!(PARService::validate_request_parameters(&invalid_request).is_err());
    }

    #[test]
    fn test_authorization_url_creation() {
        let url = PARService::create_authorization_url(
            "https://auth.example.com/oauth/authorize",
            "test_client",
            "urn:ietf:params:oauth:request_uri:abc123",
            Some("state123"),
        );

        assert!(url.contains("client_id=test_client"));
        assert!(url.contains("request_uri=urn%3Aietf%3Aparams%3Aoauth%3Arequest_uri%3Aabc123"));
        assert!(url.contains("state=state123"));
    }
}