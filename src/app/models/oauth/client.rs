use crate::app::models::DieselUlid;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use crate::app::query_builder::{SortDirection};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, QueryableByName)]
#[diesel(table_name = crate::schema::oauth_clients)]
pub struct Client {
    pub id: DieselUlid,
    pub user_id: Option<String>,
    pub name: String,
    pub secret: Option<String>,
    pub provider: Option<String>,
    pub redirect_uris: String,
    pub personal_access_client: bool,
    pub password_client: bool,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateClient {
    pub user_id: Option<String>,
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub personal_access_client: bool,
    pub password_client: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateClient {
    pub name: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    pub revoked: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ClientResponse {
    pub id: DieselUlid,
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
        user_id: Option<String>,
        name: String,
        secret: Option<String>,
        redirect_uris: String,
        personal_access_client: bool,
        password_client: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
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
            "user_id",
            "name",
            "personal_access_client",
            "password_client",
            "revoked",
            "created_at",
            "updated_at",
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
            "user_id",
            "name",
            "redirect_uris",
            "personal_access_client",
            "password_client",
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
            "user",
            "access_tokens",
            "refresh_tokens",
        ]
    }
}

// Implement the query builder service for Client
crate::impl_query_builder_service!(Client);