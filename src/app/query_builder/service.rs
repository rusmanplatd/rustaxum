use crate::app::query_builder::{QueryBuilder, QueryBuilderExt, QueryExecutor, QueryParams, Queryable, PaginationResult};
use crate::database::{DbPool};
use anyhow::Result;
use axum::extract::Query;

/// Service trait for models that can be queried using the query builder
/// This provides a high-level interface for controllers to use
pub trait QueryBuilderService<T>
where
    T: Queryable,
{
    /// Get paginated results using query parameters
    fn index(
        query_params: Query<QueryParams>,
        pool: &DbPool,
    ) -> Result<PaginationResult<serde_json::Value>> {
        let mut conn = pool.get()?;
        let builder = T::from_params(query_params.0)?;
        QueryExecutor::execute_paginated(builder, &mut conn)
    }

    /// Get all results (no pagination) using query parameters
    fn all(
        query_params: Query<QueryParams>,
        pool: &DbPool,
    ) -> Result<Vec<serde_json::Value>> {
        let mut conn = pool.get()?;
        let builder = T::from_params(query_params.0)?;
        QueryExecutor::execute_all(builder, &mut conn)
    }

    /// Get the first result using query parameters
    fn first(
        query_params: Query<QueryParams>,
        pool: &DbPool,
    ) -> Result<Option<serde_json::Value>> {
        let mut conn = pool.get()?;
        let builder = T::from_params(query_params.0)?;
        QueryExecutor::execute_first(builder, &mut conn)
    }

    /// Get count of results using query parameters
    fn count(
        query_params: Query<QueryParams>,
        pool: &DbPool,
    ) -> Result<i64> {
        let mut conn = pool.get()?;
        let builder = T::from_params(query_params.0)?;
        QueryExecutor::execute_count(builder, &mut conn)
    }

    /// Create a custom query builder
    fn query() -> QueryBuilder<T> {
        T::query()
    }

    /// Execute a custom query builder with pagination
    fn execute_paginated(
        builder: QueryBuilder<T>,
        pool: &DbPool,
    ) -> Result<PaginationResult<serde_json::Value>> {
        let mut conn = pool.get()?;
        QueryExecutor::execute_paginated(builder, &mut conn)
    }

    /// Execute a custom query builder without pagination
    fn execute_all(
        builder: QueryBuilder<T>,
        pool: &DbPool,
    ) -> Result<Vec<serde_json::Value>> {
        let mut conn = pool.get()?;
        QueryExecutor::execute_all(builder, &mut conn)
    }

    /// Execute a custom query builder and get the first result
    fn execute_first(
        builder: QueryBuilder<T>,
        pool: &DbPool,
    ) -> Result<Option<serde_json::Value>> {
        let mut conn = pool.get()?;
        QueryExecutor::execute_first(builder, &mut conn)
    }

    /// Execute a custom query builder and get the count
    fn execute_count(
        builder: QueryBuilder<T>,
        pool: &DbPool,
    ) -> Result<i64> {
        let mut conn = pool.get()?;
        QueryExecutor::execute_count(builder, &mut conn)
    }
}

/// Macro to automatically implement QueryBuilderService for a model
#[macro_export]
macro_rules! impl_query_builder_service {
    ($model:ty) => {
        impl crate::app::query_builder::service::QueryBuilderService<$model> for $model {}
    };
}

/// Service implementation for common query operations
pub struct QueryService;

impl QueryService {
    /// Execute a raw query with query parameters for any Queryable model
    pub fn query_with_params<T>(
        query_params: Query<QueryParams>,
        pool: &DbPool,
    ) -> Result<PaginationResult<serde_json::Value>>
    where
        T: Queryable,
    {
        let mut conn = pool.get()?;
        let builder = T::from_params(query_params.0)?;
        QueryExecutor::execute_paginated(builder, &mut conn)
    }

    /// Build a query for a specific model type
    pub fn for_model<T>() -> QueryBuilder<T>
    where
        T: Queryable,
    {
        T::query()
    }

    /// Execute any query builder
    pub fn execute<T>(
        builder: QueryBuilder<T>,
        pool: &DbPool,
    ) -> Result<PaginationResult<serde_json::Value>>
    where
        T: Queryable,
    {
        let mut conn = pool.get()?;
        QueryExecutor::execute_paginated(builder, &mut conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::query_builder::{SortDirection, Pagination};

    // Mock model for testing
    #[derive(Debug)]
    struct TestModel;

    impl Queryable for TestModel {
        fn table_name() -> &'static str {
            "test_models"
        }

        fn allowed_filters() -> Vec<&'static str> {
            vec!["id", "name", "email"]
        }

        fn allowed_sorts() -> Vec<&'static str> {
            vec!["id", "name", "created_at"]
        }

        fn allowed_fields() -> Vec<&'static str> {
            vec!["id", "name", "email", "created_at"]
        }

        fn default_sort() -> Option<(&'static str, SortDirection)> {
            Some(("created_at", SortDirection::Desc))
        }
    }

    impl_query_builder_service!(TestModel);

    #[test]
    fn test_query_service_for_model() {
        let builder = QueryService::for_model::<TestModel>();
        assert_eq!(builder.get_filters().len(), 0);
        assert_eq!(builder.get_sorts().len(), 0);
    }

    #[test]
    fn test_query_builder_service_query() {
        let builder = TestModel::query();
        assert_eq!(builder.get_filters().len(), 0);
        assert_eq!(builder.get_sorts().len(), 0);
    }
}