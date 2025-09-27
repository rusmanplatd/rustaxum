pub mod builder;
pub mod filter;
pub mod sort;
pub mod include;
pub mod pagination;
pub mod traits;
pub mod executor;
pub mod service;
pub mod response;
pub mod audit_loader;
pub mod role_permission_loader;

// Re-exports for convenient access
pub use builder::{QueryBuilder, QueryBuilderExt};
pub use filter::{Filter, FilterOperator, FilterValue};
pub use sort::{Sort, SortDirection};
pub use include::Include;
pub use pagination::{Pagination, PaginationResult, PaginationType};
pub use traits::{Queryable, Filterable, Sortable, Includable};
pub use executor::QueryExecutor;
pub use service::{QueryBuilderService, QueryService};
pub use response::{QueryResponse, QueryMeta, DataResponse, QueryErrorResponse, ResponseLinks, Link, CacheStatus};
pub use audit_loader::AuditRelationshipLoader;
pub use role_permission_loader::RolePermissionLoader;

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

    /// Parse filter parameters into structured Filter objects
    pub fn parse_filters(&self) -> Vec<Filter> {
        let mut filters = Vec::new();

        for (field_key, value) in &self.filter {
            // Handle Laravel-style nested filter syntax: filter[field][operator]=value
            if let Some(parsed_filter) = self.parse_nested_filter(field_key, value) {
                filters.push(parsed_filter);
            } else {
                // Handle simple filter syntax: filter[field]=value (defaults to eq)
                let filter = Filter::new(field_key.clone(), FilterOperator::Eq, FilterValue::single(value.clone()));
                filters.push(filter);
            }
        }

        filters
    }

    /// Parse nested filter syntax like filter[field][operator]=value
    fn parse_nested_filter(&self, field_key: &str, value: &serde_json::Value) -> Option<Filter> {
        // This would typically be handled by the HTTP query parser
        // For now, we'll handle simple cases and assume the field_key contains the operator info

        // Check if value is an object with operator keys
        if let Some(operators) = value.as_object() {
            for (operator_str, filter_value) in operators {
                if let Some(operator) = FilterOperator::from_string(operator_str) {
                    let filter_val = match operator {
                        FilterOperator::In | FilterOperator::NotIn => {
                            // Handle comma-separated values for IN operators
                            if let Some(val_str) = filter_value.as_str() {
                                let values: Vec<serde_json::Value> = val_str
                                    .split(',')
                                    .map(|s| serde_json::Value::String(s.trim().to_string()))
                                    .collect();
                                FilterValue::multiple(values)
                            } else {
                                FilterValue::single(filter_value.clone())
                            }
                        },
                        FilterOperator::Between => {
                            // Handle comma-separated range values
                            if let Some(val_str) = filter_value.as_str() {
                                let parts: Vec<&str> = val_str.split(',').collect();
                                if parts.len() == 2 {
                                    FilterValue::range(
                                        serde_json::Value::String(parts[0].trim().to_string()),
                                        serde_json::Value::String(parts[1].trim().to_string())
                                    )
                                } else {
                                    FilterValue::single(filter_value.clone())
                                }
                            } else {
                                FilterValue::single(filter_value.clone())
                            }
                        },
                        _ => FilterValue::single(filter_value.clone())
                    };

                    return Some(Filter::new(field_key.to_string(), operator, filter_val));
                }
            }
        }

        None
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

    /// Parse include relationships with validation and nested support
    pub fn parse_includes(&self, allowed_includes: &[&str]) -> Vec<Include> {
        let include_strings = self.get_includes();
        let mut includes = Vec::new();

        for include_str in include_strings {
            if self.is_valid_include(&include_str, allowed_includes) {
                includes.push(Include::new(include_str));
            } else {
                tracing::warn!("Invalid include relationship: {}", include_str);
            }
        }

        includes
    }

    /// Validate if an include relationship is allowed
    fn is_valid_include(&self, include_str: &str, allowed_includes: &[&str]) -> bool {
        // Check exact match
        if allowed_includes.contains(&include_str) {
            return true;
        }

        // Check nested relationships (e.g., "user.roles" where "user" is allowed)
        if include_str.contains('.') {
            let parts: Vec<&str> = include_str.split('.').collect();
            if let Some(root) = parts.first() {
                return allowed_includes.contains(root);
            }
        }

        false
    }

    /// Get advanced filtering options with operator support
    pub fn get_advanced_filters(&self, allowed_filters: &[&str]) -> Vec<Filter> {
        self.parse_filters()
            .into_iter()
            .filter(|filter| allowed_filters.contains(&filter.field.as_str()))
            .collect()
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