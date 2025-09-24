use crate::app::models::DieselUlid;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use crate::app::query_builder::{SortDirection};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, ToSchema)]
#[diesel(table_name = crate::schema::oauth_device_codes)]
#[schema(description = "OAuth2 device authorization code for RFC 8628")]
pub struct DeviceCode {
    pub id: DieselUlid,
    pub device_code: String,
    pub user_code: String,
    pub client_id: String,
    pub user_id: Option<String>,
    pub scopes: Option<String>,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub interval: i32, // Polling interval in seconds
    pub user_authorized: bool,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(description = "Request to initiate device authorization flow")]
pub struct CreateDeviceCode {
    pub client_id: String,
    #[schema(example = "read write")]
    pub scope: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::oauth_device_codes)]
pub struct NewDeviceCode {
    pub id: DieselUlid,
    pub device_code: String,
    pub user_code: String,
    pub client_id: String,
    pub user_id: Option<String>,
    pub scopes: Option<String>,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub interval: i32,
    pub user_authorized: bool,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(description = "Device authorization response per RFC 8628")]
pub struct DeviceAuthorizationResponse {
    pub device_code: String,
    #[schema(example = "WDJB-MJHT")]
    pub user_code: String,
    #[schema(example = "https://auth.example.com/device")]
    pub verification_uri: String,
    #[schema(example = "https://auth.example.com/device?user_code=WDJB-MJHT")]
    pub verification_uri_complete: Option<String>,
    #[schema(example = 1800)]
    pub expires_in: i64, // seconds until expiration
    #[schema(example = 5)]
    pub interval: i32, // minimum polling interval in seconds
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(description = "Device code verification request")]
pub struct DeviceCodeVerification {
    #[schema(example = "WDJB-MJHT")]
    pub user_code: String,
}

impl DeviceCode {
    pub fn new(
        device_code: String,
        user_code: String,
        client_id: String,
        scopes: Option<String>,
        verification_uri: String,
        verification_uri_complete: Option<String>,
        expires_in: i64,
        interval: i32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            device_code,
            user_code,
            client_id,
            user_id: None,
            scopes,
            verification_uri,
            verification_uri_complete,
            expires_at: now + chrono::Duration::seconds(expires_in),
            interval,
            user_authorized: false,
            revoked: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_authorization_response(&self) -> DeviceAuthorizationResponse {
        let expires_in = (self.expires_at - Utc::now()).num_seconds().max(0);

        DeviceAuthorizationResponse {
            device_code: self.device_code.clone(),
            user_code: self.user_code.clone(),
            verification_uri: self.verification_uri.clone(),
            verification_uri_complete: self.verification_uri_complete.clone(),
            expires_in,
            interval: self.interval,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.revoked && !self.is_expired()
    }

    pub fn is_pending(&self) -> bool {
        self.is_valid() && !self.user_authorized
    }

    pub fn is_authorized(&self) -> bool {
        self.is_valid() && self.user_authorized
    }

    pub fn get_scopes(&self) -> Vec<String> {
        match &self.scopes {
            Some(scope_str) => scope_str
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
            None => Vec::new(),
        }
    }

    /// Generate a human-readable user code (8 characters, uppercase letters and numbers, no ambiguous characters)
    pub fn generate_user_code() -> String {
        use rand::Rng;
        const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // No 0,O,I,1 to avoid confusion
        let mut rng = rand::thread_rng();

        (0..8)
            .map(|i| {
                if i == 4 {
                    '-' // Add separator after 4 characters: ABCD-EFGH
                } else {
                    CHARS[rng.gen_range(0..CHARS.len())] as char
                }
            })
            .collect()
    }

    /// Generate a secure device code (longer, cryptographically random)
    pub fn generate_device_code() -> String {
        use rand::Rng;
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();

        (0..64)
            .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
            .collect()
    }
}

impl NewDeviceCode {
    pub fn new(
        device_code: String,
        user_code: String,
        client_id: String,
        scopes: Option<String>,
        verification_uri: String,
        verification_uri_complete: Option<String>,
        expires_in: i64,
        interval: i32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            device_code,
            user_code,
            client_id,
            user_id: None,
            scopes,
            verification_uri,
            verification_uri_complete,
            expires_at: now + chrono::Duration::seconds(expires_in),
            interval,
            user_authorized: false,
            revoked: false,
            created_at: now,
            updated_at: now,
        }
    }
}

impl crate::app::query_builder::Queryable for DeviceCode {
    fn table_name() -> &'static str {
        "oauth_device_codes"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "device_code",
            "user_code",
            "client_id",
            "user_id",
            "user_authorized",
            "revoked",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "user_code",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "device_code",
            "user_code",
            "client_id",
            "user_id",
            "scopes",
            "verification_uri",
            "verification_uri_complete",
            "expires_at",
            "interval",
            "user_authorized",
            "revoked",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "client",
            "user",
        ]
    }
}

// Implement the query builder service for DeviceCode
crate::impl_query_builder_service!(DeviceCode);