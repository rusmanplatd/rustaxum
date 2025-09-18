use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};
use crate::query_builder::{Queryable, SortDirection};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Country {
    pub id: Ulid,
    pub name: String,
    pub iso_code: String,
    pub phone_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCountry {
    pub name: String,
    pub iso_code: String,
    pub phone_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCountry {
    pub name: Option<String>,
    pub iso_code: Option<String>,
    pub phone_code: Option<String>,
}

#[derive(Debug, Serialize)]
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