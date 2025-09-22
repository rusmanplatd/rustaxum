use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use utoipa::ToSchema;
use crate::app::query_builder::{SortDirection};

/// City model representing a city within a province
/// Contains geographical coordinates and administrative information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::cities)]
pub struct City {
    pub id: DieselUlid,
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

/// Insertable struct for creating new cities in the database
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::cities)]
pub struct NewCity {
    pub id: DieselUlid,
    pub province_id: String,
    pub name: String,
    pub code: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub id: DieselUlid,
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
        province_id: String,
        name: String,
        code: Option<String>,
        latitude: Option<Decimal>,
        longitude: Option<Decimal>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
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
            id: self.id,
            province_id: self.province_id.clone(),
            name: self.name.clone(),
            code: self.code.clone(),
            latitude: self.latitude,
            longitude: self.longitude,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl NewCity {
    pub fn new(
        province_id: String,
        name: String,
        code: Option<String>,
        latitude: Option<Decimal>,
        longitude: Option<Decimal>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            province_id,
            name,
            code,
            latitude,
            longitude,
            created_at: now,
            updated_at: now,
        }
    }
}


impl crate::app::query_builder::Queryable for City {
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

// Implement the query builder service for City
crate::impl_query_builder_service!(City);