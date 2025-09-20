use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::query_builder::{Queryable, SortDirection};

/// Job position model representing specific sys_roles within job levels
/// Contains position information and relationship to job level hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JobPosition {
    /// Unique identifier for the job position
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: Ulid,
    /// Job position name
    #[schema(example = "Software Engineering Manager")]
    pub name: String,
    /// Optional job position code
    #[schema(example = "SEM")]
    pub code: Option<String>,
    /// ID of the job level this position belongs to
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub job_level_id: Ulid,
    /// Optional description of the job position
    #[schema(example = "Manages software engineering teams and technical projects")]
    pub description: Option<String>,
    /// Whether the job position is currently active
    #[schema(example = true)]
    pub is_active: bool,
    /// Creation timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// Create job position payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateJobPosition {
    pub name: String,
    pub code: Option<String>,
    pub job_level_id: String,
    pub description: Option<String>,
}

/// Update job position payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateJobPosition {
    pub name: Option<String>,
    pub code: Option<String>,
    pub job_level_id: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

/// Job position response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
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

impl Queryable for JobPosition {
    fn table_name() -> &'static str {
        "job_positions"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "code",
            "job_level_id",
            "description",
            "is_active",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "code",
            "job_level_id",
            "description",
            "is_active",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "code",
            "job_level_id",
            "description",
            "is_active",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("name", SortDirection::Asc))
    }
}