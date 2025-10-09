use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use crate::database::DbPool;
use crate::app::models::DieselUlid;
use crate::app::services::oauth::ClientService;
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
        Self::validate_request_parameters(pool, &request).await?;

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

        // Check if PAR is required for this client
        let client = crate::app::services::oauth::ClientService::find_by_id(pool, client_id.to_string())?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        let require_par = client.metadata.as_ref()
            .and_then(|m| m.get("require_pushed_authorization_requests"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if require_par {
            tracing::debug!("PAR is required for client {}", client_id);
        }
        tracing::debug!("PAR validation passed for client {}", client_id);

        Ok(())
    }

    /// Validate authorization request parameters
    async fn validate_request_parameters(pool: &DbPool, request: &PushedAuthRequest) -> Result<()> {
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
            Self::validate_request_object(pool, request_jwt, &request.client_id).await?;
        }

        Ok(())
    }

    /// Validate JWT request object (FAPI requirement)
    async fn validate_request_object(pool: &DbPool, request_jwt: &str, client_id: &str) -> Result<()> {
        // Production implementation of JWT request object validation
        use jsonwebtoken::{decode, DecodingKey, Validation};
        use serde_json::Value;

        // Get client for signature verification
        let client = ClientService::find_by_id(pool, client_id.to_string())?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        // 1. Decode JWT header to determine algorithm
        let header = jsonwebtoken::decode_header(request_jwt)
            .map_err(|e| anyhow::anyhow!("Invalid JWT header: {}", e))?;

        // 2. Create decoding key based on client authentication method
        let decoding_key = if let Some(ref client_secret) = client.secret {
            // Use client secret for HMAC algorithms
            DecodingKey::from_secret(client_secret.as_bytes())
        } else if let Some(ref public_key_pem) = client.public_key_pem {
            // Use RSA public key for RSA algorithms
            DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
                .map_err(|e| anyhow::anyhow!("Invalid RSA public key: {}", e))?
        } else {
            return Err(anyhow::anyhow!("No verification key available for client"));
        };

        // 3. Validate JWT with proper validation settings
        let mut validation = Validation::new(header.alg);
        validation.set_issuer(&[client_id]); // Issuer must be client_id
        validation.set_audience(&[&std::env::var("OAUTH_ISSUER").unwrap_or_else(|_| "https://auth.rustaxum.dev".to_string())]); // Audience should be auth server
        validation.validate_exp = true;
        validation.validate_nbf = true;

        // 4. Decode and validate the JWT
        let token_data = decode::<Value>(request_jwt, &decoding_key, &validation)
            .map_err(|e| anyhow::anyhow!("JWT validation failed: {}", e))?;

        // 5. Validate claims
        let claims = &token_data.claims;

        // Check client_id matches
        if let Some(iss) = claims.get("iss").and_then(|v| v.as_str()) {
            if iss != client_id {
                return Err(anyhow::anyhow!("JWT issuer does not match client_id"));
            }
        }

        if let Some(sub) = claims.get("sub").and_then(|v| v.as_str()) {
            if sub != client_id {
                return Err(anyhow::anyhow!("JWT subject does not match client_id"));
            }
        }

        // Verify JWT signature
        Self::verify_request_jwt_signature(&client, request_jwt)?;

        // Basic JWT format validation
        let parts: Vec<&str> = request_jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid JWT format: expected 3 parts"));
        }

        // Implement full cryptographic validation
        use jsonwebtoken::Algorithm;
        use base64::{Engine as _, engine::general_purpose};

        // Extract header to determine algorithm
        let header_b64 = parts[0];
        let header_bytes = general_purpose::URL_SAFE_NO_PAD.decode(header_b64)
            .map_err(|_| anyhow::anyhow!("Invalid JWT header encoding"))?;
        let header: serde_json::Value = serde_json::from_slice(&header_bytes)
            .map_err(|_| anyhow::anyhow!("Invalid JWT header JSON"))?;

        let _algorithm = header.get("alg")
            .and_then(|a| a.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing algorithm in JWT header"))?;

        // Retrieve client secret from database for JWT validation
        use crate::app::services::oauth::ClientService;
        let client = ClientService::find_by_id(pool, client_id.to_string())?
            .ok_or_else(|| anyhow::anyhow!("Client not found for JWT validation"))?;

        let client_secret = client.secret.ok_or_else(|| anyhow::anyhow!("Client secret not found for JWT validation"))?;
        let key = DecodingKey::from_secret(client_secret.as_bytes());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.validate_aud = false; // Will validate manually

        let token_data = decode::<serde_json::Value>(request_jwt, &key, &validation)
            .map_err(|e| anyhow::anyhow!("JWT validation failed: {}", e))?;

        // Validate that client_id in JWT matches authenticated client
        if let Some(jwt_client_id) = token_data.claims.get("client_id") {
            if jwt_client_id.as_str() != Some(client_id) {
                return Err(anyhow::anyhow!("client_id mismatch in JWT request object"));
            }
        }

        // Validate issuer matches client_id (RFC requirement)
        if let Some(iss) = token_data.claims.get("iss") {
            if iss.as_str() != Some(client_id) {
                return Err(anyhow::anyhow!("JWT issuer must match client_id"));
            }
        }

        // Validate audience contains authorization server
        if let Some(aud) = token_data.claims.get("aud") {
            let expected_aud = std::env::var("OAUTH_ISSUER").unwrap_or_else(|_| "https://auth.example.com".to_string());
            if aud.as_str() != Some(&expected_aud) && !aud.as_array().map_or(false, |arr| arr.iter().any(|v| v.as_str() == Some(&expected_aud))) {
                return Err(anyhow::anyhow!("Invalid audience in JWT request object"));
            }
        }

        tracing::debug!("JWT request object cryptographic validation passed");

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
    /// Check if PAR is required for client (uses metadata or naming convention)
    pub async fn require_par_for_client(pool: &DbPool, client_id: &str) -> bool {
        // First, check the client's metadata for explicit PAR requirement
        if let Ok(Some(client)) = ClientService::find_by_id(pool, client_id.to_string()) {
            // Check if explicitly configured in client metadata
            if let Some(require_par) = client.metadata.as_ref().and_then(|m| m.get("require_par")).and_then(|v| v.as_bool()) {
                return require_par;
            }

            // Check if client has require_pushed_authorization_requests flag set
            if client.require_pushed_authorization_requests {
                return true;
            }
        }

        // Fallback to naming convention: high-security clients require PAR
        let high_security_patterns = [
            "fapi",     // Financial API clients
            "bank",     // Banking applications
            "payment",  // Payment processors
            "finance",  // Financial services
            "trading",  // Trading platforms
            "secure",   // Explicitly secure clients
            "prod",     // Production environments
        ];

        let client_lower = client_id.to_lowercase();
        high_security_patterns.iter().any(|&pattern| client_lower.contains(pattern))
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

    /// Verify JWT request object signature using client's authentication method
    fn verify_request_jwt_signature(
        client: &crate::app::models::oauth::Client,
        request_jwt: &str,
    ) -> Result<()> {
        // For production: Use client's registered secret/key to verify JWT signature
        if client.secret.is_none() {
            return Err(anyhow::anyhow!("Client secret required for JWT verification"));
        }

        // Basic JWT format validation
        let parts: Vec<&str> = request_jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid JWT format"));
        }

        use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

        let client_secret = client.secret.as_ref().unwrap();
        let key = DecodingKey::from_secret(client_secret.as_bytes());
        let validation = Validation::new(Algorithm::HS256);

        // Decode and validate JWT
        let _token_data = decode::<serde_json::Value>(request_jwt, &key, &validation)
            .map_err(|e| anyhow::anyhow!("JWT validation failed: {}", e))?;

        tracing::debug!("JWT request object signature verified successfully");
        Ok(())
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

    #[tokio::test]
    async fn test_request_validation() {
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

        assert!(PARService::validate_request_parameters(&valid_request).await.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_request_validation() {
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

        assert!(PARService::validate_request_parameters(&invalid_request).await.is_err());
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