use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use crate::query_builder::{Queryable, SortDirection};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    pub id: Ulid,
    pub access_token_id: Ulid,
    pub revoked: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRefreshToken {
    pub access_token_id: Ulid,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub id: String,
    pub access_token_id: String,
    pub revoked: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RefreshToken {
    pub fn new(access_token_id: Ulid, expires_at: Option<DateTime<Utc>>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            access_token_id,
            revoked: false,
            expires_at,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> RefreshTokenResponse {
        RefreshTokenResponse {
            id: self.id.to_string(),
            access_token_id: self.access_token_id.to_string(),
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

impl Queryable for RefreshToken {
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
}