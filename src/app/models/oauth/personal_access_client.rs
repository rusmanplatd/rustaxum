use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use crate::app::query_builder::{Queryable, SortDirection};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalAccessClient {
    pub id: Ulid,
    pub client_id: Ulid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePersonalAccessClient {
    pub client_id: Ulid,
}

#[derive(Debug, Serialize)]
pub struct PersonalAccessClientResponse {
    pub id: String,
    pub client_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PersonalAccessClient {
    pub fn new(client_id: Ulid) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            client_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> PersonalAccessClientResponse {
        PersonalAccessClientResponse {
            id: self.id.to_string(),
            client_id: self.client_id.to_string(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl crate::app::query_builder::Queryable for PersonalAccessClient {
    fn table_name() -> &'static str {
        "oauth_personal_access_clients"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "client_id",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "client_id",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "client",
        ]
    }
}

// Implement the query builder service for PersonalAccessClient
crate::impl_query_builder_service!(PersonalAccessClient);