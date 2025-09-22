use serde::{Deserialize, Serialize};
use crate::app::models::DieselUlid;
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use crate::app::query_builder::{SortDirection};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::oauth_auth_codes)]
pub struct AuthCode {
    pub id: DieselUlid,
    pub user_id: String,
    pub client_id: String,
    pub scopes: Option<String>,
    pub revoked: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub challenge: Option<String>,
    pub challenge_method: Option<String>,
    pub redirect_uri: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuthCode {
    pub user_id: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub redirect_uri: String,
    pub challenge: Option<String>,
    pub challenge_method: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::oauth_auth_codes)]
pub struct NewAuthCode {
    pub id: DieselUlid,
    pub user_id: String,
    pub client_id: String,
    pub scopes: Option<String>,
    pub revoked: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub challenge: Option<String>,
    pub challenge_method: Option<String>,
    pub redirect_uri: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AuthCodeResponse {
    pub id: DieselUlid,
    pub user_id: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub revoked: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub redirect_uri: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AuthCode {

    pub fn to_response(&self) -> AuthCodeResponse {
        let scopes = match &self.scopes {
            Some(scope_str) => scope_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            None => Vec::new(),
        };

        AuthCodeResponse {
            id: self.id,
            user_id: self.user_id.clone(),
            client_id: self.client_id.clone(),
            scopes,
            revoked: self.revoked,
            expires_at: self.expires_at,
            redirect_uri: self.redirect_uri.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn get_scopes(&self) -> Vec<String> {
        match &self.scopes {
            Some(scope_str) => scope_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => Utc::now() > expires_at,
            None => false,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.revoked && !self.is_expired()
    }

    pub fn verify_pkce_challenge(&self, verifier: &str) -> bool {
        match (&self.challenge, &self.challenge_method) {
            (Some(challenge), Some(method)) => {
                match method.as_str() {
                    "S256" => {
                        use sha2::{Sha256, Digest};
                        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

                        let mut hasher = Sha256::new();
                        hasher.update(verifier.as_bytes());
                        let digest = hasher.finalize();
                        let encoded = URL_SAFE_NO_PAD.encode(digest);

                        encoded == *challenge
                    },
                    "plain" => verifier == challenge,
                    _ => false,
                }
            },
            (None, None) => true, // No PKCE challenge
            _ => false, // Invalid PKCE setup
        }
    }
}

impl NewAuthCode {
    pub fn new(
        user_id: String,
        client_id: String,
        scopes: Option<String>,
        redirect_uri: String,
        challenge: Option<String>,
        challenge_method: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            user_id,
            client_id,
            scopes,
            revoked: false,
            expires_at,
            challenge,
            challenge_method,
            redirect_uri,
            created_at: now,
            updated_at: now,
        }
    }
}

impl crate::app::query_builder::Queryable for AuthCode {
    fn table_name() -> &'static str {
        "oauth_auth_codes"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "client_id",
            "revoked",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "client_id",
            "scopes",
            "revoked",
            "expires_at",
            "redirect_uri",
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

// Implement the query builder service for AuthCode
crate::impl_query_builder_service!(AuthCode);