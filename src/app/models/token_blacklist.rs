use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl FromRow<'_, PgRow> for TokenBlacklist {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        let user_id_str: String = row.try_get("user_id")?;
        let user_id = Ulid::from_string(&user_id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "user_id".to_string(),
            source: Box::new(e),
        })?;

        Ok(TokenBlacklist {
            id,
            token_hash: row.try_get("token_hash")?,
            user_id,
            expires_at: row.try_get("expires_at")?,
            revoked_at: row.try_get("revoked_at")?,
            reason: row.try_get("reason")?,
        })
    }
}