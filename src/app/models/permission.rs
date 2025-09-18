use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: Ulid,
    pub name: String,
    pub guard_name: String,
    pub resource: Option<String>,
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePermission {
    pub name: String,
    pub guard_name: Option<String>,
    pub resource: Option<String>,
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePermission {
    pub name: Option<String>,
    pub guard_name: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PermissionResponse {
    pub id: String,
    pub name: String,
    pub guard_name: String,
    pub resource: Option<String>,
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Permission {
    pub fn new(name: String, guard_name: Option<String>, resource: Option<String>, action: String) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            name,
            guard_name: guard_name.unwrap_or_else(|| "web".to_string()),
            resource,
            action,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> PermissionResponse {
        PermissionResponse {
            id: self.id.to_string(),
            name: self.name.clone(),
            guard_name: self.guard_name.clone(),
            resource: self.resource.clone(),
            action: self.action.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl FromRow<'_, PgRow> for Permission {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        Ok(Permission {
            id,
            name: row.try_get("name")?,
            guard_name: row.try_get("guard_name")?,
            resource: row.try_get("resource")?,
            action: row.try_get("action")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
