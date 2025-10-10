use crate::app::models::DieselUlid;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use crate::app::query_builder::{SortDirection};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = crate::schema::oauth_refresh_tokens)]
pub struct RefreshToken {
    pub id: DieselUlid,
    pub access_token_id: String,
    pub revoked: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRefreshToken {
    pub access_token_id: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub id: DieselUlid,
    pub access_token_id: String,
    pub revoked: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RefreshToken {

    pub fn to_response(&self) -> RefreshTokenResponse {
        RefreshTokenResponse {
            id: self.id,
            access_token_id: self.access_token_id.clone(),
            revoked: self.revoked,
            expires_at: self.expires_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
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
}

impl RefreshToken {
    pub fn new(access_token_id: String, expires_at: Option<DateTime<Utc>>) -> Self {
        let now = Utc::now();
        RefreshToken {
            id: DieselUlid::new(),
            access_token_id,
            revoked: false,
            expires_at,
            created_at: now,
            updated_at: now,
        }
    }
}

impl crate::app::query_builder::Queryable for RefreshToken {
    fn table_name() -> &'static str {
        "oauth_refresh_tokens"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "access_token_id",
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
            "access_token_id",
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
            "access_token",
        ]
    }
}

// Implement the query builder service for RefreshToken
crate::impl_query_builder_service!(RefreshToken);