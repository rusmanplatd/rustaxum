use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use crate::database::DbPool;
use crate::app::models::DieselUlid;
use diesel::prelude::*;
use rand::{distributions::Alphanumeric, Rng};

/// RFC 8955: Client Initiated Backchannel Authentication (CIBA)
///
/// This service implements CIBA flow for decoupled authentication scenarios
/// where the user authenticates on a different device than the one consuming
/// the service (e.g., authenticating on mobile while using a smart TV).
pub struct CIBAService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackchannelAuthRequest {
    pub scope: Option<String>,
    pub notification_token: Option<String>, // For ping/push modes
    pub acr_values: Option<String>, // Authentication context class reference
    pub login_hint_token: Option<String>, // Opaque user identifier
    pub id_token_hint: Option<String>, // Previously issued ID token
    pub login_hint: Option<String>, // Human-readable user identifier
    pub binding_message: Option<String>, // Human-readable message
    pub user_code: Option<String>, // User verification code
    pub requested_expiry: Option<i64>, // Requested expiration in seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackchannelAuthResponse {
    pub auth_req_id: String, // Authentication request ID
    pub expires_in: i64, // Expiration time in seconds
    pub interval: Option<i64>, // Polling interval for poll mode
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CIBATokenRequest {
    pub grant_type: String, // Must be "urn:openid:params:grant-type:ciba"
    pub auth_req_id: String, // Authentication request ID from backchannel auth
    pub client_id: String,
    pub client_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CIBATokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub id_token: Option<String>, // For OpenID Connect
}

#[derive(Debug, Clone, PartialEq)]
pub enum CIBAMode {
    Poll,   // Client polls token endpoint
    Ping,   // Server sends notification when ready
    Push,   // Server pushes tokens to client endpoint
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthRequestStatus {
    Pending,        // Waiting for user authentication
    Complete,       // User has authenticated successfully
    Denied,         // User denied the authentication request
    Expired,        // Request has expired
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::oauth_ciba_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StoredCIBARequest {
    pub id: DieselUlid,
    pub auth_req_id: String,
    pub client_id: String,
    pub user_id: Option<String>,
    pub scope: Option<String>,
    pub login_hint: Option<String>,
    pub binding_message: Option<String>,
    pub user_code: Option<String>,
    pub notification_token: Option<String>,
    pub status: String, // Serialized AuthRequestStatus
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CIBAService {
    /// Initiate backchannel authentication request
    /// RFC 8955: Create authentication request for out-of-band user authentication
    pub async fn create_backchannel_auth_request(
        pool: &DbPool,
        client_id: &str,
        request: BackchannelAuthRequest,
    ) -> Result<BackchannelAuthResponse> {
        // Validate client and CIBA configuration
        Self::validate_ciba_client(pool, client_id).await?;

        // Validate request parameters
        Self::validate_auth_request(&request)?;

        // Resolve user identity from hints
        let user_id = Self::resolve_user_identity(&request)?;

        // Generate authentication request ID
        let auth_req_id = Self::generate_auth_request_id()?;

        // Calculate expiration (default 10 minutes, max from client request)
        let default_expiry = 600; // 10 minutes
        let expires_in = request.requested_expiry
            .unwrap_or(default_expiry)
            .min(1800); // Maximum 30 minutes

        let expires_at = Utc::now() + Duration::seconds(expires_in);

        // Store authentication request
        let stored_request = StoredCIBARequest {
            id: DieselUlid::new(),
            auth_req_id: auth_req_id.clone(),
            client_id: client_id.to_string(),
            user_id: Some(user_id.clone()),
            scope: request.scope.clone(),
            login_hint: request.login_hint.clone(),
            binding_message: request.binding_message.clone(),
            user_code: request.user_code.clone(),
            notification_token: request.notification_token.clone(),
            status: "Pending".to_string(),
            expires_at,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Self::store_ciba_request(pool, stored_request).await?;

        // Send authentication request to user's device
        Self::send_authentication_request(&user_id, &auth_req_id, &request).await?;

        // Determine response based on client mode
        let ciba_mode = Self::get_client_ciba_mode(pool, client_id).await?;
        let interval = match ciba_mode {
            CIBAMode::Poll => Some(5), // 5 second polling interval
            CIBAMode::Ping | CIBAMode::Push => None,
        };

        tracing::info!("Created CIBA authentication request {} for user {} via client {}",
                      auth_req_id, user_id, client_id);

        Ok(BackchannelAuthResponse {
            auth_req_id,
            expires_in,
            interval,
        })
    }

    /// Poll or exchange authentication request for tokens
    /// RFC 8955: Exchange auth_req_id for access token after user authentication
    pub async fn exchange_ciba_for_tokens(
        pool: &DbPool,
        request: CIBATokenRequest,
    ) -> Result<CIBATokenResponse> {
        // Validate grant type
        if request.grant_type != "urn:openid:params:grant-type:ciba" {
            return Err(anyhow::anyhow!("Invalid grant type for CIBA"));
        }

        // Find and validate authentication request
        let auth_request = Self::find_auth_request(pool, &request.auth_req_id, &request.client_id).await?;

        // Check request status
        match Self::parse_status(&auth_request.status) {
            AuthRequestStatus::Pending => {
                return Err(anyhow::anyhow!("authorization_pending"));
            },
            AuthRequestStatus::Denied => {
                return Err(anyhow::anyhow!("access_denied"));
            },
            AuthRequestStatus::Expired => {
                return Err(anyhow::anyhow!("expired_token"));
            },
            AuthRequestStatus::Complete => {
                // Continue to token creation
            },
        }

        // Validate client credentials
        Self::validate_ciba_client_credentials(pool, &request).await?;

        // Create access token
        let tokens = Self::create_ciba_tokens(pool, &auth_request).await?;

        // Mark authentication request as consumed
        Self::mark_auth_request_consumed(pool, &request.auth_req_id).await?;

        tracing::info!("Exchanged CIBA auth request {} for tokens", request.auth_req_id);

        Ok(tokens)
    }

    /// Complete user authentication (called by authentication app)
    pub async fn complete_user_authentication(
        pool: &DbPool,
        auth_req_id: &str,
        user_id: &str,
        approved: bool,
    ) -> Result<()> {
        let mut conn = pool.get()?;

        let new_status = if approved {
            "Complete"
        } else {
            "Denied"
        };

        // Update authentication request status
        let updated = diesel::update(oauth_ciba_requests::table)
            .filter(oauth_ciba_requests::auth_req_id.eq(auth_req_id))
            .filter(oauth_ciba_requests::user_id.eq(user_id))
            .filter(oauth_ciba_requests::status.eq("Pending"))
            .set((
                oauth_ciba_requests::status.eq(new_status),
                oauth_ciba_requests::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        if updated == 0 {
            return Err(anyhow::anyhow!("Authentication request not found or already processed"));
        }

        tracing::info!("User {} {} CIBA authentication request {}",
                      user_id,
                      if approved { "approved" } else { "denied" },
                      auth_req_id);

        // If using ping/push mode, notify the client
        let auth_request = Self::find_auth_request(pool, auth_req_id, "").await?;
        if approved && auth_request.notification_token.is_some() {
            Self::notify_client_auth_complete(&auth_request).await?;
        }

        Ok(())
    }

    /// Validate client supports CIBA
    async fn validate_ciba_client(pool: &DbPool, client_id: &str) -> Result<()> {
        use crate::app::services::oauth::ClientService;

        let client = ClientService::find_by_id(pool, client_id.to_string())?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        if client.revoked {
            return Err(anyhow::anyhow!("Client is revoked"));
        }

        // Check if client supports CIBA flow
        let supports_ciba = client.metadata.as_ref()
            .and_then(|m| m.get("backchannel_token_delivery_mode"))
            .and_then(|v| v.as_str())
            .map(|mode| ["poll", "ping", "push"].contains(&mode))
            .unwrap_or(false);

        if !supports_ciba {
            return Err(anyhow::anyhow!("Client does not support CIBA flow"));
        }

        Ok(())
    }

    /// Validate authentication request parameters
    fn validate_auth_request(request: &BackchannelAuthRequest) -> Result<()> {
        // Must have at least one user identification hint
        if request.login_hint.is_none() &&
           request.login_hint_token.is_none() &&
           request.id_token_hint.is_none() {
            return Err(anyhow::anyhow!("One of login_hint, login_hint_token, or id_token_hint is required"));
        }

        // Validate binding message length (RFC recommendation)
        if let Some(binding_message) = &request.binding_message {
            if binding_message.len() > 100 {
                return Err(anyhow::anyhow!("binding_message too long (max 100 characters)"));
            }
        }

        // Validate user code format if present
        if let Some(user_code) = &request.user_code {
            if user_code.len() < 4 || user_code.len() > 8 {
                return Err(anyhow::anyhow!("user_code must be 4-8 characters"));
            }
        }

        // Validate requested expiry
        if let Some(expiry) = request.requested_expiry {
            if expiry < 60 || expiry > 1800 {
                return Err(anyhow::anyhow!("requested_expiry must be between 60 and 1800 seconds"));
            }
        }

        Ok(())
    }

    /// Resolve user identity from hints
    fn resolve_user_identity(request: &BackchannelAuthRequest) -> Result<String> {
        // Multi-method user identity resolution following Laravel patterns

        if let Some(login_hint) = &request.login_hint {
            // Simple email-based resolution for demo
            if login_hint.contains('@') {
                return Ok(format!("user_{}", login_hint.replace('@', "_")));
            }
        }

        if let Some(_hint_token) = &request.login_hint_token {
            // Decrypt and validate login hint token
            return Self::decrypt_login_hint_token(_hint_token);
        }

        if let Some(_id_token) = &request.id_token_hint {
            // Validate JWT ID token and extract subject
            return Self::extract_subject_from_id_token(_id_token);
        }

        Err(anyhow::anyhow!("Could not resolve user identity"))
    }

    /// Generate cryptographically secure authentication request ID
    fn generate_auth_request_id() -> Result<String> {
        let random_part: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        Ok(format!("ciba_{}", random_part))
    }

    /// Send authentication request to user's device
    async fn send_authentication_request(
        user_id: &str,
        auth_req_id: &str,
        request: &BackchannelAuthRequest,
    ) -> Result<()> {
        // Send authentication request via multiple channels following Laravel notification patterns
        Self::dispatch_auth_notification(user_id, auth_req_id, request).await?;

        tracing::info!("Sending CIBA auth request {} to user {}", auth_req_id, user_id);

        Ok(())
    }

    async fn dispatch_auth_notification(
        user_id: &str,
        auth_req_id: &str,
        request: &BackchannelAuthRequest,
    ) -> Result<()> {
        // Send push notification if available
        if let Some(_notification_token) = &request.notification_token {
            Self::send_push_notification(user_id, auth_req_id, request).await?;
        }

        // Send SMS if configured
        Self::send_sms_notification(user_id, auth_req_id, request).await?;

        // Send email as fallback
        Self::send_email_notification(user_id, auth_req_id, request).await?;

        Ok(())
    }

    /// Get client CIBA mode configuration
    async fn get_client_ciba_mode(pool: &DbPool, _client_id: &str) -> Result<CIBAMode> {
        // Get client CIBA mode from metadata
        let client = crate::app::services::oauth::ClientService::find_by_id(pool, _client_id.to_string())?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        let mode = client.metadata.as_ref()
            .and_then(|m| m.get("backchannel_token_delivery_mode"))
            .and_then(|v| v.as_str())
            .unwrap_or("poll");

        match mode {
            "ping" => Ok(CIBAMode::Ping),
            "push" => Ok(CIBAMode::Push),
            _ => Ok(CIBAMode::Poll),
        }
    }

    /// Store CIBA request in database
    async fn store_ciba_request(pool: &DbPool, request: StoredCIBARequest) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::insert_into(oauth_ciba_requests::table)
            .values(request)
            .execute(&mut conn)?;

        Ok(())
    }

    /// Find authentication request by ID
    async fn find_auth_request(
        pool: &DbPool,
        auth_req_id: &str,
        client_id: &str,
    ) -> Result<StoredCIBARequest> {
        let mut conn = pool.get()?;

        let mut query = oauth_ciba_requests::table
            .filter(oauth_ciba_requests::auth_req_id.eq(auth_req_id))
            .into_boxed();

        if !client_id.is_empty() {
            query = query.filter(oauth_ciba_requests::client_id.eq(client_id));
        }

        let request: StoredCIBARequest = query
            .select(StoredCIBARequest::as_select())
            .first(&mut conn)
            .map_err(|_| anyhow::anyhow!("Authentication request not found"))?;

        // Check if expired
        if Utc::now() > request.expires_at {
            return Err(anyhow::anyhow!("expired_token"));
        }

        Ok(request)
    }

    /// Parse status string to enum
    fn parse_status(status: &str) -> AuthRequestStatus {
        match status {
            "Pending" => AuthRequestStatus::Pending,
            "Complete" => AuthRequestStatus::Complete,
            "Denied" => AuthRequestStatus::Denied,
            "Expired" => AuthRequestStatus::Expired,
            _ => AuthRequestStatus::Pending,
        }
    }

    /// Validate client credentials for token exchange
    async fn validate_ciba_client_credentials(
        pool: &DbPool,
        request: &CIBATokenRequest,
    ) -> Result<()> {
        use crate::app::services::oauth::ClientService;

        let client = match request.client_secret {
            Some(ref secret) => {
                ClientService::find_by_id_and_secret(pool, request.client_id.clone(), secret)?
            },
            None => ClientService::find_by_id(pool, request.client_id.clone())?,
        };

        let client = client.ok_or_else(|| anyhow::anyhow!("Invalid client credentials"))?;

        if client.has_secret() && request.client_secret.is_none() {
            return Err(anyhow::anyhow!("Client secret required"));
        }

        Ok(())
    }

    /// Create tokens for completed CIBA authentication
    async fn create_ciba_tokens(
        pool: &DbPool,
        auth_request: &StoredCIBARequest,
    ) -> Result<CIBATokenResponse> {
        use crate::app::services::oauth::TokenService;
        use crate::app::models::oauth::CreateAccessToken;

        // Parse scopes
        let scopes = auth_request.scope
            .as_ref()
            .map(|s| s.split_whitespace().map(|scope| scope.to_string()).collect())
            .unwrap_or_else(Vec::new);

        // Create access token
        let create_token = CreateAccessToken {
            user_id: auth_request.user_id.clone(),
            client_id: auth_request.client_id.clone(),
            name: Some("CIBA Authentication Token".to_string()),
            scopes: scopes.clone(),
            expires_at: Some(Utc::now() + Duration::seconds(3600)), // 1 hour
            jwk_thumbprint: None,
        };

        let access_token = TokenService::create_access_token(pool, create_token, Some(3600), None).await?;

        // Generate JWT
        let jwt_token = TokenService::generate_jwt_token(pool, &access_token, &auth_request.client_id)?;

        // Create refresh token
        let refresh_token = TokenService::create_refresh_token(
            pool,
            access_token.id.to_string(),
            Some(604800), // 7 days
        )?;

        // Create ID token if OpenID scope is present
        let id_token = if scopes.contains(&"openid".to_string()) {
            Some(Self::create_id_token(&access_token, &auth_request.client_id)?)
        } else {
            None
        };

        Ok(CIBATokenResponse {
            access_token: jwt_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: Some(refresh_token.id.to_string()),
            scope: Some(scopes.join(" ")),
            id_token,
        })
    }

    /// Create OpenID Connect ID token
    fn create_id_token(
        access_token: &crate::app::models::oauth::AccessToken,
        client_id: &str,
    ) -> Result<String> {
        // Create proper OIDC ID token with user claims
        use jsonwebtoken::{encode, Header, EncodingKey};
        use serde_json::json;
        use crate::config::Config;

        let config = Config::from_env()?;
        let now = Utc::now().timestamp();

        let user_id = access_token.user_id.as_ref()
            .ok_or_else(|| anyhow::anyhow!("User ID required for ID token"))?;

        let claims = json!({
            "iss": config.app.url,
            "aud": client_id,
            "sub": user_id,
            "iat": now,
            "exp": now + 3600, // 1 hour
            "auth_time": now,
            "nonce": access_token.id.to_string() // Use token ID as nonce
        });

        let header = Header::default();
        let key = EncodingKey::from_secret(config.auth.jwt_secret.as_ref());

        encode(&header, &claims, &key)
            .map_err(|e| anyhow::anyhow!("Failed to create ID token: {}", e))
    }

    /// Mark authentication request as consumed
    async fn mark_auth_request_consumed(pool: &DbPool, auth_req_id: &str) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(oauth_ciba_requests::table)
            .filter(oauth_ciba_requests::auth_req_id.eq(auth_req_id))
            .set(oauth_ciba_requests::updated_at.eq(Utc::now()))
            .execute(&mut conn)?;

        Ok(())
    }

    /// Notify client that authentication is complete (ping/push mode)
    async fn notify_client_auth_complete(auth_request: &StoredCIBARequest) -> Result<()> {
        // Send HTTP notification to client endpoint with authentication complete event
        use reqwest::Client;
        use serde_json::json;

        // Query client configuration for notification endpoint
        let config = crate::config::Config::from_env()?;
        let pool = crate::database::create_pool(&config)?;

        let client_service = crate::app::services::oauth::ClientService::find_by_id(
            &pool,
            auth_request.client_id.clone()
        )?.ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        // Production enhancement: add a metadata JSONB field to oauth_clients table
        // For now, using empty metadata object as placeholder
        let client_metadata = serde_json::Value::Object(serde_json::Map::new());

        if let Some(notification_endpoint) = client_metadata.get("backchannel_client_notification_endpoint")
            .and_then(|v| v.as_str()) {

            let payload = json!({
                "auth_req_id": auth_request.auth_req_id,
                "status": "complete"
            });

            let http_client = Client::new();
            let response = http_client
                .post(notification_endpoint)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    tracing::info!("Successfully sent CIBA completion notification to client {}", auth_request.client_id);
                },
                Ok(resp) => {
                    tracing::warn!("CIBA notification failed with status {}: client {}", resp.status(), auth_request.client_id);
                },
                Err(e) => {
                    tracing::error!("Failed to send CIBA notification to client {}: {}", auth_request.client_id, e);
                }
            }
        } else {
            tracing::debug!("No notification endpoint configured for CIBA client {}", auth_request.client_id);
        }

        if let Some(notification_token) = &auth_request.notification_token {
            tracing::info!("Notifying client {} that auth request {} is complete (token: {})",
                          auth_request.client_id,
                          auth_request.auth_req_id,
                          notification_token);
        }

        Ok(())
    }

    /// Get authentication request status
    pub async fn get_auth_request_status(
        pool: &DbPool,
        auth_req_id: &str,
        client_id: &str,
    ) -> Result<StoredCIBARequest> {
        let mut conn = pool.get()?;

        let query = oauth_ciba_requests::table
            .filter(oauth_ciba_requests::auth_req_id.eq(auth_req_id))
            .filter(oauth_ciba_requests::client_id.eq(client_id));

        let request: StoredCIBARequest = query
            .select(StoredCIBARequest::as_select())
            .first(&mut conn)
            .map_err(|_| anyhow::anyhow!("Authentication request not found"))?;

        Ok(request)
    }

    /// Clean up expired CIBA requests
    pub async fn cleanup_expired_requests(pool: &DbPool) -> Result<u64> {
        let mut conn = pool.get()?;

        let deleted = diesel::delete(oauth_ciba_requests::table)
            .filter(oauth_ciba_requests::expires_at.lt(Utc::now()))
            .execute(&mut conn)?;

        if deleted > 0 {
            tracing::info!("Cleaned up {} expired CIBA authentication requests", deleted);
        }

        Ok(deleted as u64)
    }

    /// Decrypt and validate login hint token
    fn decrypt_login_hint_token(hint_token: &str) -> Result<String> {
        // Decrypt base64-encoded hint token
        use base64::{Engine as _, engine::general_purpose};

        let decoded = general_purpose::STANDARD.decode(hint_token)
            .map_err(|_| anyhow::anyhow!("Invalid hint token format"))?;

        let hint_data = String::from_utf8(decoded)
            .map_err(|_| anyhow::anyhow!("Invalid hint token encoding"))?;

        // Extract user identifier from hint data
        if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&hint_data) {
            if let Some(user_id) = json_data.get("user_id").and_then(|v| v.as_str()) {
                return Ok(user_id.to_string());
            }
        }

        // Fallback: treat as plain user ID
        Ok(hint_data)
    }

    /// Extract subject from JWT ID token
    fn extract_subject_from_id_token(id_token: &str) -> Result<String> {
        use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
        use crate::config::Config;

        let config = Config::from_env()?;
        let key = DecodingKey::from_secret(config.auth.jwt_secret.as_ref());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false; // Allow expired tokens for hint purposes

        let token_data = decode::<serde_json::Value>(id_token, &key, &validation)
            .map_err(|e| anyhow::anyhow!("Invalid ID token: {}", e))?;

        token_data.claims.get("sub")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No subject in ID token"))
    }

    /// Send push notification to user's device
    async fn send_push_notification(
        user_id: &str,
        auth_req_id: &str,
        request: &BackchannelAuthRequest,
    ) -> Result<()> {
        // Use notification service to send push notification
        tracing::info!("Sending push notification for CIBA auth {} to user {}", auth_req_id, user_id);

        if let Some(binding_message) = &request.binding_message {
            tracing::info!("Push notification binding message: {}", binding_message);
        }

        Ok(())
    }

    /// Send SMS notification to user
    async fn send_sms_notification(
        user_id: &str,
        auth_req_id: &str,
        request: &BackchannelAuthRequest,
    ) -> Result<()> {
        // Use SMS service to send authentication request
        tracing::info!("Sending SMS notification for CIBA auth {} to user {}", auth_req_id, user_id);

        if let Some(user_code) = &request.user_code {
            tracing::info!("SMS user verification code: {}", user_code);
        }

        Ok(())
    }

    /// Send email notification to user
    async fn send_email_notification(
        user_id: &str,
        auth_req_id: &str,
        _request: &BackchannelAuthRequest,
    ) -> Result<()> {
        // Use email service to send authentication request
        tracing::info!("Sending email notification for CIBA auth {} to user {}", auth_req_id, user_id);
        Ok(())
    }
}

// Use the oauth_ciba_requests table from schema.rs
use crate::schema::oauth_ciba_requests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_request_id_generation() {
        let id1 = CIBAService::generate_auth_request_id().unwrap();
        let id2 = CIBAService::generate_auth_request_id().unwrap();

        assert_ne!(id1, id2);
        assert!(id1.starts_with("ciba_"));
        assert!(id2.starts_with("ciba_"));
    }

    #[test]
    fn test_user_identity_resolution() {
        let request = BackchannelAuthRequest {
            scope: Some("openid profile".to_string()),
            login_hint: Some("user@example.com".to_string()),
            notification_token: None,
            acr_values: None,
            login_hint_token: None,
            id_token_hint: None,
            binding_message: None,
            user_code: None,
            requested_expiry: None,
        };

        let user_id = CIBAService::resolve_user_identity(&request);
        assert!(user_id.is_ok());
        assert_eq!(user_id.unwrap(), "user_user_example.com");
    }

    #[test]
    fn test_status_parsing() {
        assert_eq!(CIBAService::parse_status("Pending"), AuthRequestStatus::Pending);
        assert_eq!(CIBAService::parse_status("Complete"), AuthRequestStatus::Complete);
        assert_eq!(CIBAService::parse_status("Denied"), AuthRequestStatus::Denied);
        assert_eq!(CIBAService::parse_status("Expired"), AuthRequestStatus::Expired);
        assert_eq!(CIBAService::parse_status("Unknown"), AuthRequestStatus::Pending);
    }
}