use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use crate::app::query_builder::{Queryable, SortDirection};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, QueryableByName)]
#[diesel(table_name = crate::schema::oauth_access_tokens)]
pub struct AccessToken {
    pub id: Ulid,
    pub user_id: Option<Ulid>,
    pub client_id: Ulid,
    pub name: Option<String>,
    pub scopes: Option<String>,
    pub revoked: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAccessToken {
    pub user_id: Option<Ulid>,
    pub client_id: Ulid,
    pub name: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAccessToken {
    pub name: Option<String>,
    pub revoked: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct AccessTokenResponse {
    pub id: String,
    pub user_id: Option<String>,
    pub client_id: String,
    pub name: Option<String>,
    pub scopes: Vec<String>,
    pub revoked: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AccessToken {
    pub fn new(
        user_id: Option<Ulid>,
        client_id: Ulid,
        name: Option<String>,
        scopes: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            user_id,
            client_id,
            name,
            scopes,
            revoked: false,
            expires_at,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> AccessTokenResponse {
        let scopes = match &self.scopes {
            Some(scope_str) => scope_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            None => Vec::new(),
        };

        AccessTokenResponse {
            id: self.id.to_string(),
            user_id: self.user_id.map(|id| id.to_string()),
            client_id: self.client_id.to_string(),
            name: self.name.clone(),
            scopes,
            revoked: self.revoked,
            expires_at: self.expires_at,
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

    pub fn has_scope(&self, scope: &str) -> bool {
        let scopes = self.get_scopes();
        scopes.contains(&"*".to_string()) || scopes.contains(&scope.to_string())
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
}

impl crate::app::query_builder::Queryable for AccessToken {
    fn table_name() -> &'static str {
        "oauth_access_tokens"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "client_id",
            "name",
            "revoked",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "name",
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
            "name",
            "scopes",
            "revoked",
            "expires_at",
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

// Implement the query builder service for AccessToken
crate::impl_query_builder_service!(AccessToken);