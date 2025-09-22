use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use crate::app::query_builder::{SortDirection};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::oauth_personal_access_clients)]
pub struct PersonalAccessClient {
    pub id: String,
    pub client_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePersonalAccessClient {
    pub client_id: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::oauth_personal_access_clients)]
pub struct NewPersonalAccessClient {
    pub id: String,
    pub client_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PersonalAccessClientResponse {
    pub id: String,
    pub client_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PersonalAccessClient {
    pub fn new(client_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new().to_string(),
            client_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> PersonalAccessClientResponse {
        PersonalAccessClientResponse {
            id: self.id.clone(),
            client_id: self.client_id.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl NewPersonalAccessClient {
    pub fn new(client_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new().to_string(),
            client_id,
            created_at: now,
            updated_at: now,
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