use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPosition {
    pub id: Ulid,
    pub name: String,
    pub code: Option<String>,
    pub job_level_id: Ulid,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateJobPosition {
    pub name: String,
    pub code: Option<String>,
    pub job_level_id: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateJobPosition {
    pub name: Option<String>,
    pub code: Option<String>,
    pub job_level_id: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct JobPositionResponse {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub job_level_id: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl JobPosition {
    pub fn new(name: String, code: Option<String>, job_level_id: Ulid, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            name,
            code,
            job_level_id,
            description,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> JobPositionResponse {
        JobPositionResponse {
            id: self.id.to_string(),
            name: self.name.clone(),
            code: self.code.clone(),
            job_level_id: self.job_level_id.to_string(),
            description: self.description.clone(),
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl FromRow<'_, PgRow> for JobPosition {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        let job_level_id_str: String = row.try_get("job_level_id")?;
        let job_level_id = Ulid::from_string(&job_level_id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "job_level_id".to_string(),
            source: Box::new(e),
        })?;

        Ok(JobPosition {
            id,
            name: row.try_get("name")?,
            code: row.try_get("code")?,
            job_level_id,
            description: row.try_get("description")?,
            is_active: row.try_get("is_active")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}