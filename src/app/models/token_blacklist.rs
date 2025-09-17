use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TokenBlacklist {
    pub id: Ulid,
    pub token_hash: String,
    pub user_id: Ulid,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: DateTime<Utc>,
    pub reason: Option<String>,
}

impl TokenBlacklist {
    pub fn new(token_hash: String, user_id: Ulid, expires_at: DateTime<Utc>, reason: Option<String>) -> Self {
        Self {
            id: Ulid::new(),
            token_hash,
            user_id,
            expires_at,
            revoked_at: Utc::now(),
            reason,
        }
    }
}