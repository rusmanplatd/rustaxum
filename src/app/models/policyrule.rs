use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policyrule {
    pub id: Ulid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePolicyrule {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePolicyrule {
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PolicyruleResponse {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Policyrule {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            name,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> PolicyruleResponse {
        PolicyruleResponse {
            id: self.id.to_string(),
            name: self.name.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl FromRow<'_, PgRow> for Policyrule {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        Ok(Policyrule {
            id,
            name: row.try_get("name")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
