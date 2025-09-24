pub mod builder;
pub mod filter;
pub mod sort;
pub mod include;
pub mod pagination;
pub mod traits;
pub mod executor;
pub mod service;

// Re-exports for convenient access
pub use builder::{QueryBuilder, QueryBuilderExt};
pub use filter::{Filter, FilterOperator, FilterValue};
pub use sort::{Sort, SortDirection};
pub use include::Include;
pub use pagination::{Pagination, PaginationResult, PaginationType};
pub use traits::{Queryable, Filterable, Sortable, Includable};
pub use executor::QueryExecutor;
pub use service::{QueryBuilderService, QueryService};

use serde::Deserialize;
use std::collections::HashMap;
use axum::extract::Query;

/// Query parameters that can be passed to the query builder
#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct QueryParams {
    /// Filter parameters (e.g., ?filter[name]=John&filter[age][gte]=18)
    #[serde(default)]
    pub filter: HashMap<String, serde_json::Value>,

    /// Sort parameters (e.g., ?sort=name,-created_at)
    pub sort: Option<String>,

    /// Include relationships (e.g., ?include=user,organization)
    pub include: Option<String>,

    /// Fields to select (e.g., ?fields[users]=name,email&fields[organization]=name)
    #[serde(default)]
    pub fields: HashMap<String, String>,

    /// Pagination - page number
    pub page: Option<u32>,

    /// Pagination - items per page
    #[serde(rename = "per_page")]
    pub per_page: Option<u32>,

    /// Pagination type - offset or cursor (default: cursor)
    #[serde(rename = "pagination_type")]
    pub pagination_type: Option<PaginationType>,

    /// Cursor for cursor-based pagination
    pub cursor: Option<String>,

    /// Custom append parameters
    #[serde(default)]
    pub append: HashMap<String, String>,
}

impl Default for QueryParams {
    fn default() -> Self {
        Self {
            filter: HashMap::new(),
            sort: None,
            include: None,
            fields: HashMap::new(),
            page: Some(1),
            per_page: Some(15),
            pagination_type: Some(PaginationType::default()),
            cursor: None,
            append: HashMap::new(),
        }
    }
}

impl QueryParams {
    /// Create a new QueryParams instance from HTTP query parameters
    pub fn from_query(query: Query<QueryParams>) -> Self {
        query.0
    }

    /// Get filter value for a specific field
    pub fn get_filter(&self, field: &str) -> Option<&serde_json::Value> {
        self.filter.get(field)
    }

    /// Get sort fields as a vector of Sort structs
    pub fn get_sorts(&self) -> Vec<Sort> {
        match &self.sort {
            Some(sort_string) => Sort::from_string(sort_string),
            None => vec![],
        }
    }

    /// Get include relationships as a vector of strings
    pub fn get_includes(&self) -> Vec<String> {
        match &self.include {
            Some(include_string) => include_string
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            None => vec![],
        }
    }

    /// Get fields for a specific resource type
    pub fn get_fields(&self, resource: &str) -> Option<Vec<String>> {
        self.fields.get(resource).map(|fields| {
            fields
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
    }

    /// Get pagination info
    pub fn get_pagination(&self) -> Pagination {
        let pagination_type = self.pagination_type.unwrap_or_default();
        match pagination_type {
            PaginationType::Offset => Pagination::page_based(
                self.page.unwrap_or(1),
                self.per_page.unwrap_or(15),
            ),
            PaginationType::Cursor => Pagination::cursor(
                self.per_page.unwrap_or(15),
                self.cursor.clone(),
            ),
        }
    }
}