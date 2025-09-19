use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::query_builder::{Queryable, SortDirection};

/// Country model representing a country entity
/// Contains country information including name, ISO code, and phone code
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Country {
    /// Unique identifier for the country
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: Ulid,
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
}

/// Create country payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateCountry {
    pub name: String,
    pub iso_code: String,
    pub phone_code: Option<String>,
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
    pub id: String,
    pub name: String,
    pub iso_code: String,
    pub phone_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Country {
    pub fn new(name: String, iso_code: String, phone_code: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            name,
            iso_code,
            phone_code,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> CountryResponse {
        CountryResponse {
            id: self.id.to_string(),
            name: self.name.clone(),
            iso_code: self.iso_code.clone(),
            phone_code: self.phone_code.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl FromRow<'_, PgRow> for Country {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        Ok(Country {
            id,
            name: row.try_get("name")?,
            iso_code: row.try_get("iso_code")?,
            phone_code: row.try_get("phone_code")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl Queryable for Country {
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