use crate::app::query_builder::{
    Filter, Sort, Include, Pagination, QueryParams, Queryable,
};
use anyhow::Result;
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

    /// Set page size
    pub fn per_page(mut self, per_page: u32) -> Self {
        let current_page = self.pagination.as_ref().map(|p| p.page).unwrap_or(1);
        self.pagination = Some(Pagination::new(current_page, per_page));
        self
    }

    /// Set page number
    pub fn page(mut self, page: u32) -> Self {
        let current_per_page = self.pagination.as_ref().map(|p| p.per_page).unwrap_or(15);
        self.pagination = Some(Pagination::new(page, current_per_page));
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
}