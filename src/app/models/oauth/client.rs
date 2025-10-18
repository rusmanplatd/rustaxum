use crate::app::models::DieselUlid;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use crate::app::query_builder::{SortDirection};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, QueryableByName, ToSchema)]
#[diesel(table_name = crate::schema::oauth_clients)]
pub struct Client {
    pub id: DieselUlid,
    pub organization_id: Option<DieselUlid>,
    pub user_id: Option<DieselUlid>,
    pub name: String,
    pub secret: Option<String>,
    pub provider: Option<String>,
    pub redirect_uris: String,
    pub personal_access_client: bool,
    pub password_client: bool,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by_id: DieselUlid,
    pub updated_by_id: DieselUlid,
    pub deleted_by_id: Option<DieselUlid>,
    pub public_key_pem: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub jwks_uri: Option<String>,
    pub token_endpoint_auth_method: String,
    pub response_types: Option<Vec<Option<String>>>,
    pub grant_types: Option<Vec<Option<String>>>,
    pub scope: String,
    pub audience: Option<Vec<Option<String>>>,
    pub require_auth_time: bool,
    pub default_max_age: Option<i32>,
    pub require_pushed_authorization_requests: bool,
    pub certificate_bound_access_tokens: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateClient {
    pub organization_id: Option<DieselUlid>,
    pub user_id: Option<DieselUlid>,
    #[schema(example = "My OAuth App")]
    pub name: String,
    pub redirect_uris: Vec<String>,
    #[schema(example = false)]
    pub personal_access_client: bool,
    #[schema(example = false)]
    pub password_client: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateClient {
    pub name: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    pub revoked: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ClientResponse {
    pub id: DieselUlid,
    #[schema(example = "My OAuth App")]
    pub name: String,
    pub secret: Option<String>,
    pub redirect_uris: Vec<String>,
    pub personal_access_client: bool,
    pub password_client: bool,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Client {
    pub fn new(
        organization_id: Option<DieselUlid>,
        user_id: Option<DieselUlid>,
        name: String,
        secret: Option<String>,
        redirect_uris: String,
        personal_access_client: bool,
        password_client: bool,
        created_by_id: DieselUlid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            organization_id,
            user_id,
            name,
            secret,
            provider: None,
            redirect_uris,
            personal_access_client,
            password_client,
            revoked: false,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by_id,
            updated_by_id: created_by_id,
            deleted_by_id: None,
            public_key_pem: None,
            metadata: Some(serde_json::json!({})),
            jwks_uri: None,
            token_endpoint_auth_method: "client_secret_basic".to_string(),
            response_types: Some(vec![Some("code".to_string())]),
            grant_types: Some(vec![Some("authorization_code".to_string()), Some("refresh_token".to_string())]),
            scope: "openid".to_string(),
            audience: None,
            require_auth_time: false,
            default_max_age: None,
            require_pushed_authorization_requests: false,
            certificate_bound_access_tokens: false,
        }
    }

    pub fn to_response(&self) -> ClientResponse {
        ClientResponse {
            id: self.id,
            name: self.name.clone(),
            secret: self.secret.clone(),
            redirect_uris: self.redirect_uris
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            personal_access_client: self.personal_access_client,
            password_client: self.password_client,
            revoked: self.revoked,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn to_response_without_secret(&self) -> ClientResponse {
        let mut response = self.to_response();
        response.secret = None;
        response
    }

    pub fn get_redirect_uris(&self) -> Vec<String> {
        self.redirect_uris
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    }

    pub fn is_valid_redirect_uri(&self, uri: &str) -> bool {
        self.get_redirect_uris().contains(&uri.to_string())
    }

    pub fn has_secret(&self) -> bool {
        self.secret.is_some()
    }

    pub fn verify_secret(&self, secret: &str) -> bool {
        match &self.secret {
            Some(client_secret) => client_secret == secret,
            None => false,
        }
    }
}


impl crate::app::query_builder::Queryable for Client {
    fn table_name() -> &'static str {
        "oauth_clients"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "organization_id",
            "user_id",
            "name",
            "personal_access_client",
            "password_client",
            "revoked",
            "created_at",
            "updated_at",
            "deleted_at",
            "created_by_id",
            "updated_by_id",
            "deleted_by_id",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "organization_id",
            "user_id",
            "name",
            "redirect_uris",
            "personal_access_client",
            "password_client",
            "revoked",
            "created_at",
            "updated_at",
            "deleted_at",
            "created_by_id",
            "updated_by_id",
            "deleted_by_id",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "user",
            "access_tokens",
            "refresh_tokens",
        ]
    }
}

// Implement the query builder service for Client
crate::impl_query_builder_service!(Client);