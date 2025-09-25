use anyhow::Result;
use crate::database::DbPool;
use diesel::prelude::*;
use chrono::{Utc, Duration};
use serde_json::json;

use crate::schema::oauth_device_codes;
use crate::app::models::oauth::{DeviceCode, NewDeviceCode, DeviceAuthorizationResponse, CreateDeviceCode};
use crate::app::services::oauth::{ClientService, ScopeService, TokenService};
use crate::app::models::oauth::CreateAccessToken;
use crate::app::traits::ServiceActivityLogger;

pub struct DeviceService;

impl ServiceActivityLogger for DeviceService {}

impl DeviceService {
    /// RFC 8628: Device Authorization Request
    /// Creates a device code and user code for device authorization flow
    pub async fn create_device_authorization(
        pool: &DbPool,
        request: CreateDeviceCode,
    ) -> Result<DeviceAuthorizationResponse> {
        // Validate client exists
        let client = ClientService::find_by_id(pool, request.client_id.clone())?
            .ok_or_else(|| anyhow::anyhow!("Invalid client"))?;

        // Validate scopes
        let requested_scopes = request.scope
            .as_deref()
            .unwrap_or("")
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let scopes = ScopeService::validate_scopes(pool, &requested_scopes).await?;

        // Generate codes
        let device_code = DeviceCode::generate_device_code();
        let user_code = DeviceCode::generate_user_code();

        // Configuration (these could be moved to config)
        let expires_in = 1800; // 30 minutes (RFC 8628 recommends between 10 minutes and several hours)
        let interval = 5; // 5 seconds minimum polling interval
        let verification_uri = std::env::var("OAUTH_DEVICE_VERIFICATION_URI")
            .unwrap_or_else(|_| "/oauth/device".to_string());

        let verification_uri_complete = Some(format!("{}?user_code={}", verification_uri, user_code));

        // Create device code record
        let new_device_code = NewDeviceCode::new(
            device_code.clone(),
            user_code.clone(),
            client.id.to_string(),
            Some(ScopeService::get_scope_names(&scopes).join(" ")),
            verification_uri.clone(),
            verification_uri_complete.clone(),
            expires_in,
            interval,
        );

        let device_auth = Self::create_device_code_record(pool, new_device_code)?;

        // Log device authorization creation
        let service = DeviceService;
        let properties = json!({
            "device_code_id": device_auth.id.to_string(),
            "client_id": request.client_id,
            "user_code": user_code,
            "scopes": ScopeService::get_scope_names(&scopes),
            "expires_in": expires_in,
            "verification_uri": verification_uri
        });

        if let Err(e) = service.log_system_event(
            "device_authorization_created",
            &format!("Device authorization created with user code: {}", user_code),
            Some(properties)
        ).await {
            eprintln!("Failed to log device authorization creation: {}", e);
        }

        Ok(device_auth.to_authorization_response())
    }

    /// RFC 8628: Device Access Token Request (Polling)
    /// Exchange device code for access token when user has authorized
    pub async fn poll_device_token(
        pool: &DbPool,
        device_code: String,
        client_id: String,
    ) -> Result<crate::app::services::oauth::TokenResponse> {
        // Find device code
        let device_auth = Self::find_by_device_code(pool, &device_code)?
            .ok_or_else(|| anyhow::anyhow!("Invalid device code"))?;

        // Validate client
        if device_auth.client_id != client_id {
            return Err(anyhow::anyhow!("Device code was not issued to this client"));
        }

        // Check if expired
        if device_auth.is_expired() {
            Self::revoke_device_code(pool, device_auth.id.to_string())?;
            return Err(anyhow::anyhow!("Device code expired"));
        }

        // Check if revoked
        if device_auth.revoked {
            return Err(anyhow::anyhow!("Device code has been revoked"));
        }

        // Check authorization status
        if !device_auth.user_authorized {
            return Err(anyhow::anyhow!("Authorization pending")); // RFC 8628 "authorization_pending" error
        }

        let user_id = device_auth.user_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("User not found for authorized device"))?;

        // Create access token
        let create_token = CreateAccessToken {
            user_id: Some(user_id.clone()),
            client_id: client_id.clone(),
            name: Some("Device Authorization Grant".to_string()),
            scopes: device_auth.get_scopes(),
            expires_at: Some(Utc::now() + Duration::seconds(3600)), // 1 hour
            jwk_thumbprint: None, // Device flow doesn't use DPoP
        };

        let access_token = TokenService::create_access_token(pool, create_token, Some(3600), None).await?;

        // Create refresh token
        let refresh_token = TokenService::create_refresh_token(
            pool,
            access_token.id.to_string(),
            Some(604800), // 7 days
        )?;

        // Revoke the device code (single use)
        Self::revoke_device_code(pool, device_auth.id.to_string())?;

        // Generate JWT
        let jwt_token = TokenService::generate_jwt_token(&access_token, &client_id)?;

        // Log successful token exchange
        let service = DeviceService;
        let properties = json!({
            "device_code_id": device_auth.id.to_string(),
            "access_token_id": access_token.id.to_string(),
            "user_id": user_id,
            "client_id": client_id,
            "scopes": device_auth.get_scopes()
        });

        if let Err(e) = service.log_system_event(
            "device_token_granted",
            &format!("Device authorization token granted for user: {}", user_id),
            Some(properties)
        ).await {
            eprintln!("Failed to log device token grant: {}", e);
        }

        Ok(crate::app::services::oauth::TokenResponse {
            access_token: jwt_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: Some(refresh_token.id.to_string()),
            scope: device_auth.get_scopes().join(" "),
        })
    }

    /// User authorization of device code
    /// This is called when user visits verification URI and authorizes the device
    pub async fn authorize_device_code(
        pool: &DbPool,
        user_code: String,
        user_id: String,
    ) -> Result<()> {
        let mut conn = pool.get()?;

        // Find device code by user code
        let device_code = oauth_device_codes::table
            .filter(oauth_device_codes::user_code.eq(user_code.clone()))
            .filter(oauth_device_codes::revoked.eq(false))
            .first::<DeviceCode>(&mut conn)
            .optional()?
            .ok_or_else(|| anyhow::anyhow!("Invalid or expired user code"))?;

        // Check if expired
        if device_code.is_expired() {
            Self::revoke_device_code(pool, device_code.id.to_string())?;
            return Err(anyhow::anyhow!("User code has expired"));
        }

        // Check if already authorized
        if device_code.user_authorized {
            return Err(anyhow::anyhow!("Device has already been authorized"));
        }

        // Validate user has access to the client's organization
        let user_id_ulid = crate::app::models::DieselUlid::from_string(&user_id)?;
        if !ClientService::validate_user_organization_access(pool, device_code.client_id.clone(), user_id_ulid)? {
            return Err(anyhow::anyhow!("User does not have access to this application"));
        }

        // Authorize the device
        diesel::update(oauth_device_codes::table.filter(oauth_device_codes::id.eq(device_code.id)))
            .set((
                oauth_device_codes::user_authorized.eq(true),
                oauth_device_codes::user_id.eq(Some(user_id.clone())),
                oauth_device_codes::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        // Log device authorization
        let service = DeviceService;
        let properties = json!({
            "device_code_id": device_code.id.to_string(),
            "user_code": user_code,
            "user_id": user_id,
            "client_id": device_code.client_id,
            "scopes": device_code.get_scopes()
        });

        if let Err(e) = service.log_system_event(
            "device_authorized",
            &format!("Device with user code {} authorized by user {}", user_code, user_id),
            Some(properties)
        ).await {
            eprintln!("Failed to log device authorization: {}", e);
        }

        Ok(())
    }

    /// Find device code by user code (for user authorization)
    pub fn find_by_user_code(pool: &DbPool, user_code: &str) -> Result<Option<DeviceCode>> {
        let mut conn = pool.get()?;

        let device_code = oauth_device_codes::table
            .filter(oauth_device_codes::user_code.eq(user_code))
            .filter(oauth_device_codes::revoked.eq(false))
            .first::<DeviceCode>(&mut conn)
            .optional()?;

        Ok(device_code)
    }

    /// Find device code by device code (for polling)
    pub fn find_by_device_code(pool: &DbPool, device_code: &str) -> Result<Option<DeviceCode>> {
        let mut conn = pool.get()?;

        let device_auth = oauth_device_codes::table
            .filter(oauth_device_codes::device_code.eq(device_code))
            .filter(oauth_device_codes::revoked.eq(false))
            .first::<DeviceCode>(&mut conn)
            .optional()?;

        Ok(device_auth)
    }

    /// Verify device code by user code and authorize it for the given user
    pub async fn verify_device_code(pool: &DbPool, user_code: &str, user_id: &str) -> Result<()> {
        use diesel::prelude::*;

        let mut conn = pool.get()?;
        let now = Utc::now();

        // Find device code by user_code
        let device_code: DeviceCode = oauth_device_codes::table
            .filter(oauth_device_codes::user_code.eq(user_code))
            .filter(oauth_device_codes::expires_at.gt(now))
            .filter(oauth_device_codes::revoked.eq(false))
            .select(DeviceCode::as_select())
            .first(&mut conn)
            .map_err(|_| anyhow::anyhow!("Invalid or expired user code"))?;

        // Check if already authorized
        if device_code.user_id.is_some() {
            return Err(anyhow::anyhow!("Device code already authorized"));
        }

        // Update device code with user_id to authorize it
        diesel::update(oauth_device_codes::table.filter(oauth_device_codes::id.eq(&device_code.id)))
            .set((
                oauth_device_codes::user_id.eq(Some(user_id)),
                oauth_device_codes::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        tracing::info!("Device code {} successfully authorized for user {}", user_code, user_id);
        Ok(())
    }

    /// Revoke device code
    pub fn revoke_device_code(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;
        let now = Utc::now();

        diesel::update(oauth_device_codes::table.filter(oauth_device_codes::id.eq(id)))
            .set((
                oauth_device_codes::revoked.eq(true),
                oauth_device_codes::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    /// Cleanup expired device codes (should be run periodically)
    pub fn cleanup_expired_codes(pool: &DbPool) -> Result<usize> {
        let mut conn = pool.get()?;
        let now = Utc::now();

        let deleted_count = diesel::delete(oauth_device_codes::table)
            .filter(oauth_device_codes::expires_at.lt(now))
            .execute(&mut conn)?;

        Ok(deleted_count)
    }

    /// Create device code record
    fn create_device_code_record(pool: &DbPool, new_device_code: NewDeviceCode) -> Result<DeviceCode> {
        let mut conn = pool.get()?;

        let inserted_device_code: DeviceCode = diesel::insert_into(oauth_device_codes::table)
            .values(&new_device_code)
            .get_result(&mut conn)?;

        Ok(inserted_device_code)
    }

    /// List active device codes for client (admin function)
    pub fn list_active_codes(pool: &DbPool, client_id: Option<String>) -> Result<Vec<DeviceCode>> {
        let mut conn = pool.get()?;
        let now = Utc::now();

        let mut query = oauth_device_codes::table
            .filter(oauth_device_codes::revoked.eq(false))
            .filter(oauth_device_codes::expires_at.gt(now))
            .into_boxed();

        if let Some(client_id) = client_id {
            query = query.filter(oauth_device_codes::client_id.eq(client_id));
        }

        let device_codes = query
            .order(oauth_device_codes::created_at.desc())
            .load::<DeviceCode>(&mut conn)?;

        Ok(device_codes)
    }

    /// Get device authorization statistics (admin function)
    pub fn get_device_stats(pool: &DbPool) -> Result<serde_json::Value> {
        let mut conn = pool.get()?;
        let now = Utc::now();

        // Count active codes
        let active_codes = oauth_device_codes::table
            .filter(oauth_device_codes::revoked.eq(false))
            .filter(oauth_device_codes::expires_at.gt(now))
            .count()
            .get_result::<i64>(&mut conn)?;

        // Count pending authorizations
        let pending_auth = oauth_device_codes::table
            .filter(oauth_device_codes::revoked.eq(false))
            .filter(oauth_device_codes::expires_at.gt(now))
            .filter(oauth_device_codes::user_authorized.eq(false))
            .count()
            .get_result::<i64>(&mut conn)?;

        // Count authorized but not yet polled
        let authorized_pending = oauth_device_codes::table
            .filter(oauth_device_codes::revoked.eq(false))
            .filter(oauth_device_codes::expires_at.gt(now))
            .filter(oauth_device_codes::user_authorized.eq(true))
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(json!({
            "active_device_codes": active_codes,
            "pending_authorization": pending_auth,
            "authorized_pending_poll": authorized_pending,
            "timestamp": now
        }))
    }
}