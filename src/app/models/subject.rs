use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub id: Ulid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSubject {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSubject {
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubjectResponse {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Subject {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            name,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> SubjectResponse {
        SubjectResponse {
            id: self.id.to_string(),
            name: self.name.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl FromRow<'_, PgRow> for Subject {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        Ok(Subject {
            id,
            name: row.try_get("name")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
