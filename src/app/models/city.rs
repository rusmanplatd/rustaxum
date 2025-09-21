use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use utoipa::ToSchema;
use crate::query_builder::{Queryable, SortDirection};

/// City model representing a city within a province
/// Contains geographical coordinates and administrative information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct City {
    pub id: Ulid,
    pub province_id: Ulid,
    pub name: String,
    pub code: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub latitude: Option<Decimal>,
    #[schema(value_type = Option<f64>)]
    pub longitude: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create city payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateCity {
    pub province_id: String,
    pub name: String,
    pub code: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub latitude: Option<Decimal>,
    #[schema(value_type = Option<f64>)]
    pub longitude: Option<Decimal>,
}

/// Update city payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateCity {
    pub province_id: Option<String>,
    pub name: Option<String>,
    pub code: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub latitude: Option<Decimal>,
    #[schema(value_type = Option<f64>)]
    pub longitude: Option<Decimal>,
}

/// City response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct CityResponse {
    pub id: String,
    pub province_id: String,
    pub name: String,
    pub code: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub latitude: Option<Decimal>,
    #[schema(value_type = Option<f64>)]
    pub longitude: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl City {
    pub fn new(
        province_id: Ulid,
        name: String,
        code: Option<String>,
        latitude: Option<Decimal>,
        longitude: Option<Decimal>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            province_id,
            name,
            code,
            latitude,
            longitude,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> CityResponse {
        CityResponse {
            id: self.id.to_string(),
            province_id: self.province_id.to_string(),
            name: self.name.clone(),
            code: self.code.clone(),
            latitude: self.latitude,
            longitude: self.longitude,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}


impl Queryable for City {
    fn table_name() -> &'static str {
        "cities"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "province_id",
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
            "province_id",
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
            "province_id",
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