use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermission {
    pub id: Ulid,
    pub role_id: Ulid,
    pub permission_id: Ulid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRolePermission {
    pub role_id: Ulid,
    pub permission_id: Ulid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRolePermission {
    pub role_id: Option<Ulid>,
    pub permission_id: Option<Ulid>,
}

#[derive(Debug, Serialize)]
pub struct RolePermissionResponse {
    pub id: String,
    pub role_id: String,
    pub permission_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RolePermission {
    pub fn new(role_id: Ulid, permission_id: Ulid) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            role_id,
            permission_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> RolePermissionResponse {
        RolePermissionResponse {
            id: self.id.to_string(),
            role_id: self.role_id.to_string(),
            permission_id: self.permission_id.to_string(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl FromRow<'_, PgRow> for RolePermission {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        let role_id_str: String = row.try_get("role_id")?;
        let role_id = Ulid::from_string(&role_id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "role_id".to_string(),
            source: Box::new(e),
        })?;

        let permission_id_str: String = row.try_get("permission_id")?;
        let permission_id = Ulid::from_string(&permission_id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "permission_id".to_string(),
            source: Box::new(e),
        })?;

        Ok(RolePermission {
            id,
            role_id,
            permission_id,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
