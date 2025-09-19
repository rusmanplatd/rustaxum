use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobLevel {
    pub id: Ulid,
    pub name: String,
    pub code: Option<String>,
    pub level: i32,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateJobLevel {
    pub name: String,
    pub code: Option<String>,
    pub level: i32,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateJobLevel {
    pub name: Option<String>,
    pub code: Option<String>,
    pub level: Option<i32>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct JobLevelResponse {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub level: i32,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl JobLevel {
    pub fn new(name: String, code: Option<String>, level: i32, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            name,
            code,
            level,
            description,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> JobLevelResponse {
        JobLevelResponse {
            id: self.id.to_string(),
            name: self.name.clone(),
            code: self.code.clone(),
            level: self.level,
            description: self.description.clone(),
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl FromRow<'_, PgRow> for JobLevel {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        Ok(JobLevel {
            id,
            name: row.try_get("name")?,
            code: row.try_get("code")?,
            level: row.try_get("level")?,
            description: row.try_get("description")?,
            is_active: row.try_get("is_active")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}