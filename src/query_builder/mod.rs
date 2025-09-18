pub mod builder;
pub mod filter;
pub mod sort;
pub mod select;
pub mod params;

use serde::Serialize;
use sqlx::{FromRow, postgres::PgRow};
use std::collections::HashMap;

pub use builder::QueryBuilder;
pub use filter::{Filter, FilterOperator};
pub use sort::{Sort, SortDirection};
pub use select::FieldSelector;
pub use params::QueryParams;

/// Trait for models that can be queried with the QueryBuilder
pub trait Queryable: for<'r> FromRow<'r, PgRow> + Send + Unpin + Serialize {
    /// Get the table name for the model
    fn table_name() -> &'static str;

    /// Get the allowed filter fields for the model
    fn allowed_filters() -> Vec<&'static str>;

    /// Get the allowed sort fields for the model
    fn allowed_sorts() -> Vec<&'static str>;

    /// Get the allowed select fields for the model
    fn allowed_fields() -> Vec<&'static str>;

    /// Get the default sort field and direction
    fn default_sort() -> Option<(&'static str, SortDirection)> {
        None
    }
}

/// Main QueryBuilder struct that provides Laravel-like query building
#[derive(Debug, Clone)]
pub struct QueryBuilderRequest {
    pub filters: HashMap<String, String>,
    pub sorts: Vec<String>,
    pub fields: Option<Vec<String>>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

impl Default for QueryBuilderRequest {
    fn default() -> Self {
        Self {
            filters: HashMap::new(),
            sorts: Vec::new(),
            fields: None,
            page: None,
            per_page: Some(15), // Default pagination
        }
    }
}

/// Response wrapper for paginated results
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub current_page: u64,
    pub per_page: u64,
    pub total: u64,
    pub last_page: u64,
    pub from: Option<u64>,
    pub to: Option<u64>,
}

impl PaginationMeta {
    pub fn new(current_page: u64, per_page: u64, total: u64) -> Self {
        let last_page = if total == 0 { 1 } else { (total as f64 / per_page as f64).ceil() as u64 };
        let from = if total == 0 { None } else { Some((current_page - 1) * per_page + 1) };
        let to = if total == 0 { None } else { Some(std::cmp::min(current_page * per_page, total)) };

        Self {
            current_page,
            per_page,
            total,
            last_page,
            from,
            to,
        }
    }
}