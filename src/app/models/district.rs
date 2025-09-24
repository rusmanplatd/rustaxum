use super::DieselUlid;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::{SortDirection};

/// District model representing a district within a city
/// Contains administrative information for sub-city divisions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::districts)]
pub struct District {
    /// Unique district identifier
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    /// ID of the city this district belongs to
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub city_id: String,
    /// District name
    #[schema(example = "Downtown")]
    pub name: String,
    /// Optional district code
    #[schema(example = "DT")]
    pub code: Option<String>,
    /// Creation timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
    /// Soft delete timestamp
    pub deleted_at: Option<DateTime<Utc>>,
    /// User who created this record
    pub created_by: Option<DieselUlid>,
    /// User who last updated this record
    pub updated_by: Option<DieselUlid>,
    /// User who deleted this record
    pub deleted_by: Option<DieselUlid>,
}

/// Create district payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateDistrict {
    pub city_id: String,
    pub name: String,
    pub code: Option<String>,
}

/// Insertable struct for districts
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::districts)]
pub struct NewDistrict {
    pub id: DieselUlid,
    pub city_id: String,
    pub name: String,
    pub code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by: Option<DieselUlid>,
    pub updated_by: Option<DieselUlid>,
    pub deleted_by: Option<DieselUlid>,
}

/// Update district payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateDistrict {
    pub city_id: Option<String>,
    pub name: Option<String>,
    pub code: Option<String>,
}

/// District response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct DistrictResponse {
    pub id: DieselUlid,
    pub city_id: String,
    pub name: String,
    pub code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NewDistrict {
    pub fn new(city_id: String, name: String, code: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            city_id,
            name,
            code,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        }
    }
}

impl District {
    pub fn new(city_id: String, name: String, code: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            city_id,
            name,
            code,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        }
    }

    pub fn to_response(&self) -> DistrictResponse {
        DistrictResponse {
            id: self.id,
            city_id: self.city_id.clone(),
            name: self.name.clone(),
            code: self.code.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl crate::app::query_builder::Queryable for District {
    fn table_name() -> &'static str {
        "districts"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "city_id",
            "name",
            "code",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "city_id",
            "name",
            "code",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "city_id",
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

// Implement the query builder service for District
crate::impl_query_builder_service!(District);

impl crate::app::models::HasModelType for District {
    fn model_type() -> &'static str {
        "District"
    }
}

impl crate::app::models::activity_log::HasId for District {
    fn id(&self) -> String {
        self.id.to_string()
    }
}