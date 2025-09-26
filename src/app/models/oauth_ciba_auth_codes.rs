use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::oauth_ciba_auth_codes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OAuthCibaAuthCode {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub ciba_request_id: DieselUlid,
    pub code: String,
    pub client_id: DieselUlid,
    pub user_id: DieselUlid,
    pub scopes: Option<String>,
    pub redirect_uri: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateOAuthCibaAuthCode {
    pub ciba_request_id: DieselUlid,
    pub client_id: DieselUlid,
    pub user_id: DieselUlid,
    pub scopes: Option<String>,
    pub redirect_uri: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub expires_in: Option<i32>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::oauth_ciba_auth_codes)]
pub struct NewOAuthCibaAuthCode {
    pub id: DieselUlid,
    pub ciba_request_id: DieselUlid,
    pub code: String,
    pub client_id: DieselUlid,
    pub user_id: DieselUlid,
    pub scopes: Option<String>,
    pub redirect_uri: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OAuthCibaAuthCodeResponse {
    pub id: DieselUlid,
    pub ciba_request_id: DieselUlid,
    pub client_id: DieselUlid,
    pub user_id: DieselUlid,
    pub scopes: Option<String>,
    pub redirect_uri: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OAuthCibaAuthCode {
    pub fn new(
        ciba_request_id: DieselUlid,
        client_id: DieselUlid,
        user_id: DieselUlid,
        scopes: Option<String>,
        redirect_uri: Option<String>,
        code_challenge: Option<String>,
        code_challenge_method: Option<String>,
        expires_in: Option<i32>,
    ) -> NewOAuthCibaAuthCode {
        let now = Utc::now();
        let expires_in_seconds = expires_in.unwrap_or(600); // Default 10 minutes
        let expires_at = now + chrono::Duration::seconds(expires_in_seconds as i64);
        let code = Self::generate_auth_code();

        NewOAuthCibaAuthCode {
            id: DieselUlid::new(),
            ciba_request_id,
            code,
            client_id,
            user_id,
            scopes,
            redirect_uri,
            code_challenge,
            code_challenge_method,
            expires_at,
            revoked: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> OAuthCibaAuthCodeResponse {
        OAuthCibaAuthCodeResponse {
            id: self.id,
            ciba_request_id: self.ciba_request_id,
            client_id: self.client_id,
            user_id: self.user_id,
            scopes: self.scopes.clone(),
            redirect_uri: self.redirect_uri.clone(),
            code_challenge: self.code_challenge.clone(),
            code_challenge_method: self.code_challenge_method.clone(),
            expires_at: self.expires_at,
            revoked: self.revoked,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.revoked && !self.is_expired()
    }

    pub fn revoke(&mut self) {
        self.revoked = true;
        self.updated_at = Utc::now();
    }

    pub fn has_pkce(&self) -> bool {
        self.code_challenge.is_some()
    }

    pub fn verify_pkce(&self, code_verifier: &str) -> bool {
        match (&self.code_challenge, &self.code_challenge_method) {
            (Some(challenge), Some(method)) => {
                match method.as_str() {
                    "S256" => {
                        use sha2::{Sha256, Digest};
                        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

                        let mut hasher = Sha256::new();
                        hasher.update(code_verifier.as_bytes());
                        let hash = hasher.finalize();
                        let encoded = URL_SAFE_NO_PAD.encode(hash);

                        encoded == *challenge
                    }
                    "plain" => code_verifier == challenge,
                    _ => false,
                }
            }
            _ => true, // No PKCE challenge, so verification passes
        }
    }

    pub fn get_scopes_vec(&self) -> Vec<String> {
        self.scopes
            .as_ref()
            .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }

    fn generate_auth_code() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();

        (0..128)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}

impl HasId for OAuthCibaAuthCode {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for OAuthCibaAuthCode {
    fn table_name() -> &'static str {
        "oauth_ciba_auth_codes"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "ciba_request_id",
            "client_id",
            "user_id",
            "expires_at",
            "revoked",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "ciba_request_id",
            "client_id",
            "user_id",
            "expires_at",
            "revoked",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "ciba_request_id",
            "client_id",
            "user_id",
            "scopes",
            "redirect_uri",
            "code_challenge",
            "code_challenge_method",
            "expires_at",
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
            "ciba_request",
            "client",
            "user",
        ]
    }
}

crate::impl_query_builder_service!(OAuthCibaAuthCode);