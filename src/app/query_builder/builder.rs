use crate::app::query_builder::{
    Filter, Sort, Include, Pagination, QueryParams, Queryable,
};
use anyhow::{Result, anyhow};
use std::collections::HashMap;

/// Main query builder for constructing and executing database queries
/// This is the main entry point for the Spatie-like query builder functionality
#[derive(Debug, Clone)]
pub struct QueryBuilder<T>
where
    T: Queryable,
{
    /// Applied filters
    filters: Vec<Filter>,
    /// Applied sorts
    sorts: Vec<Sort>,
    /// Applied includes
    includes: Vec<Include>,
    /// Selected fields
    fields: Option<Vec<String>>,
    /// Pagination settings
    pagination: Option<Pagination>,
    /// Custom appends
    appends: HashMap<String, String>,
    /// Whether to use soft deletes
    with_trashed: bool,
    /// Whether to only show trashed records
    only_trashed: bool,

    _phantom: std::marker::PhantomData<T>,
}

impl<T> QueryBuilder<T>
where
    T: Queryable,
{
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            sorts: Vec::new(),
            includes: Vec::new(),
            fields: None,
            pagination: None,
            appends: HashMap::new(),
            with_trashed: false,
            only_trashed: false,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a query builder from query parameters
    pub fn from_params(params: QueryParams) -> Result<Self> {
        let mut builder = Self::new();

        // Apply filters
        let filters = Filter::from_params(&params.filter);
        for filter in filters {
            if T::is_filter_allowed(&filter.field) {
                builder = builder.filter(filter);
            }
        }

        // Apply sorts
        let sorts = params.get_sorts();
        for sort in sorts {
            if T::is_sort_allowed(&sort.field) {
                builder = builder.sort(sort);
            }
        }

        // Apply includes
        let includes = params.get_includes();
        for include in includes {
            if T::is_include_allowed(&include) {
                builder = builder.include(Include::new(include));
            }
        }

        // Apply field selection
        if let Some(fields) = params.get_fields(T::table_name()) {
            let allowed_fields: Vec<String> = fields
                .into_iter()
                .filter(|f| T::is_field_allowed(f))
                .collect();
            if !allowed_fields.is_empty() {
                builder = builder.fields(allowed_fields);
            }
        }

        // Apply pagination
        builder = builder.paginate(params.get_pagination());

        // Apply appends
        builder.appends = params.append;

        Ok(builder)
    }

    /// Add a filter to the query
    pub fn filter(mut self, filter: Filter) -> Self {
        if T::is_filter_allowed(&filter.field) {
            self.filters.push(filter);
        }
        self
    }

    /// Add multiple filters
    pub fn filters(mut self, filters: Vec<Filter>) -> Self {
        for filter in filters {
            if T::is_filter_allowed(&filter.field) {
                self.filters.push(filter);
            }
        }
        self
    }

    /// Add a where equals filter
    pub fn where_eq(self, field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.filter(Filter::eq(field, value))
    }

    /// Add a where not equals filter
    pub fn where_ne(self, field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.filter(Filter::ne(field, value))
    }

    /// Add a where greater than filter
    pub fn where_gt(self, field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.filter(Filter::gt(field, value))
    }

    /// Add a where greater than or equal filter
    pub fn where_gte(self, field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.filter(Filter::gte(field, value))
    }

    /// Add a where less than filter
    pub fn where_lt(self, field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.filter(Filter::lt(field, value))
    }

    /// Add a where less than or equal filter
    pub fn where_lte(self, field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.filter(Filter::lte(field, value))
    }

    /// Add a where like filter
    pub fn where_like(self, field: impl Into<String>, pattern: impl Into<serde_json::Value>) -> Self {
        self.filter(Filter::like(field, pattern))
    }

    /// Add a where ilike filter
    pub fn where_ilike(self, field: impl Into<String>, pattern: impl Into<serde_json::Value>) -> Self {
        self.filter(Filter::ilike(field, pattern))
    }

    /// Add a where in filter
    pub fn where_in(self, field: impl Into<String>, values: Vec<impl Into<serde_json::Value>>) -> Self {
        self.filter(Filter::in_values(field, values))
    }

    /// Add a where not in filter
    pub fn where_not_in(self, field: impl Into<String>, values: Vec<impl Into<serde_json::Value>>) -> Self {
        self.filter(Filter::not_in(field, values))
    }

    /// Add a where null filter
    pub fn where_null(self, field: impl Into<String>) -> Self {
        self.filter(Filter::is_null(field))
    }

    /// Add a where not null filter
    pub fn where_not_null(self, field: impl Into<String>) -> Self {
        self.filter(Filter::is_not_null(field))
    }

    /// Add a where between filter
    pub fn where_between(
        self,
        field: impl Into<String>,
        start: impl Into<serde_json::Value>,
        end: impl Into<serde_json::Value>
    ) -> Self {
        self.filter(Filter::between(field, start, end))
    }

    /// Add a sort to the query
    pub fn sort(mut self, sort: Sort) -> Self {
        if T::is_sort_allowed(&sort.field) {
            self.sorts.push(sort);
        }
        self
    }

    /// Add multiple sorts
    pub fn sorts(mut self, sorts: Vec<Sort>) -> Self {
        for sort in sorts {
            if T::is_sort_allowed(&sort.field) {
                self.sorts.push(sort);
            }
        }
        self
    }

    /// Add an ascending sort
    pub fn order_by(self, field: impl Into<String>) -> Self {
        self.sort(Sort::asc(field))
    }

    /// Add a descending sort
    pub fn order_by_desc(self, field: impl Into<String>) -> Self {
        self.sort(Sort::desc(field))
    }

    /// Add an include relationship
    pub fn include(mut self, include: Include) -> Self {
        if T::is_include_allowed(&include.relation) {
            self.includes.push(include);
        }
        self
    }

    /// Add multiple includes
    pub fn includes(mut self, includes: Vec<Include>) -> Self {
        for include in includes {
            if T::is_include_allowed(&include.relation) {
                self.includes.push(include);
            }
        }
        self
    }

    /// Add a simple include by relation name
    pub fn with(self, relation: impl Into<String>) -> Self {
        self.include(Include::new(relation))
    }

    /// Set fields to select
    pub fn fields(mut self, fields: Vec<String>) -> Self {
        let allowed_fields: Vec<String> = fields
            .into_iter()
            .filter(|f| T::is_field_allowed(f))
            .collect();

        if !allowed_fields.is_empty() {
            self.fields = Some(allowed_fields);
        }
        self
    }

    /// Select only specific fields
    pub fn select(self, fields: Vec<impl Into<String>>) -> Self {
        let field_strings: Vec<String> = fields.into_iter().map(|f| f.into()).collect();
        self.fields(field_strings)
    }

    /// Set pagination
    pub fn paginate(mut self, pagination: Pagination) -> Self {
        self.pagination = Some(pagination);
        self
    }

    /// Set cursor-based pagination
    pub fn cursor_paginate(mut self, per_page: u32, cursor: Option<String>) -> Self {
        self.pagination = Some(Pagination::cursor(per_page, cursor));
        self
    }

    /// Set offset-based pagination (traditional page/per_page)
    pub fn offset_paginate(mut self, page: u32, per_page: u32) -> Self {
        self.pagination = Some(Pagination::page_based(page, per_page));
        self
    }

    /// Set page size for current pagination type
    pub fn per_page(mut self, per_page: u32) -> Self {
        if let Some(ref pagination) = self.pagination {
            match pagination.pagination_type {
                crate::app::query_builder::PaginationType::Cursor => {
                    self.pagination = Some(Pagination::cursor(per_page, pagination.cursor.clone()));
                }
                crate::app::query_builder::PaginationType::Offset => {
                    self.pagination = Some(Pagination::page_based(pagination.page, per_page));
                }
            }
        } else {
            // Default to cursor pagination if no pagination is set
            self.pagination = Some(Pagination::cursor(per_page, None));
        }
        self
    }

    /// Set page number (only valid for offset pagination)
    pub fn page(mut self, page: u32) -> Self {
        if let Some(ref pagination) = self.pagination {
            match pagination.pagination_type {
                crate::app::query_builder::PaginationType::Offset => {
                    self.pagination = Some(Pagination::page_based(page, pagination.per_page));
                }
                crate::app::query_builder::PaginationType::Cursor => {
                    // For cursor pagination, we ignore page numbers
                    // This maintains the existing cursor-based setup
                }
            }
        } else {
            // Default to offset pagination when setting a page
            self.pagination = Some(Pagination::page_based(page, 15));
        }
        self
    }

    /// Set cursor (only valid for cursor pagination)
    pub fn cursor(mut self, cursor: Option<String>) -> Self {
        if let Some(ref pagination) = self.pagination {
            match pagination.pagination_type {
                crate::app::query_builder::PaginationType::Cursor => {
                    self.pagination = Some(Pagination::cursor(pagination.per_page, cursor));
                }
                crate::app::query_builder::PaginationType::Offset => {
                    // For offset pagination, we ignore cursors
                    // This maintains the existing offset-based setup
                }
            }
        } else {
            // Default to cursor pagination when setting a cursor
            self.pagination = Some(Pagination::cursor(15, cursor));
        }
        self
    }

    /// Include soft deleted records
    pub fn with_trashed(mut self) -> Self {
        self.with_trashed = true;
        self
    }

    /// Only show soft deleted records
    pub fn only_trashed(mut self) -> Self {
        self.only_trashed = true;
        self
    }

    /// Add an append field
    pub fn append(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.appends.insert(key.into(), value.into());
        self
    }

    /// Get the current filters
    pub fn get_filters(&self) -> &[Filter] {
        &self.filters
    }

    /// Get the current sorts
    pub fn get_sorts(&self) -> &[Sort] {
        &self.sorts
    }

    /// Get the current includes
    pub fn get_includes(&self) -> &[Include] {
        &self.includes
    }

    /// Get the current fields
    pub fn get_fields(&self) -> Option<&[String]> {
        self.fields.as_deref()
    }

    /// Get the current pagination
    pub fn get_pagination(&self) -> Option<&Pagination> {
        self.pagination.as_ref()
    }

    /// Check if using cursor-based pagination
    pub fn is_cursor_pagination(&self) -> bool {
        self.pagination.as_ref().map_or(false, |p| p.is_cursor())
    }

    /// Check if using offset-based pagination
    pub fn is_offset_pagination(&self) -> bool {
        self.pagination.as_ref().map_or(false, |p| p.is_offset())
    }

    /// Get SQL LIMIT clause value
    pub fn get_limit(&self) -> u32 {
        self.pagination.as_ref().map_or(15, |p| p.limit())
    }

    /// Get SQL OFFSET clause value (only for offset pagination)
    pub fn get_offset(&self) -> u32 {
        self.pagination.as_ref().map_or(0, |p| p.offset())
    }

    /// Get cursor WHERE clause for SQL queries (only for cursor pagination)
    pub fn get_cursor_where(&self) -> Option<(String, Vec<i64>)> {
        self.pagination.as_ref().and_then(|p| p.cursor_where_clause())
    }

    /// Count total records without pagination
    pub fn count(&self) -> Self {
        let mut builder = self.clone();
        builder.pagination = None;
        builder
    }

    /// Add a raw where clause (use with caution)
    pub fn where_raw(self, sql: impl Into<String>, bindings: Vec<serde_json::Value>) -> Self {
        self.filter(Filter::raw(sql, bindings))
    }

    /// Add a where clause that checks if a field exists (NOT NULL)
    pub fn where_exists(self, field: impl Into<String>) -> Self {
        self.filter(Filter::is_not_null(field))
    }

    /// Add a where clause that checks if a field doesn't exist (IS NULL)
    pub fn where_not_exists(self, field: impl Into<String>) -> Self {
        self.filter(Filter::is_null(field))
    }

    /// Add a where clause with OR logic (for next filter)
    pub fn or_where(self, field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.filter(Filter::or_eq(field, value))
    }

    /// Add multiple filters with AND logic
    pub fn where_all(mut self, conditions: Vec<(String, serde_json::Value)>) -> Self {
        for (field, value) in conditions {
            self = self.where_eq(field, value);
        }
        self
    }

    /// Add multiple filters with OR logic
    pub fn where_any(mut self, conditions: Vec<(String, serde_json::Value)>) -> Self {
        for (field, value) in conditions {
            self = self.or_where(field, value);
        }
        self
    }

    /// Add a where date filter (matches specific date)
    pub fn where_date(self, field: impl Into<String>, date: impl Into<String>) -> Self {
        self.filter(Filter::date(field, date))
    }

    /// Add a where year filter
    pub fn where_year(self, field: impl Into<String>, year: i32) -> Self {
        self.filter(Filter::year(field, year))
    }

    /// Add a where month filter
    pub fn where_month(self, field: impl Into<String>, month: u32) -> Self {
        self.filter(Filter::month(field, month))
    }

    /// Add a where day filter
    pub fn where_day(self, field: impl Into<String>, day: u32) -> Self {
        self.filter(Filter::day(field, day))
    }

    /// Add a search filter across multiple fields
    pub fn search(mut self, term: impl Into<String>, fields: Vec<String>) -> Self {
        let search_term = term.into();
        for field in fields {
            if T::is_filter_allowed(&field) {
                self = self.or_where(field, format!("%{}%", search_term));
            }
        }
        self
    }

    /// Add a JSON field filter (for JSONB columns)
    pub fn where_json(self, field: impl Into<String>, path: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.filter(Filter::json(field, path, value))
    }

    /// Add a relationship exists filter
    pub fn has(self, relation: impl Into<String>) -> Self {
        self.filter(Filter::has_relation(relation))
    }

    /// Add a relationship doesn't exist filter
    pub fn doesnt_have(self, relation: impl Into<String>) -> Self {
        self.filter(Filter::doesnt_have_relation(relation))
    }

    /// Add a relationship count filter
    pub fn has_count(self, relation: impl Into<String>, operator: impl Into<String>, count: u32) -> Self {
        self.filter(Filter::has_relation_count(relation, operator, count))
    }

    /// Clone the current query builder
    pub fn clone_query(&self) -> Self {
        self.clone()
    }

    /// Reset all filters
    pub fn reset_filters(mut self) -> Self {
        self.filters.clear();
        self
    }

    /// Reset all sorts
    pub fn reset_sorts(mut self) -> Self {
        self.sorts.clear();
        self
    }

    /// Reset pagination
    pub fn reset_pagination(mut self) -> Self {
        self.pagination = None;
        self
    }

    /// Get a copy of the query builder without pagination (useful for counting)
    pub fn without_pagination(&self) -> Self {
        let mut builder = self.clone();
        builder.pagination = None;
        builder
    }

    /// Get a copy of the query builder without sorting (useful for specific operations)
    pub fn without_sorting(&self) -> Self {
        let mut builder = self.clone();
        builder.sorts.clear();
        builder
    }

    /// Apply a closure to conditionally modify the query
    pub fn when<F>(self, condition: bool, callback: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        if condition {
            callback(self)
        } else {
            self
        }
    }

    /// Apply a closure to conditionally modify the query with an option
    pub fn when_some<T2, F>(self, option: Option<T2>, callback: F) -> Self
    where
        F: FnOnce(Self, T2) -> Self,
    {
        match option {
            Some(value) => callback(self, value),
            None => self,
        }
    }

    /// Get query statistics and metadata
    pub fn get_query_info(&self) -> QueryInfo {
        QueryInfo {
            filters_count: self.filters.len(),
            sorts_count: self.sorts.len(),
            includes_count: self.includes.len(),
            has_pagination: self.pagination.is_some(),
            pagination_type: self.pagination.as_ref().map(|p| p.pagination_type.as_str()),
            has_field_selection: self.fields.is_some(),
            selected_fields_count: self.fields.as_ref().map(|f| f.len()).unwrap_or(0),
            with_trashed: self.with_trashed,
            only_trashed: self.only_trashed,
        }
    }

    /// Clear all filters
    pub fn clear_filters(mut self) -> Self {
        self.filters.clear();
        self
    }

    /// Clear all sorts
    pub fn clear_sorts(mut self) -> Self {
        self.sorts.clear();
        self
    }

    /// Clear all includes
    pub fn clear_includes(mut self) -> Self {
        self.includes.clear();
        self
    }

    /// Reset to default state
    pub fn reset(self) -> Self {
        Self::new()
    }

    /// Apply default sort if no sorts are specified
    pub fn apply_default_sort(mut self) -> Self {
        if self.sorts.is_empty() {
            if let Some((field, direction)) = T::default_sort() {
                self.sorts.push(Sort::new(field.to_string(), direction));
            }
        }
        self
    }

    /// Apply default fields if no fields are specified
    pub fn apply_default_fields(mut self) -> Self {
        if self.fields.is_none() {
            let default_fields: Vec<String> = T::default_fields()
                .into_iter()
                .map(|f| f.to_string())
                .collect();
            self.fields = Some(default_fields);
        }
        self
    }

    /// Clone the query builder
    pub fn clone_builder(&self) -> Self {
        QueryBuilder {
            filters: self.filters.clone(),
            sorts: self.sorts.clone(),
            includes: self.includes.clone(),
            fields: self.fields.clone(),
            pagination: self.pagination.clone(),
            appends: self.appends.clone(),
            with_trashed: self.with_trashed,
            only_trashed: self.only_trashed,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Query metadata and statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct QueryInfo {
    pub filters_count: usize,
    pub sorts_count: usize,
    pub includes_count: usize,
    pub has_pagination: bool,
    pub pagination_type: Option<&'static str>,
    pub has_field_selection: bool,
    pub selected_fields_count: usize,
    pub with_trashed: bool,
    pub only_trashed: bool,
}

impl<T> Default for QueryBuilder<T>
where
    T: Queryable,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for easy query builder creation
pub trait QueryBuilderExt: Queryable + Sized {
    /// Create a new query builder for this model
    fn query() -> QueryBuilder<Self> {
        QueryBuilder::new()
    }

    /// Create a query builder from query parameters
    fn from_params(params: QueryParams) -> Result<QueryBuilder<Self>> {
        QueryBuilder::from_params(params)
    }
}

// Automatically implement QueryBuilderExt for all Queryable types
impl<T> QueryBuilderExt for T where T: Queryable {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::query_builder::{SortDirection, FilterOperator};

    // Mock model for testing
    #[derive(Debug)]
    struct TestModel;

    impl Queryable for TestModel {
        fn table_name() -> &'static str {
            "test_models"
        }

        fn allowed_filters() -> Vec<&'static str> {
            vec!["id", "name", "email", "created_at"]
        }

        fn allowed_sorts() -> Vec<&'static str> {
            vec!["id", "name", "email", "created_at"]
        }

        fn allowed_fields() -> Vec<&'static str> {
            vec!["id", "name", "email", "created_at"]
        }

        fn allowed_includes() -> Vec<&'static str> {
            vec!["organization", "roles"]
        }

        fn default_sort() -> Option<(&'static str, SortDirection)> {
            Some(("created_at", SortDirection::Desc))
        }
    }

    #[test]
    fn test_query_builder_creation() {
        let builder = TestModel::query();
        assert_eq!(builder.get_filters().len(), 0);
        assert_eq!(builder.get_sorts().len(), 0);
        assert_eq!(builder.get_includes().len(), 0);
    }

    #[test]
    fn test_query_builder_filters() {
        let builder = TestModel::query()
            .where_eq("name", "John")
            .where_gt("id", 10)
            .where_in("email", vec!["john@example.com", "jane@example.com"]);

        assert_eq!(builder.get_filters().len(), 3);
        assert_eq!(builder.get_filters()[0].field, "name");
        assert_eq!(builder.get_filters()[0].operator, FilterOperator::Eq);
    }

    #[test]
    fn test_query_builder_sorts() {
        let builder = TestModel::query()
            .order_by("name")
            .order_by_desc("created_at");

        assert_eq!(builder.get_sorts().len(), 2);
        assert_eq!(builder.get_sorts()[0].field, "name");
        assert_eq!(builder.get_sorts()[0].direction, SortDirection::Asc);
        assert_eq!(builder.get_sorts()[1].field, "created_at");
        assert_eq!(builder.get_sorts()[1].direction, SortDirection::Desc);
    }

    #[test]
    fn test_query_builder_includes() {
        let builder = TestModel::query()
            .with("organization")
            .with("roles");

        assert_eq!(builder.get_includes().len(), 2);
        assert_eq!(builder.get_includes()[0].relation, "organization");
        assert_eq!(builder.get_includes()[1].relation, "roles");
    }

    #[test]
    fn test_query_builder_pagination() {
        let builder = TestModel::query()
            .page(2)
            .per_page(25);

        let pagination = builder.get_pagination().unwrap();
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.per_page, 25);
    }

    #[test]
    fn test_query_builder_fields() {
        let builder = TestModel::query()
            .select(vec!["id", "name", "email"]);

        let fields = builder.get_fields().unwrap();
        assert_eq!(fields.len(), 3);
        assert!(fields.contains(&"id".to_string()));
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"email".to_string()));
    }

    #[test]
    fn test_query_builder_default_sort() {
        let builder = TestModel::query().apply_default_sort();
        assert_eq!(builder.get_sorts().len(), 1);
        assert_eq!(builder.get_sorts()[0].field, "created_at");
        assert_eq!(builder.get_sorts()[0].direction, SortDirection::Desc);
    }

    #[test]
    fn test_cursor_pagination() {
        let builder = TestModel::query()
            .cursor_paginate(20, Some("test_cursor".to_string()));

        assert!(builder.is_cursor_pagination());
        assert!(!builder.is_offset_pagination());
        assert_eq!(builder.get_limit(), 20);

        let pagination = builder.get_pagination().unwrap();
        assert_eq!(pagination.pagination_type, crate::app::query_builder::PaginationType::Cursor);
        assert_eq!(pagination.per_page, 20);
        assert_eq!(pagination.cursor, Some("test_cursor".to_string()));
    }

    #[test]
    fn test_offset_pagination() {
        let builder = TestModel::query()
            .offset_paginate(3, 25);

        assert!(!builder.is_cursor_pagination());
        assert!(builder.is_offset_pagination());
        assert_eq!(builder.get_limit(), 25);
        assert_eq!(builder.get_offset(), 50); // (3-1) * 25 = 50

        let pagination = builder.get_pagination().unwrap();
        assert_eq!(pagination.pagination_type, crate::app::query_builder::PaginationType::Offset);
        assert_eq!(pagination.page, 3);
        assert_eq!(pagination.per_page, 25);
    }

    #[test]
    fn test_pagination_type_switching() {
        // Start with cursor pagination
        let mut builder = TestModel::query()
            .cursor_paginate(15, None);
        assert!(builder.is_cursor_pagination());

        // Switch to offset pagination
        builder = builder.offset_paginate(2, 20);
        assert!(builder.is_offset_pagination());
        assert_eq!(builder.get_offset(), 20);

        // Switch back to cursor
        builder = builder.cursor_paginate(10, Some("new_cursor".to_string()));
        assert!(builder.is_cursor_pagination());
    }

    #[test]
    fn test_per_page_with_different_pagination_types() {
        // Test with cursor pagination
        let cursor_builder = TestModel::query()
            .cursor_paginate(15, None)
            .per_page(30);
        assert!(cursor_builder.is_cursor_pagination());
        assert_eq!(cursor_builder.get_limit(), 30);

        // Test with offset pagination
        let offset_builder = TestModel::query()
            .offset_paginate(2, 15)
            .per_page(30);
        assert!(offset_builder.is_offset_pagination());
        assert_eq!(offset_builder.get_limit(), 30);
        assert_eq!(offset_builder.get_offset(), 30); // (2-1) * 30 = 30
    }

    #[test]
    fn test_cursor_method() {
        let builder = TestModel::query()
            .cursor_paginate(20, None)
            .cursor(Some("my_cursor".to_string()));

        let pagination = builder.get_pagination().unwrap();
        assert_eq!(pagination.cursor, Some("my_cursor".to_string()));
    }

    #[test]
    fn test_page_method_with_cursor_pagination() {
        // When using cursor pagination, page() should be ignored
        let builder = TestModel::query()
            .cursor_paginate(20, Some("cursor".to_string()))
            .page(5);

        // Should still be cursor pagination and ignore the page setting
        assert!(builder.is_cursor_pagination());
        let pagination = builder.get_pagination().unwrap();
        assert_eq!(pagination.pagination_type, crate::app::query_builder::PaginationType::Cursor);
    }

    #[test]
    fn test_cursor_method_with_offset_pagination() {
        // When using offset pagination, cursor() should be ignored
        let builder = TestModel::query()
            .offset_paginate(2, 20)
            .cursor(Some("cursor".to_string()));

        // Should still be offset pagination and ignore the cursor setting
        assert!(builder.is_offset_pagination());
        let pagination = builder.get_pagination().unwrap();
        assert_eq!(pagination.pagination_type, crate::app::query_builder::PaginationType::Offset);
    }
}