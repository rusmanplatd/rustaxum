pub mod builder;
pub mod filter;
pub mod sort;
pub mod select;
pub mod params;
pub mod includes;
pub mod filter_group;
pub mod bind;
pub mod scopes;
pub mod cache;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

pub use builder::QueryBuilder;
pub use filter::{Filter, FilterOperator};
pub use sort::{Sort, SortDirection};
pub use select::FieldSelector;
pub use params::QueryParams;
pub use includes::{IncludeSelector, Relatable, Relationship, RelationshipType, WithRelationships};
pub use filter_group::{FilterGroup, FilterCondition, FilterGroupBuilder, SimpleFilter};
pub use scopes::{Scopeable, CommonScopes, ScopeResolver, ScopedQueryBuilder};

/// Pagination type for controlling how pagination works
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PaginationType {
    Offset,
    Cursor,
}

impl Default for PaginationType {
    fn default() -> Self {
        Self::Cursor
    }
}

/// Trait for models that can be queried with the QueryBuilder
// TODO: Convert to Diesel traits - temporarily using simpler trait during migration
pub trait Queryable: Send + Unpin + Serialize {
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
    pub filter_groups: Option<filter_group::FilterGroup>,
    pub sorts: Vec<String>,
    pub fields: Option<Vec<String>>,
    pub includes: Option<Vec<String>>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub pagination_type: PaginationType,
    pub cursor: Option<String>,
}

impl Default for QueryBuilderRequest {
    fn default() -> Self {
        Self {
            filters: HashMap::new(),
            filter_groups: None,
            sorts: Vec::new(),
            fields: None,
            includes: None,
            page: None,
            per_page: Some(15), // Default pagination
            pagination_type: PaginationType::default(),
            cursor: None,
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
    pub pagination_type: PaginationType,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
}

impl PaginationMeta {
    pub fn new(current_page: u64, per_page: u64, total: u64) -> Self {
        Self::offset_pagination(current_page, per_page, total)
    }

    pub fn offset_pagination(current_page: u64, per_page: u64, total: u64) -> Self {
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
            pagination_type: PaginationType::Offset,
            next_cursor: None,
            prev_cursor: None,
        }
    }

    pub fn cursor_pagination(per_page: u64, _has_more: bool, next_cursor: Option<String>, prev_cursor: Option<String>) -> Self {
        Self {
            current_page: 1, // Not applicable for cursor pagination
            per_page,
            total: 0, // Not applicable for cursor pagination
            last_page: 1, // Not applicable for cursor pagination
            from: None,
            to: None,
            pagination_type: PaginationType::Cursor,
            next_cursor,
            prev_cursor,
        }
    }
}