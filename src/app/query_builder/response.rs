use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use std::collections::HashMap;
use crate::app::query_builder::{PaginationResult, Filter, Sort, Include};
use crate::app::query_builder::builder::QueryInfo;
use crate::app::query_builder::pagination::PaginationInfo;

/// Comprehensive query response structure that includes data, pagination, and metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QueryResponse<T> {
    /// The actual data items
    pub data: Vec<T>,

    /// Pagination information
    pub pagination: PaginationInfo,

    /// Query metadata and execution statistics
    pub meta: QueryMeta,

    /// Links for navigation (HATEOAS support)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<ResponseLinks>,

    /// Included related data (for relationship includes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included: Option<HashMap<String, Vec<serde_json::Value>>>,
}

/// Query execution metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QueryMeta {
    /// Query execution time in milliseconds
    #[schema(example = 45)]
    pub execution_time_ms: u64,

    /// Number of filters applied
    #[schema(example = 3)]
    pub filters_applied: usize,

    /// Number of sorts applied
    #[schema(example = 2)]
    pub sorts_applied: usize,

    /// Number of includes applied
    #[schema(example = 1)]
    pub includes_applied: usize,

    /// Whether field selection was used
    #[schema(example = true)]
    pub field_selection_used: bool,

    /// Number of fields selected (if field selection was used)
    #[schema(example = 5)]
    pub fields_selected: Option<usize>,

    /// Query complexity score (0-100, higher = more complex)
    #[schema(example = 35)]
    pub complexity_score: u8,

    /// Cache status information
    #[schema(example = "miss")]
    pub cache_status: CacheStatus,

    /// API version
    #[schema(example = "1.0")]
    pub api_version: String,

    /// Request timestamp (ISO 8601)
    #[schema(example = "2023-01-01T12:00:00Z")]
    pub request_timestamp: String,

    /// Whether the query was optimized
    #[schema(example = true)]
    pub optimized: bool,

    /// Performance warnings if any
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

/// Cache status for query responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum CacheStatus {
    /// Cache hit - data served from cache
    Hit,
    /// Cache miss - data fetched from database
    Miss,
    /// Cache stale - served stale data while refreshing
    Stale,
    /// Cache disabled for this query
    Disabled,
}

/// HATEOAS links for API navigation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResponseLinks {
    /// Link to self (current page)
    #[serde(rename = "self")]
    pub self_link: Link,

    /// Link to first page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<Link>,

    /// Link to previous page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<Link>,

    /// Link to next page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<Link>,

    /// Link to last page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<Link>,

    /// Related resource links
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub related: HashMap<String, Link>,
}

/// Individual link structure
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Link {
    /// The URL for the link
    #[schema(example = "/api/users?page=2&per_page=15")]
    pub href: String,

    /// HTTP method for this link
    #[schema(example = "GET")]
    pub method: String,

    /// Media type expected/returned
    #[schema(example = "application/json")]
    pub type_: Option<String>,

    /// Human-readable title
    #[schema(example = "Next page")]
    pub title: Option<String>,
}

/// Applied query filters information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AppliedFilter {
    /// Field name being filtered
    pub field: String,

    /// Filter operator used
    pub operator: String,

    /// Filter value (serialized as string for display)
    pub value: String,

    /// Whether this filter was optimized (e.g., using index)
    pub optimized: bool,
}

/// Applied query sorts information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AppliedSort {
    /// Field name being sorted
    pub field: String,

    /// Sort direction
    pub direction: String,

    /// Sort priority (1 = primary sort, 2 = secondary, etc.)
    pub priority: u8,

    /// Whether this sort was optimized (e.g., using index)
    pub optimized: bool,
}

/// Applied query includes information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AppliedInclude {
    /// Relationship name
    pub relationship: String,

    /// Number of related records loaded
    pub loaded_count: usize,

    /// Whether this include was eagerly loaded
    pub eager_loaded: bool,

    /// Nested includes (for deep relationships)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub nested: Vec<AppliedInclude>,
}

/// Extended query response with debug information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DebugQueryResponse<T> {
    /// Standard query response
    #[serde(flatten)]
    pub response: QueryResponse<T>,

    /// Debug information (only included when debug mode is enabled)
    pub debug: DebugInfo,
}

/// Debug information for query analysis
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DebugInfo {
    /// Generated SQL query
    pub sql: String,

    /// SQL query parameters
    pub sql_params: Vec<serde_json::Value>,

    /// Database execution plan (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_plan: Option<String>,

    /// Applied filters with details
    pub applied_filters: Vec<AppliedFilter>,

    /// Applied sorts with details
    pub applied_sorts: Vec<AppliedSort>,

    /// Applied includes with details
    pub applied_includes: Vec<AppliedInclude>,

    /// Query optimization suggestions
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub optimization_suggestions: Vec<String>,

    /// Performance metrics
    pub performance: PerformanceMetrics,
}

/// Detailed performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PerformanceMetrics {
    /// Total execution time in microseconds
    pub total_execution_time_us: u64,

    /// Database query time in microseconds
    pub db_query_time_us: u64,

    /// Serialization time in microseconds
    pub serialization_time_us: u64,

    /// Memory usage in bytes
    pub memory_usage_bytes: u64,

    /// Number of database queries executed
    pub db_queries_count: u32,

    /// Cache operations performed
    pub cache_operations: u32,
}

/// Error response structure for failed queries
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QueryErrorResponse {
    /// Error message
    pub error: String,

    /// Detailed error description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Error code
    pub code: u32,

    /// Field-specific validation errors
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub errors: HashMap<String, Vec<String>>,

    /// Request context for debugging
    pub context: ErrorContext,
}

/// Error context information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorContext {
    /// Request timestamp
    pub timestamp: String,

    /// Request ID for tracing
    pub request_id: String,

    /// API endpoint
    pub endpoint: String,

    /// Query parameters that caused the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_params: Option<serde_json::Value>,
}

/// Success response wrapper for non-paginated responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DataResponse<T> {
    /// The data
    pub data: T,

    /// Response metadata
    pub meta: SimpleMeta,

    /// Links for navigation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<ResponseLinks>,
}

/// Simplified metadata for single item responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimpleMeta {
    /// Request timestamp
    pub timestamp: String,

    /// API version
    pub api_version: String,

    /// Response type
    pub type_: String,
}

impl<T> QueryResponse<T> {
    /// Create a new query response from a pagination result
    pub fn from_pagination_result(result: PaginationResult<T>) -> Self {
        Self {
            data: result.data,
            pagination: result.pagination,
            meta: QueryMeta::default(),
            links: None,
            included: None,
        }
    }

    /// Add metadata to the response
    pub fn with_meta(mut self, meta: QueryMeta) -> Self {
        self.meta = meta;
        self
    }

    /// Add HATEOAS links to the response
    pub fn with_links(mut self, links: ResponseLinks) -> Self {
        self.links = Some(links);
        self
    }

    /// Add included relationships data
    pub fn with_included(mut self, included: HashMap<String, Vec<serde_json::Value>>) -> Self {
        self.included = Some(included);
        self
    }

    /// Calculate and set complexity score based on query parameters
    pub fn calculate_complexity(&mut self, query_info: &QueryInfo) {
        let mut score = 0u8;

        // Base complexity
        score += 10;

        // Filters complexity
        score += (query_info.filters_count * 5).min(25) as u8;

        // Sorts complexity
        score += (query_info.sorts_count * 3).min(15) as u8;

        // Includes complexity
        score += (query_info.includes_count * 8).min(30) as u8;

        // Field selection reduces complexity
        if query_info.has_field_selection {
            score = score.saturating_sub(5);
        }

        // Pagination type affects complexity
        if let Some(pagination_type) = query_info.pagination_type {
            match pagination_type {
                "cursor" => score = score.saturating_sub(5),
                "offset" => score += 5,
                _ => {}
            }
        }

        self.meta.complexity_score = score.min(100);
    }

    /// Add performance warning
    pub fn add_warning(&mut self, warning: String) {
        self.meta.warnings.push(warning);
    }
}

impl QueryMeta {
    /// Create query metadata from query info and execution time
    pub fn from_query_info(query_info: &QueryInfo, execution_time_ms: u64) -> Self {
        Self {
            execution_time_ms,
            filters_applied: query_info.filters_count,
            sorts_applied: query_info.sorts_count,
            includes_applied: query_info.includes_count,
            field_selection_used: query_info.has_field_selection,
            fields_selected: if query_info.has_field_selection {
                Some(query_info.selected_fields_count)
            } else {
                None
            },
            complexity_score: 0, // Will be calculated
            cache_status: CacheStatus::Miss,
            api_version: "1.0".to_string(),
            request_timestamp: chrono::Utc::now().to_rfc3339(),
            optimized: false,
            warnings: Vec::new(),
        }
    }
}

impl Default for QueryMeta {
    fn default() -> Self {
        Self {
            execution_time_ms: 0,
            filters_applied: 0,
            sorts_applied: 0,
            includes_applied: 0,
            field_selection_used: false,
            fields_selected: None,
            complexity_score: 10,
            cache_status: CacheStatus::Miss,
            api_version: "1.0".to_string(),
            request_timestamp: chrono::Utc::now().to_rfc3339(),
            optimized: false,
            warnings: Vec::new(),
        }
    }
}

impl Link {
    /// Create a new link
    pub fn new(href: String, method: String) -> Self {
        Self {
            href,
            method,
            type_: Some("application/json".to_string()),
            title: None,
        }
    }

    /// Create a GET link
    pub fn get(href: String) -> Self {
        Self::new(href, "GET".to_string())
    }

    /// Create a POST link
    pub fn post(href: String) -> Self {
        Self::new(href, "POST".to_string())
    }

    /// Add a title to the link
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }
}

impl ResponseLinks {
    /// Create basic pagination links
    pub fn from_pagination(
        base_url: &str,
        current_page: u32,
        total_pages: u32,
        per_page: u32,
    ) -> Self {
        let mut links = Self {
            self_link: Link::get(format!("{}?page={}&per_page={}", base_url, current_page, per_page)),
            first: Some(Link::get(format!("{}?page=1&per_page={}", base_url, per_page))),
            prev: None,
            next: None,
            last: Some(Link::get(format!("{}?page={}&per_page={}", base_url, total_pages, per_page))),
            related: HashMap::new(),
        };

        if current_page > 1 {
            links.prev = Some(Link::get(format!("{}?page={}&per_page={}", base_url, current_page - 1, per_page)));
        }

        if current_page < total_pages {
            links.next = Some(Link::get(format!("{}?page={}&per_page={}", base_url, current_page + 1, per_page)));
        }

        links
    }

    /// Add a related resource link
    pub fn add_related(mut self, name: String, link: Link) -> Self {
        self.related.insert(name, link);
        self
    }
}

impl<T> DataResponse<T> {
    /// Create a new data response
    pub fn new(data: T) -> Self {
        Self {
            data,
            meta: SimpleMeta {
                timestamp: chrono::Utc::now().to_rfc3339(),
                api_version: "1.0".to_string(),
                type_: "single".to_string(),
            },
            links: None,
        }
    }

    /// Add links to the response
    pub fn with_links(mut self, links: ResponseLinks) -> Self {
        self.links = Some(links);
        self
    }
}

/// Convert filters to applied filter info
pub fn filters_to_applied(filters: &[Filter]) -> Vec<AppliedFilter> {
    filters.iter().map(|filter| AppliedFilter {
        field: filter.field.clone(),
        operator: filter.operator.to_string(),
        value: format!("{:?}", filter.value), // Simplified value representation
        optimized: false, // This would be determined by query execution analysis
    }).collect()
}

/// Convert sorts to applied sort info
pub fn sorts_to_applied(sorts: &[Sort]) -> Vec<AppliedSort> {
    sorts.iter().enumerate().map(|(index, sort)| AppliedSort {
        field: sort.field.clone(),
        direction: sort.direction.to_string(),
        priority: (index + 1) as u8,
        optimized: false, // This would be determined by query execution analysis
    }).collect()
}

/// Convert includes to applied include info
pub fn includes_to_applied(includes: &[Include]) -> Vec<AppliedInclude> {
    includes.iter().map(|include| AppliedInclude {
        relationship: include.relation.clone(),
        loaded_count: 0, // This would be populated after relationship loading
        eager_loaded: true, // This would be determined by the include strategy
        nested: Vec::new(), // This would be populated for nested includes
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::query_builder::PaginationType;
    use crate::app::query_builder::pagination::PaginationInfo;

    #[test]
    fn test_query_response_creation() {
        let pagination_info = PaginationInfo {
            pagination_type: PaginationType::Offset,
            current_page: Some(1),
            per_page: 10,
            total: Some(100),
            total_pages: Some(10),
            from: Some(1),
            to: Some(10),
            has_more_pages: true,
            prev_page: None,
            next_page: Some(2),
            prev_cursor: None,
            next_cursor: None,
            first_page_url: Some("?page=1&per_page=10".to_string()),
            last_page_url: Some("?page=10&per_page=10".to_string()),
            prev_page_url: None,
            next_page_url: Some("?page=2&per_page=10".to_string()),
            path: "/api/users".to_string(),
        };

        let pagination_result = PaginationResult {
            data: vec![1, 2, 3],
            pagination: pagination_info,
        };

        let response = QueryResponse::from_pagination_result(pagination_result);
        assert_eq!(response.data, vec![1, 2, 3]);
        assert_eq!(response.pagination.per_page, 10);
        assert_eq!(response.meta.complexity_score, 10);
    }

    #[test]
    fn test_link_creation() {
        let link = Link::get("/api/users".to_string())
            .with_title("Users List".to_string());

        assert_eq!(link.href, "/api/users");
        assert_eq!(link.method, "GET");
        assert_eq!(link.title, Some("Users List".to_string()));
    }

    #[test]
    fn test_response_links_pagination() {
        let links = ResponseLinks::from_pagination("/api/users", 2, 5, 10);

        assert!(links.prev.is_some());
        assert!(links.next.is_some());
        assert_eq!(links.first.unwrap().href, "/api/users?page=1&per_page=10");
        assert_eq!(links.last.unwrap().href, "/api/users?page=5&per_page=10");
    }

    #[test]
    fn test_cache_status_serialization() {
        let status = CacheStatus::Hit;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"hit\"");
    }

    #[test]
    fn test_data_response_creation() {
        let response = DataResponse::new("test_data".to_string());
        assert_eq!(response.data, "test_data");
        assert_eq!(response.meta.type_, "single");
        assert_eq!(response.meta.api_version, "1.0");
    }
}