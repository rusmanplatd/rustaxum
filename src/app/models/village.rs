use super::DieselUlid;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use utoipa::ToSchema;
use crate::app::query_builder::{SortDirection};

/// Village model representing a village within a district
/// Contains geographical coordinates and local community information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::villages)]
pub struct Village {
    /// Unique village identifier
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    /// ID of the district this village belongs to
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub district_id: String,
    /// Village name
    #[schema(example = "Green Valley")]
    pub name: String,
    /// Optional village code
    #[schema(example = "GV")]
    pub code: Option<String>,
    /// Optional latitude coordinate
    #[schema(value_type = Option<f64>)]
    pub latitude: Option<Decimal>,
    /// Optional longitude coordinate
    #[schema(value_type = Option<f64>)]
    pub longitude: Option<Decimal>,
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

/// Create village payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateVillage {
    pub district_id: String,
    pub name: String,
    pub code: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub latitude: Option<Decimal>,
    #[schema(value_type = Option<f64>)]
    pub longitude: Option<Decimal>,
}

/// Insertable struct for creating new villages in the database
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::villages)]
pub struct NewVillage {
    pub id: DieselUlid,
    pub district_id: String,
    pub name: String,
    pub code: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by: Option<DieselUlid>,
    pub updated_by: Option<DieselUlid>,
    pub deleted_by: Option<DieselUlid>,
}

/// Update village payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateVillage {
    pub district_id: Option<String>,
    pub name: Option<String>,
    pub code: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub latitude: Option<Decimal>,
    #[schema(value_type = Option<f64>)]
    pub longitude: Option<Decimal>,
}

/// Village response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct VillageResponse {
    pub id: DieselUlid,
    pub district_id: String,
    pub name: String,
    pub code: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub latitude: Option<Decimal>,
    #[schema(value_type = Option<f64>)]
    pub longitude: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Village {
    pub fn new(
        district_id: String,
        name: String,
        code: Option<String>,
        latitude: Option<Decimal>,
        longitude: Option<Decimal>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            district_id,
            name,
            code,
            latitude,
            longitude,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        }
    }

    pub fn to_response(&self) -> VillageResponse {
        VillageResponse {
            id: self.id,
            district_id: self.district_id.clone(),
            name: self.name.clone(),
            code: self.code.clone(),
            latitude: self.latitude,
            longitude: self.longitude,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl NewVillage {
    pub fn new(
        district_id: String,
        name: String,
        code: Option<String>,
        latitude: Option<Decimal>,
        longitude: Option<Decimal>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            district_id,
            name,
            code,
            latitude,
            longitude,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        }
    }
}

impl crate::app::query_builder::Queryable for Village {
    fn table_name() -> &'static str {
        "villages"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "district_id",
            "name",
            "code",
            "latitude",
            "longitude",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "district_id",
            "name",
            "code",
            "latitude",
            "longitude",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "district_id",
            "name",
            "code",
            "latitude",
            "longitude",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("name", SortDirection::Asc))
    }
}

// Implement the query builder service for Village
crate::impl_query_builder_service!(Village);

impl crate::app::models::HasModelType for Village {
    fn model_type() -> &'static str {
        "Village"
    }
}

impl crate::app::models::activity_log::HasId for Village {
    fn id(&self) -> String {
        self.id.to_string()
    }
}