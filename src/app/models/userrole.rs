use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub id: Ulid,
    pub user_id: Ulid,
    pub role_id: Ulid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRole {
    pub user_id: Ulid,
    pub role_id: Ulid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRole {
    pub user_id: Option<Ulid>,
    pub role_id: Option<Ulid>,
}

#[derive(Debug, Serialize)]
pub struct UserRoleResponse {
    pub id: String,
    pub user_id: String,
    pub role_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserRole {
    pub fn new(user_id: Ulid, role_id: Ulid) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            user_id,
            role_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> UserRoleResponse {
        UserRoleResponse {
            id: self.id.to_string(),
            user_id: self.user_id.to_string(),
            role_id: self.role_id.to_string(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl FromRow<'_, PgRow> for UserRole {
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

        let role_id_str: String = row.try_get("role_id")?;
        let role_id = Ulid::from_string(&role_id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "role_id".to_string(),
            source: Box::new(e),
        })?;

        Ok(UserRole {
            id,
            user_id,
            role_id,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
