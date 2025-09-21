use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use crate::app::query_builder::{Queryable, SortDirection};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub id: Ulid,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateScope {
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateScope {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ScopeResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Scope {
    pub fn new(name: String, description: Option<String>, is_default: bool) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            name,
            description,
            is_default,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> ScopeResponse {
        ScopeResponse {
            id: self.id.to_string(),
            name: self.name.clone(),
            description: self.description.clone(),
            is_default: self.is_default,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn is_wildcard(&self) -> bool {
        self.name == "*"
    }

    pub fn implies(&self, other: &str) -> bool {
        self.is_wildcard() || self.name == other
    }
}

impl crate::app::query_builder::Queryable for Scope {
    fn table_name() -> &'static str {
        "oauth_scopes"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "is_default",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "description",
            "is_default",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("name", SortDirection::Asc))
    }
}

// Implement the query builder service for Scope
crate::impl_query_builder_service!(Scope);