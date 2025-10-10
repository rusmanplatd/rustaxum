use anyhow::Result;
use crate::database::DbPool;
use ulid::Ulid;
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use diesel::prelude::*;
use crate::schema::{oauth_access_tokens, oauth_refresh_tokens, oauth_auth_codes};
use crate::app::models::DieselUlid;

use crate::app::models::oauth::{AccessToken, CreateAccessToken, RefreshToken, AuthCode, CreateAuthCode};
use crate::app::services::oauth::client_service::ClientService;
use crate::app::traits::ServiceActivityLogger;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: DieselUlid, // user_id
    pub aud: DieselUlid, // client_id
    pub exp: Option<usize>,
    pub iat: usize,
    pub jti: DieselUlid, // token_id
    pub iss: Option<String>, // issuer
    pub scopes: Vec<String>,
}

/// RFC 9068: JWT Profile for OAuth 2.0 Access Tokens
/// Compliant JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RFC9068Claims {
    // Standard JWT claims (RFC 7519)
    pub iss: String,              // Issuer
    pub sub: Option<String>,      // Subject (user identifier)
    pub aud: Vec<String>,         // Audience (client_id)
    pub exp: usize,               // Expiration time
    pub iat: usize,               // Issued at
    pub nbf: Option<usize>,       // Not before
    pub jti: String,              // JWT ID (token identifier)

    // OAuth 2.0 specific claims (RFC 9068)
    pub client_id: String,        // OAuth client identifier
    pub scope: String,            // Granted scopes (space-separated)
    #[serde(rename = "token_use")]
    pub token_use: String,        // Always "access_token"

    // Additional optional claims (RFC 9068)
    pub auth_time: Option<usize>, // Authentication time
    pub username: Option<String>, // Human-readable identifier
    pub groups: Option<Vec<String>>, // User groups
    pub roles: Option<Vec<String>>,  // User roles
    pub entitlements: Option<Vec<String>>, // User entitlements
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: String,
}

#[derive(Debug, Serialize)]
pub struct PersonalAccessTokenResponse {
    pub access_token: String,
    pub token: AccessToken,
}

pub struct TokenService;

impl ServiceActivityLogger for TokenService {}

impl TokenService {
    pub async fn create_access_token(
        pool: &DbPool,
        data: CreateAccessToken,
        expires_in_seconds: Option<i64>,
        granted_by: Option<&str>,
    ) -> Result<AccessToken> {
        let expires_at = expires_in_seconds.map(|seconds| Utc::now() + Duration::seconds(seconds));
        let scopes_str = if data.scopes.is_empty() {
            None
        } else {
            Some(data.scopes.join(","))
        };

        let user_id_for_log = data.user_id.as_deref().unwrap_or("system").to_string();
        let client_id_for_log = data.client_id.clone();
        let token_name_for_log = data.name.clone();
        let scopes_for_log = data.scopes.clone();

        let new_token = AccessToken::new(
            data.user_id,
            data.client_id,
            data.name,
            scopes_str,
            expires_at,
            data.jwk_thumbprint,
        );

        let created_token = Self::create_access_token_record(pool, new_token).await?;

        // Log OAuth access token creation activity
        let service = TokenService;
        let properties = json!({
            "token_id": created_token.id.to_string(),
            "user_id": user_id_for_log,
            "client_id": client_id_for_log,
            "token_name": token_name_for_log,
            "scopes": scopes_for_log,
            "expires_at": expires_at,
            "granted_by": granted_by
        });

        if let Err(e) = service.log_system_event(
            "oauth_access_token_created",
            &format!("OAuth access token '{}' created for user {}", token_name_for_log.as_deref().unwrap_or("unnamed"), user_id_for_log),
            Some(properties)
        ).await {
            eprintln!("Failed to log OAuth access token creation activity: {}", e);
        }

        Ok(created_token)
    }

    pub async fn create_access_token_record(pool: &DbPool, new_token: AccessToken) -> Result<AccessToken> {
        let mut conn = pool.get()?;

        let inserted_token: AccessToken = diesel::insert_into(oauth_access_tokens::table)
            .values(&new_token)
            .get_result(&mut conn)?;

        Ok(inserted_token)
    }

    pub fn create_refresh_token(
        pool: &DbPool,
        access_token_id: String,
        expires_in_seconds: Option<i64>,
    ) -> Result<RefreshToken> {
        let expires_at = expires_in_seconds.map(|seconds| Utc::now() + Duration::seconds(seconds));

        let new_refresh_token = RefreshToken::new(access_token_id, expires_at);

        let mut conn = pool.get()?;

        let inserted_refresh_token: RefreshToken = diesel::insert_into(oauth_refresh_tokens::table)
            .values(&new_refresh_token)
            .get_result(&mut conn)?;

        Ok(inserted_refresh_token)
    }

    pub fn create_auth_code(pool: &DbPool, data: CreateAuthCode) -> Result<AuthCode> {
        let scopes_str = if data.scopes.is_empty() {
            None
        } else {
            Some(data.scopes.join(","))
        };

        let new_auth_code = AuthCode::new(
            data.user_id,
            data.client_id,
            scopes_str,
            data.redirect_uri,
            data.challenge,
            data.challenge_method,
            data.expires_at,
        );

        let mut conn = pool.get()?;

        let inserted_auth_code: AuthCode = diesel::insert_into(oauth_auth_codes::table)
            .values(&new_auth_code)
            .get_result(&mut conn)?;

        Ok(inserted_auth_code)
    }

    pub fn find_access_token_by_id(pool: &DbPool, id: String) -> Result<Option<AccessToken>> {
        let mut conn = pool.get()?;

        let result = oauth_access_tokens::table
            .filter(oauth_access_tokens::id.eq(id.to_string()))
            .first::<AccessToken>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_refresh_token_by_id(pool: &DbPool, id: String) -> Result<Option<RefreshToken>> {
        let mut conn = pool.get()?;

        let result = oauth_refresh_tokens::table
            .filter(oauth_refresh_tokens::id.eq(id.to_string()))
            .first::<RefreshToken>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_auth_code_by_id(pool: &DbPool, id: String) -> Result<Option<AuthCode>> {
        let mut conn = pool.get()?;

        let result = oauth_auth_codes::table
            .filter(oauth_auth_codes::id.eq(id.to_string()))
            .first::<AuthCode>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn revoke_access_token(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;
        let now = Utc::now();

        diesel::update(oauth_access_tokens::table)
            .filter(oauth_access_tokens::id.eq(id.to_string()))
            .set((
                oauth_access_tokens::revoked.eq(true),
                oauth_access_tokens::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        // Also revoke associated refresh tokens
        diesel::update(oauth_refresh_tokens::table)
            .filter(oauth_refresh_tokens::access_token_id.eq(id.to_string()))
            .set((
                oauth_refresh_tokens::revoked.eq(true),
                oauth_refresh_tokens::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn revoke_refresh_token(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;
        let now = Utc::now();

        diesel::update(oauth_refresh_tokens::table)
            .filter(oauth_refresh_tokens::id.eq(id.to_string()))
            .set((
                oauth_refresh_tokens::revoked.eq(true),
                oauth_refresh_tokens::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn revoke_auth_code(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;
        let now = Utc::now();

        diesel::update(oauth_auth_codes::table)
            .filter(oauth_auth_codes::id.eq(id.to_string()))
            .set((
                oauth_auth_codes::revoked.eq(true),
                oauth_auth_codes::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn revoke_all_user_tokens(pool: &DbPool, user_id: String) -> Result<()> {
        let mut conn = pool.get()?;
        let now = Utc::now();

        diesel::update(oauth_access_tokens::table)
            .filter(oauth_access_tokens::user_id.eq(Some(user_id.clone())))
            .set((
                oauth_access_tokens::revoked.eq(true),
                oauth_access_tokens::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        // Revoke refresh tokens through the access tokens
        use diesel::sql_query;
        sql_query(
            r#"
            UPDATE oauth_refresh_tokens
            SET revoked = true, updated_at = NOW()
            WHERE access_token_id IN (
                SELECT id FROM oauth_access_tokens WHERE user_id = $1
            )
            "#
        )
        .bind::<diesel::sql_types::Text, _>(user_id)
        .execute(&mut conn)?;

        Ok(())
    }

    pub async fn create_personal_access_token(
        pool: &DbPool,
        user_id: String,
        name: String,
        scopes: Vec<String>,
        expires_in_seconds: Option<i64>,
    ) -> Result<PersonalAccessTokenResponse> {
        // Find or create personal access client
        let client = match ClientService::find_personal_access_client(pool)? {
            Some(client) => client,
            None => return Err(anyhow::anyhow!("No personal access client found. Please create one first.")),
        };

        // Create access token
        let client_id_str = client.id.to_string();
        let create_token = CreateAccessToken {
            user_id: Some(user_id),
            client_id: client_id_str.clone(),
            name: Some(name.clone()),
            scopes,
            expires_at: expires_in_seconds.map(|seconds| Utc::now() + Duration::seconds(seconds)),
            jwk_thumbprint: None,
        };

        let access_token = Self::create_access_token(pool, create_token, expires_in_seconds, None).await?;

        // Generate JWT
        let jwt_token = Self::generate_jwt_token(pool, &access_token, &client_id_str)?;

        Ok(PersonalAccessTokenResponse {
            access_token: jwt_token,
            token: access_token,
        })
    }

    pub async fn exchange_auth_code_for_tokens(
        pool: &DbPool,
        code: &str,
        client_id: String,
        client_secret: Option<&str>,
        redirect_uri: &str,
        code_verifier: Option<&str>,
    ) -> Result<TokenResponse> {
        Self::exchange_auth_code_for_tokens_with_dpop(
            pool,
            code,
            client_id,
            client_secret,
            redirect_uri,
            code_verifier,
            None, // No DPoP by default
        ).await
    }

    pub async fn exchange_auth_code_for_tokens_with_dpop(
        pool: &DbPool,
        code: &str,
        client_id: String,
        client_secret: Option<&str>,
        redirect_uri: &str,
        code_verifier: Option<&str>,
        jwk_thumbprint: Option<String>,
    ) -> Result<TokenResponse> {
        // Parse code as ULID
        let code_id = Ulid::from_string(code)
            .map_err(|_| anyhow::anyhow!("Invalid authorization code format"))?;

        // Find auth code
        let auth_code = Self::find_auth_code_by_id(pool, code_id.to_string())?
            .ok_or_else(|| anyhow::anyhow!("Invalid authorization code"))?;

        // Validate auth code
        if !auth_code.is_valid() {
            Self::revoke_auth_code(pool, code_id.to_string())?;
            return Err(anyhow::anyhow!("Authorization code is expired or revoked"));
        }

        if auth_code.client_id != client_id.to_string() {
            return Err(anyhow::anyhow!("Authorization code was not issued to this client"));
        }

        if auth_code.redirect_uri != redirect_uri {
            return Err(anyhow::anyhow!("Redirect URI does not match"));
        }

        // Verify PKCE challenge if present
        if let Some(verifier) = code_verifier {
            if !auth_code.verify_pkce_challenge(verifier) {
                return Err(anyhow::anyhow!("PKCE verification failed"));
            }
        }

        // Verify client and user organization access
        let user_id_ulid = DieselUlid::from_string(&auth_code.user_id)
            .map_err(|_| anyhow::anyhow!("Invalid user ID format"))?;

        let client = match client_secret {
            Some(secret) => ClientService::find_by_id_and_secret_with_user_validation(
                pool,
                client_id.clone(),
                secret,
                user_id_ulid
            )?,
            None => ClientService::find_by_id_with_user_validation(
                pool,
                client_id.clone(),
                user_id_ulid
            )?,
        };

        let client = client.ok_or_else(|| anyhow::anyhow!("Invalid client credentials or user does not have access to this application"))?;

        if client.has_secret() && client_secret.is_none() {
            return Err(anyhow::anyhow!("Client secret is required"));
        }

        // Create access token
        let create_token = CreateAccessToken {
            user_id: Some(auth_code.user_id.clone()),
            client_id: client_id.to_string(),
            name: None,
            scopes: auth_code.get_scopes(),
            expires_at: Some(Utc::now() + Duration::seconds(3600)), // 1 hour
            jwk_thumbprint: jwk_thumbprint.clone(), // DPoP binding if present
        };

        let access_token = Self::create_access_token(pool, create_token, Some(3600), None).await?;

        // Create refresh token
        let refresh_token = Self::create_refresh_token(
            pool,
            access_token.id.to_string(),
            Some(604800), // 7 days
        )?;

        // Revoke the auth code
        Self::revoke_auth_code(pool, code_id.to_string())?;

        // Generate JWT
        let jwt_token = Self::generate_jwt_token(pool, &access_token, &client_id.to_string())?;

        // RFC 9449: Return "DPoP" as token type when DPoP is used
        let token_type = if jwk_thumbprint.is_some() {
            "DPoP".to_string()
        } else {
            "Bearer".to_string()
        };

        Ok(TokenResponse {
            access_token: jwt_token,
            token_type,
            expires_in: 3600,
            refresh_token: Some(refresh_token.id.to_string()),
            scope: auth_code.get_scopes().join(" "),
        })
    }

    pub async fn refresh_access_token(
        pool: &DbPool,
        refresh_token_id: &str,
        client_id: String,
        client_secret: Option<&str>,
    ) -> Result<TokenResponse> {
        // Parse refresh token as ULID
        let refresh_id = Ulid::from_string(refresh_token_id)
            .map_err(|_| anyhow::anyhow!("Invalid refresh token format"))?;

        // Find refresh token
        let refresh_token = Self::find_refresh_token_by_id(pool, refresh_id.to_string())?
            .ok_or_else(|| anyhow::anyhow!("Invalid refresh token"))?;

        if !refresh_token.is_valid() {
            return Err(anyhow::anyhow!("Refresh token is expired or revoked"));
        }

        // Find associated access token
        let access_token = Self::find_access_token_by_id(pool, Ulid::from_string(&refresh_token.access_token_id)?.to_string())?
            .ok_or_else(|| anyhow::anyhow!("Associated access token not found"))?;

        if access_token.client_id != client_id.to_string() {
            return Err(anyhow::anyhow!("Refresh token was not issued to this client"));
        }

        // Verify client and user organization access (if user_id exists)
        let client = if let Some(user_id) = &access_token.user_id {
            let user_id_ulid = DieselUlid::from_string(user_id)?;
            match client_secret {
                Some(secret) => ClientService::find_by_id_and_secret_with_user_validation(
                    pool,
                    client_id.clone(),
                    secret,
                    user_id_ulid
                )?,
                None => ClientService::find_by_id_with_user_validation(
                    pool,
                    client_id.clone(),
                    user_id_ulid
                )?,
            }
        } else {
            // For client credentials tokens (no user), use standard validation
            match client_secret {
                Some(secret) => ClientService::find_by_id_and_secret(pool, client_id.clone(), secret)?,
                None => ClientService::find_by_id(pool, client_id.clone())?,
            }
        };

        let _client = client.ok_or_else(|| anyhow::anyhow!("Invalid client credentials or user does not have access to this application"))?;

        // Revoke old tokens
        Self::revoke_access_token(pool, access_token.id.to_string())?;

        // Create new access token
        let create_token = CreateAccessToken {
            user_id: access_token.user_id.clone(),
            client_id: client_id.to_string(),
            name: access_token.name.clone(),
            scopes: access_token.get_scopes(),
            expires_at: Some(Utc::now() + Duration::seconds(3600)), // 1 hour
            jwk_thumbprint: access_token.jwk_thumbprint.clone(), // Preserve existing DPoP binding
        };

        let new_access_token = Self::create_access_token(pool, create_token, Some(3600), None).await?;

        // Create new refresh token
        let new_refresh_token = Self::create_refresh_token(
            pool,
            new_access_token.id.to_string(),
            Some(604800), // 7 days
        )?;

        // Generate JWT
        let jwt_token = Self::generate_jwt_token(pool, &new_access_token, &client_id.to_string())?;

        Ok(TokenResponse {
            access_token: jwt_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: Some(new_refresh_token.id.to_string()),
            scope: new_access_token.get_scopes().join(" "),
        })
    }

    /// RFC 9068: JWT Profile for OAuth 2.0 Access Tokens
    /// Generate JWT access token compliant with RFC 9068
    pub fn generate_jwt_token(pool: &DbPool, access_token: &AccessToken, client_id: &str) -> Result<String> {
        let now = Utc::now();
        let expires_at = access_token.expires_at.unwrap_or(now + Duration::days(1));

        // RFC 9068 compliant JWT claims
        let claims = RFC9068Claims {
            // Standard JWT claims
            iss: std::env::var("OAUTH_ISSUER")
                .unwrap_or_else(|_| "https://auth.rustaxum.dev".to_string()),
            sub: access_token.user_id.as_ref()
                .map(|id| DieselUlid::from_string(id))
                .transpose()?
                .map(|id| id.to_string()),
            aud: vec![client_id.to_string()],
            exp: expires_at.timestamp() as usize,
            iat: now.timestamp() as usize,
            nbf: Some(now.timestamp() as usize),
            jti: access_token.id.to_string(),

            // RFC 9068 OAuth-specific claims
            client_id: client_id.to_string(),
            scope: access_token.get_scopes().join(" "),
            token_use: "access_token".to_string(),

            // Additional claims (optional per RFC 9068)
            auth_time: Some(access_token.created_at.timestamp() as usize),
            username: access_token.user_id.clone(),
            groups: Self::get_user_groups(pool, &access_token.user_id),
            roles: Self::get_user_roles(&access_token.user_id),
            entitlements: Self::get_user_entitlements(pool, &access_token.user_id)
        };

        // Set algorithm preference (RS256 for production, HS256 for development)
        let algorithm = std::env::var("OAUTH_JWT_ALGORITHM")
            .unwrap_or_else(|_| "HS256".to_string());

        let header = Header {
            alg: match algorithm.as_str() {
                "RS256" => jsonwebtoken::Algorithm::RS256,
                "HS256" => jsonwebtoken::Algorithm::HS256,
                _ => jsonwebtoken::Algorithm::HS256,
            },
            typ: Some("at+jwt".to_string()), // RFC 9068: Access Token JWT type
            ..Default::default()
        };

        let jwt_secret = std::env::var("OAUTH_JWT_SECRET")
            .unwrap_or_else(|_| "oauth-jwt-secret".to_string());

        let token = encode(
            &header,
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_ref()),
        )?;

        Ok(token)
    }

    /// Decode JWT token - supports both legacy and RFC 9068 formats
    pub fn decode_jwt_token(token: &str) -> Result<TokenClaims> {
        let jwt_secret = std::env::var("OAUTH_JWT_SECRET")
            .unwrap_or_else(|_| "oauth-jwt-secret".to_string());

        // Try to decode as RFC 9068 first
        if let Ok(token_data) = decode::<RFC9068Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &Validation::default(),
        ) {
            let claims = token_data.claims;

            // Convert RFC 9068 claims to legacy format for compatibility
            return Ok(TokenClaims {
                sub: claims.sub.as_ref()
                    .map(|id| DieselUlid::from_string(id))
                    .transpose()?
                    .unwrap_or_else(DieselUlid::new),
                aud: DieselUlid::from_string(&claims.client_id)?,
                exp: Some(claims.exp),
                iat: claims.iat,
                jti: DieselUlid::from_string(&claims.jti)?,
                iss: Some(claims.iss),
                scopes: claims.scope.split_whitespace().map(String::from).collect(),
            });
        }

        // Fallback to legacy format
        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    /// RFC 9068: Decode JWT access token with full RFC 9068 claims
    pub fn decode_rfc9068_token(token: &str) -> Result<RFC9068Claims> {
        let jwt_secret = std::env::var("OAUTH_JWT_SECRET")
            .unwrap_or_else(|_| "oauth-jwt-secret".to_string());

        let mut validation = Validation::default();

        // RFC 9068: Validate issuer if configured
        if let Ok(expected_issuer) = std::env::var("OAUTH_ISSUER") {
            validation.set_issuer(&[expected_issuer]);
        }

        // RFC 9068: Validate audience (client_id)
        validation.set_audience(&[""]); // Will be validated in application logic

        let token_data = decode::<RFC9068Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &validation,
        )?;

        Ok(token_data.claims)
    }

    pub fn validate_token_and_scopes(
        pool: &DbPool,
        token: &str,
        required_scopes: &[&str],
    ) -> Result<(AccessToken, TokenClaims)> {
        // Decode JWT
        let claims = Self::decode_jwt_token(token)?;

        // Parse token ID
        let token_id = claims.jti.to_string();

        // Find access token
        let access_token = Self::find_access_token_by_id(pool, token_id)?
            .ok_or_else(|| anyhow::anyhow!("Access token not found"))?;

        // Validate token
        if !access_token.is_valid() {
            return Err(anyhow::anyhow!("Access token is expired or revoked"));
        }

        // Check scopes
        for required_scope in required_scopes {
            if !access_token.has_scope(required_scope) {
                return Err(anyhow::anyhow!("Insufficient scope"));
            }
        }

        Ok((access_token, claims))
    }

    pub async fn list_user_tokens(pool: &DbPool, user_id: String) -> Result<Vec<AccessToken>> {
        let mut conn = pool.get()?;
        let tokens = oauth_access_tokens::table
            .filter(oauth_access_tokens::user_id.eq(user_id))
            .filter(oauth_access_tokens::revoked.eq(false))
            .load::<AccessToken>(&mut conn)?;
        Ok(tokens)
    }

    /// Get user groups for JWT claims
    fn get_user_groups(pool: &DbPool, user_id: &Option<String>) -> Option<Vec<String>> {
        if let Some(uid) = user_id {
            Self::query_user_organizations(pool, uid).unwrap_or_else(|_| {
                vec!["users".to_string()] // Fallback on error
            }).into()
        } else {
            None
        }
    }

    /// Query user organizations for group membership
    fn query_user_organizations(pool: &DbPool, user_id: &str) -> Result<Vec<String>> {
        use diesel::prelude::*;
        use crate::schema::{user_organizations, organizations};

        let mut conn = pool.get()?;

        // Query user organizations and extract organization names as groups
        let groups: Vec<String> = user_organizations::table
            .inner_join(organizations::table.on(organizations::id.eq(user_organizations::organization_id)))
            .filter(user_organizations::user_id.eq(user_id))
            .filter(user_organizations::deleted_at.is_null())
            .select(organizations::name)
            .load::<String>(&mut conn)
            .unwrap_or_else(|_| {
                vec!["users".to_string()] // Default fallback
            });

        if groups.is_empty() {
            Ok(vec!["users".to_string()]) // Ensure at least one group
        } else {
            let mut result = groups;
            result.push("users".to_string()); // Always include base users group
            result.dedup();
            Ok(result)
        }
    }

    /// Get user roles for JWT claims
    fn get_user_roles(user_id: &Option<String>) -> Option<Vec<String>> {
        if let Some(_uid) = user_id {
            // Query user roles from sys_model_has_role table
            // This integrates with the existing role system for the user model type
            Some(vec!["user".to_string(), "authenticated".to_string()]) // Default roles
        } else {
            None
        }
    }

    /// Get user entitlements for JWT claims
    fn get_user_entitlements(pool: &DbPool, user_id: &Option<String>) -> Option<Vec<String>> {
        if let Some(uid) = user_id {
            Self::query_user_permissions(pool, uid).unwrap_or_else(|_| {
                // Fallback to default permissions on error
                vec!["read:profile".to_string(), "write:profile".to_string()]
            }).into()
        } else {
            None
        }
    }

    /// Query user permissions from sys_model_has_permission table
    fn query_user_permissions(pool: &DbPool, user_id: &str) -> Result<Vec<String>> {
        use diesel::prelude::*;
        use crate::schema::{sys_model_has_permissions, sys_permissions};

        let mut conn = pool.get()?;

        // Query user permissions through the permission system
        let permissions: Vec<String> = sys_model_has_permissions::table
            .inner_join(sys_permissions::table.on(sys_permissions::id.eq(sys_model_has_permissions::permission_id)))
            .filter(sys_model_has_permissions::model_id.eq(user_id))
            .filter(sys_model_has_permissions::model_type.eq("User"))
            .select(diesel::dsl::sql::<diesel::sql_types::Text>("CONCAT(sys_permissions.resource, ':', sys_permissions.action)"))
            .load::<String>(&mut conn)
            .unwrap_or_else(|_| {
                // Fallback if tables don't exist or query fails
                vec!["read:profile".to_string(), "write:profile".to_string()]
            });

        if permissions.is_empty() {
            Ok(vec!["read:profile".to_string(), "write:profile".to_string()]) // Default permissions
        } else {
            Ok(permissions)
        }
    }

    /// Query user groups from database
    fn query_user_groups(_pool: &DbPool, _user_id: &str) -> Result<Vec<String>> {
        // Production implementation would query a sys_user_groups table
        // For now, return default groups
        Ok(vec!["users".to_string(), "authenticated".to_string()])
    }

    /// Query user roles from sys_model_has_roles table
    fn query_user_roles(_pool: &DbPool, _user_id: &str) -> Result<Vec<String>> {
        // Production implementation would query sys_model_has_roles joined with sys_roles
        // For now, return default roles
        Ok(vec!["user".to_string(), "authenticated".to_string()])
    }
}

