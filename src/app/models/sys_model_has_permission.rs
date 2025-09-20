use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::query_builder::Queryable;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SysModelHasPermission {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: Ulid,
    #[schema(example = "User")]
    pub model_type: String,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub model_id: Ulid,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub permission_id: Ulid,
    #[schema(example = "organization")]
    pub scope_type: Option<String>,
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub scope_id: Option<Ulid>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSysModelHasPermission {
    pub model_type: String,
    pub model_id: Ulid,
    pub permission_id: Ulid,
    pub scope_type: Option<String>,
    pub scope_id: Option<Ulid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateSysModelHasPermission {
    pub model_type: Option<String>,
    pub model_id: Option<Ulid>,
    pub permission_id: Option<Ulid>,
    pub scope_type: Option<String>,
    pub scope_id: Option<Ulid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SysModelHasPermissionResponse {
    pub id: Ulid,
    pub model_type: String,
    pub model_id: Ulid,
    pub permission_id: Ulid,
    pub scope_type: Option<String>,
    pub scope_id: Option<Ulid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, PgRow> for SysModelHasPermission {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let model_id_str: String = row.try_get("model_id")?;
        let permission_id_str: String = row.try_get("permission_id")?;
        let scope_id_str: Option<String> = row.try_get("scope_id")?;

        Ok(Self {
            id: Ulid::from_string(&id_str).map_err(|_| sqlx::Error::Decode("Invalid ULID for id".into()))?,
            model_type: row.try_get("model_type")?,
            model_id: Ulid::from_string(&model_id_str).map_err(|_| sqlx::Error::Decode("Invalid ULID for model_id".into()))?,
            permission_id: Ulid::from_string(&permission_id_str).map_err(|_| sqlx::Error::Decode("Invalid ULID for permission_id".into()))?,
            scope_type: row.try_get("scope_type")?,
            scope_id: scope_id_str.map(|s| Ulid::from_string(&s)).transpose().map_err(|_| sqlx::Error::Decode("Invalid ULID for scope_id".into()))?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl Queryable for SysModelHasPermission {
    fn table_name() -> &'static str {
        "sys_model_has_permissions"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "model_type",
            "model_id",
            "permission_id",
            "scope_type",
            "scope_id",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "model_type",
            "scope_type",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "model_type",
            "model_id",
            "permission_id",
            "scope_type",
            "scope_id",
            "created_at",
            "updated_at",
        ]
    }
}

impl SysModelHasPermission {
    pub fn new(model_type: String, model_id: Ulid, permission_id: Ulid, scope_type: Option<String>, scope_id: Option<Ulid>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            model_type,
            model_id,
            permission_id,
            scope_type,
            scope_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> SysModelHasPermissionResponse {
        SysModelHasPermissionResponse {
            id: self.id,
            model_type: self.model_type.clone(),
            model_id: self.model_id,
            permission_id: self.permission_id,
            scope_type: self.scope_type.clone(),
            scope_id: self.scope_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}