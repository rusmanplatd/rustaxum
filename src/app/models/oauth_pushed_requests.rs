use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::{HasModelType, activity_log::HasId};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::oauth_pushed_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OAuthPushedRequest {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub request_uri: String,
    pub client_id: DieselUlid,
    pub request_data: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateOAuthPushedRequest {
    pub client_id: DieselUlid,
    pub request_data: String,
    pub expires_in: Option<i32>, // Seconds until expiration
}
#[derive(Debug, Serialize, ToSchema)]
pub struct OAuthPushedRequestResponse {
    pub id: DieselUlid,
    pub request_uri: String,
    pub client_id: DieselUlid,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PushedAuthorizationResponse {
    pub request_uri: String,
    pub expires_in: i64,
}

impl OAuthPushedRequest {
    pub fn new(
        client_id: DieselUlid,
        request_data: String,
        expires_in: Option<i32>,
    ) -> Self {
        let now = Utc::now();
        let expires_in_seconds = expires_in.unwrap_or(600); // Default 10 minutes
        let expires_at = now + chrono::Duration::seconds(expires_in_seconds as i64);
        let request_uri = format!("urn:ietf:params:oauth:request_uri:{}", DieselUlid::new());

        OAuthPushedRequest {
            id: DieselUlid::new(),
            request_uri,
            client_id,
            request_data,
            expires_at,
            used: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> OAuthPushedRequestResponse {
        OAuthPushedRequestResponse {
            id: self.id,
            request_uri: self.request_uri.clone(),
            client_id: self.client_id,
            expires_at: self.expires_at,
            used: self.used,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn to_par_response(&self) -> PushedAuthorizationResponse {
        let expires_in = (self.expires_at - Utc::now()).num_seconds().max(0);

        PushedAuthorizationResponse {
            request_uri: self.request_uri.clone(),
            expires_in,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.used && !self.is_expired()
    }

    pub fn mark_used(&mut self) {
        self.used = true;
        self.updated_at = Utc::now();
    }

    pub fn get_request_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.request_data)
    }

    pub fn extract_authorization_details(&self) -> Result<AuthorizationDetails, serde_json::Error> {
        let data: serde_json::Value = self.get_request_data()?;

        Ok(AuthorizationDetails {
            response_type: data.get("response_type")
                .and_then(|v| v.as_str())
                .unwrap_or("code")
                .to_string(),
            scope: data.get("scope")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            redirect_uri: data.get("redirect_uri")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            state: data.get("state")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            code_challenge: data.get("code_challenge")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            code_challenge_method: data.get("code_challenge_method")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizationDetails {
    pub response_type: String,
    pub scope: Option<String>,
    pub redirect_uri: String,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

impl HasId for OAuthPushedRequest {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for OAuthPushedRequest {
    fn table_name() -> &'static str {
        "oauth_pushed_requests"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "request_uri",
            "client_id",
            "expires_at",
            "used",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "client_id",
            "expires_at",
            "used",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "request_uri",
            "client_id",
            "expires_at",
            "used",
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
        ]
    }
}

impl HasModelType for OAuthPushedRequest {
    fn model_type() -> &'static str {
        "OAuthPushedRequest"
    }
}

crate::impl_query_builder_service!(OAuthPushedRequest);