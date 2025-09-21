use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::{Queryable, SortDirection};

/// Province model representing a state/province within a country
/// Contains geographical and administrative information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Province {
    /// Unique province identifier
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: Ulid,
    /// ID of the country this province belongs to
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub country_id: Ulid,
    /// Province name
    #[schema(example = "California")]
    pub name: String,
    /// Optional province/state code
    #[schema(example = "CA")]
    pub code: Option<String>,
    /// Creation timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// Create province payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateProvince {
    pub country_id: String,
    pub name: String,
    pub code: Option<String>,
}

/// Update province payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateProvince {
    pub country_id: Option<String>,
    pub name: Option<String>,
    pub code: Option<String>,
}

/// Province response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct ProvinceResponse {
    pub id: String,
    pub country_id: String,
    pub name: String,
    pub code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Province {
    pub fn new(country_id: Ulid, name: String, code: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            country_id,
            name,
            code,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> ProvinceResponse {
        ProvinceResponse {
            id: self.id.to_string(),
            country_id: self.country_id.to_string(),
            name: self.name.clone(),
            code: self.code.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl crate::app::query_builder::Queryable for Province {
    fn table_name() -> &'static str {
        "provinces"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "country_id",
            "name",
            "code",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "country_id",
            "name",
            "code",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "country_id",
            "name",
            "code",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("name", SortDirection::Asc))
    }
}

// Implement the query builder service for Province
crate::impl_query_builder_service!(Province);