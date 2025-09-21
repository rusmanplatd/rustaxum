use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::query_builder::{Queryable, SortDirection};

/// Job level model representing organizational hierarchy levels
/// Contains level information including rank, code, and description
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JobLevel {
    /// Unique identifier for the job level
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: Ulid,
    /// Job level name
    #[schema(example = "Senior Manager")]
    pub name: String,
    /// Optional job level code
    #[schema(example = "SM")]
    pub code: Option<String>,
    /// Numeric level ranking (higher number = higher level)
    #[schema(example = 5)]
    pub level: i32,
    /// Optional description of the job level
    #[schema(example = "Senior management position with team leadership responsibilities")]
    pub description: Option<String>,
    /// Whether the job level is currently active
    #[schema(example = true)]
    pub is_active: bool,
    /// Creation timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// Create job level payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateJobLevel {
    pub name: String,
    pub code: Option<String>,
    pub level: i32,
    pub description: Option<String>,
}

/// Update job level payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateJobLevel {
    pub name: Option<String>,
    pub code: Option<String>,
    pub level: Option<i32>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

/// Job level response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
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

impl Queryable for JobLevel {
    fn table_name() -> &'static str {
        "job_levels"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "code",
            "level",
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
            "level",
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
            "level",
            "description",
            "is_active",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("level", SortDirection::Asc))
    }
}