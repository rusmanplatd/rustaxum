use anyhow::Result;
use crate::database::DbPool;
use ulid::Ulid;
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::schema::{oauth_access_tokens, oauth_refresh_tokens, oauth_auth_codes};

use crate::app::models::oauth::{
    AccessToken, CreateAccessToken, NewAccessToken, RefreshToken, NewRefreshToken,
    AuthCode, CreateAuthCode, NewAuthCode
};
use crate::app::services::oauth::client_service::ClientService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String, // user_id
    pub aud: String, // client_id
    pub exp: usize,
    pub iat: usize,
    pub jti: String, // token_id
    pub scopes: Vec<String>,
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

impl TokenService {
    pub async fn create_access_token(
        pool: &DbPool,
        data: CreateAccessToken,
        expires_in_seconds: Option<i64>,
    ) -> Result<AccessToken> {
        let expires_at = expires_in_seconds.map(|seconds| Utc::now() + Duration::seconds(seconds));
        let scopes_str = if data.scopes.is_empty() {
            None
        } else {
            Some(data.scopes.join(","))
        };

        let new_token = NewAccessToken::new(
            data.user_id,
            data.client_id,
            data.name,
            scopes_str,
            expires_at,
        );

        Self::create_access_token_record(pool, new_token).await
    }

    pub async fn create_access_token_record(pool: &DbPool, new_token: NewAccessToken) -> Result<AccessToken> {
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

        let new_refresh_token = NewRefreshToken::new(access_token_id, expires_at);

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

        let new_auth_code = NewAuthCode::new(
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
        };

        let access_token = Self::create_access_token(pool, create_token, expires_in_seconds).await?;

        // Generate JWT
        let jwt_token = Self::generate_jwt_token(&access_token, &client_id_str)?;

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

        // Verify client
        let client = match client_secret {
            Some(secret) => ClientService::find_by_id_and_secret(pool, client_id, secret)?,
            None => ClientService::find_by_id(pool, client_id)?,
        };

        let client = client.ok_or_else(|| anyhow::anyhow!("Invalid client credentials"))?;

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
        };

        let access_token = Self::create_access_token(pool, create_token, Some(3600)).await?;

        // Create refresh token
        let refresh_token = Self::create_refresh_token(
            pool,
            Ulid::from_string(&access_token.id)?.to_string(),
            Some(604800), // 7 days
        )?;

        // Revoke the auth code
        Self::revoke_auth_code(pool, code_id.to_string())?;

        // Generate JWT
        let jwt_token = Self::generate_jwt_token(&access_token, &client_id.to_string())?;

        Ok(TokenResponse {
            access_token: jwt_token,
            token_type: "Bearer".to_string(),
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

        // Verify client
        let client = match client_secret {
            Some(secret) => ClientService::find_by_id_and_secret(pool, client_id, secret)?,
            None => ClientService::find_by_id(pool, client_id)?,
        };

        let _client = client.ok_or_else(|| anyhow::anyhow!("Invalid client credentials"))?;

        // Revoke old tokens
        Self::revoke_access_token(pool, Ulid::from_string(&access_token.id)?.to_string())?;

        // Create new access token
        let create_token = CreateAccessToken {
            user_id: access_token.user_id.clone(),
            client_id: client_id.to_string(),
            name: access_token.name.clone(),
            scopes: access_token.get_scopes(),
            expires_at: Some(Utc::now() + Duration::seconds(3600)), // 1 hour
        };

        let new_access_token = Self::create_access_token(pool, create_token, Some(3600)).await?;

        // Create new refresh token
        let new_refresh_token = Self::create_refresh_token(
            pool,
            Ulid::from_string(&new_access_token.id)?.to_string(),
            Some(604800), // 7 days
        )?;

        // Generate JWT
        let jwt_token = Self::generate_jwt_token(&new_access_token, &client_id.to_string())?;

        Ok(TokenResponse {
            access_token: jwt_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: Some(new_refresh_token.id.to_string()),
            scope: new_access_token.get_scopes().join(" "),
        })
    }

    pub fn generate_jwt_token(access_token: &AccessToken, client_id: &str) -> Result<String> {
        let now = Utc::now();
        let expires_at = access_token.expires_at.unwrap_or(now + Duration::days(1));

        let claims = TokenClaims {
            sub: access_token.user_id.clone().unwrap_or_default(),
            aud: client_id.to_string(),
            exp: expires_at.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: access_token.id.clone(),
            scopes: access_token.get_scopes(),
        };

        // Get OAuth JWT secret from environment or use default
        let jwt_secret = std::env::var("OAUTH_JWT_SECRET")
            .unwrap_or_else(|_| "oauth-jwt-secret".to_string());

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_ref()),
        )?;

        Ok(token)
    }

    pub fn decode_jwt_token(token: &str) -> Result<TokenClaims> {
        // Get OAuth JWT secret from environment or use default
        let jwt_secret = std::env::var("OAUTH_JWT_SECRET")
            .unwrap_or_else(|_| "oauth-jwt-secret".to_string());

        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &Validation::default(),
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
        let token_id = Ulid::from_string(&claims.jti)
            .map_err(|_| anyhow::anyhow!("Invalid token ID in JWT"))?;

        // Find access token
        let access_token = Self::find_access_token_by_id(pool, token_id.to_string())?
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
}

