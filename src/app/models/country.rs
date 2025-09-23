use super::DieselUlid;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::{SortDirection};

/// Country model representing a country entity
/// Contains country information including name, ISO code, and phone code
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::countries)]
pub struct Country {
    /// Unique identifier for the country
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    /// Country name
    #[schema(example = "United States")]
    pub name: String,
    /// ISO country code (2-3 letters)
    #[schema(example = "US")]
    pub iso_code: String,
    /// Optional phone country code
    #[schema(example = "+1")]
    pub phone_code: Option<String>,
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

/// Create country payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateCountry {
    pub name: String,
    pub iso_code: String,
    pub phone_code: Option<String>,
}

/// Insertable struct for countries
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::countries)]
pub struct NewCountry {
    pub id: DieselUlid,
    pub name: String,
    pub iso_code: String,
    pub phone_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by: Option<DieselUlid>,
    pub updated_by: Option<DieselUlid>,
    pub deleted_by: Option<DieselUlid>,
}

/// Update country payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateCountry {
    pub name: Option<String>,
    pub iso_code: Option<String>,
    pub phone_code: Option<String>,
}

/// Country response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct CountryResponse {
    pub id: DieselUlid,
    pub name: String,
    pub iso_code: String,
    pub phone_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NewCountry {
    pub fn new(name: String, iso_code: String, phone_code: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            name,
            iso_code,
            phone_code,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        }
    }
}

impl Country {
    pub fn new(name: String, iso_code: String, phone_code: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            name,
            iso_code,
            phone_code,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        }
    }

    pub fn to_response(&self) -> CountryResponse {
        CountryResponse {
            id: self.id,
            name: self.name.clone(),
            iso_code: self.iso_code.clone(),
            phone_code: self.phone_code.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}


impl crate::app::query_builder::Queryable for Country {
    fn table_name() -> &'static str {
        "countries"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "iso_code",
            "phone_code",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "iso_code",
            "phone_code",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "iso_code",
            "phone_code",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("name", SortDirection::Asc))
    }
}

// Implement the query builder service for Country
crate::impl_query_builder_service!(Country);

impl crate::app::models::HasModelType for Country {
    fn model_type() -> &'static str {
        "Country"
    }
}

impl crate::app::models::activity_log::HasId for Country {
    fn id(&self) -> String {
        self.id.to_string()
    }
}