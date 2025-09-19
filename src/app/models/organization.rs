use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: Ulid,
    pub name: String,
    pub organization_type: String,
    pub parent_id: Option<Ulid>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrganization {
    pub name: String,
    pub organization_type: String,
    pub parent_id: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOrganization {
    pub name: Option<String>,
    pub organization_type: Option<String>,
    pub parent_id: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct OrganizationResponse {
    pub id: String,
    pub name: String,
    pub organization_type: String,
    pub parent_id: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Organization {
    pub fn new(name: String, organization_type: String, parent_id: Option<Ulid>, code: Option<String>, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            name,
            organization_type,
            parent_id,
            code,
            description,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> OrganizationResponse {
        OrganizationResponse {
            id: self.id.to_string(),
            name: self.name.clone(),
            organization_type: self.organization_type.clone(),
            parent_id: self.parent_id.map(|id| id.to_string()),
            code: self.code.clone(),
            description: self.description.clone(),
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl FromRow<'_, PgRow> for Organization {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        let parent_id = if let Ok(parent_id_str) = row.try_get::<String, _>("parent_id") {
            Some(Ulid::from_string(&parent_id_str).map_err(|e| sqlx::Error::ColumnDecode {
                index: "parent_id".to_string(),
                source: Box::new(e),
            })?)
        } else {
            None
        };

        Ok(Organization {
            id,
            name: row.try_get("name")?,
            organization_type: row.try_get("type")?,
            parent_id,
            code: row.try_get("code")?,
            description: row.try_get("description")?,
            is_active: row.try_get("is_active")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
