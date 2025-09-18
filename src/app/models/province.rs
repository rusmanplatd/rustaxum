use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};
use crate::query_builder::{Queryable, SortDirection};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Province {
    pub id: Ulid,
    pub country_id: Ulid,
    pub name: String,
    pub code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProvince {
    pub country_id: String,
    pub name: String,
    pub code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProvince {
    pub country_id: Option<String>,
    pub name: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Serialize)]
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

impl FromRow<'_, PgRow> for Province {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        let country_id_str: String = row.try_get("country_id")?;
        let country_id = Ulid::from_string(&country_id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "country_id".to_string(),
            source: Box::new(e),
        })?;

        Ok(Province {
            id,
            country_id,
            name: row.try_get("name")?,
            code: row.try_get("code")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl Queryable for Province {
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