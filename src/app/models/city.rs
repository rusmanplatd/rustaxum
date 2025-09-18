use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use crate::query_builder::{Queryable, SortDirection};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct City {
    pub id: Ulid,
    pub province_id: Ulid,
    pub name: String,
    pub code: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCity {
    pub province_id: String,
    pub name: String,
    pub code: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCity {
    pub province_id: Option<String>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct CityResponse {
    pub id: String,
    pub province_id: String,
    pub name: String,
    pub code: Option<String>,
    pub latitude: Option<Decimal>,
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

impl FromRow<'_, PgRow> for City {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        let province_id_str: String = row.try_get("province_id")?;
        let province_id = Ulid::from_string(&province_id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "province_id".to_string(),
            source: Box::new(e),
        })?;

        Ok(City {
            id,
            province_id,
            name: row.try_get("name")?,
            code: row.try_get("code")?,
            latitude: row.try_get("latitude")?,
            longitude: row.try_get("longitude")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
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