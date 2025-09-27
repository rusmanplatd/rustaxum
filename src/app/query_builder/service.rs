use crate::app::query_builder::{QueryBuilder, QueryBuilderExt, QueryExecutor, QueryParams, Queryable, Filterable, Sortable, Includable, PaginationResult};
use crate::database::{DbPool};
use anyhow::Result;
use axum::extract::Query;

/// Service trait for models that can be queried using the query builder
/// This provides a high-level interface for controllers to use
pub trait QueryBuilderService<T>
where
    T: Queryable + Clone,
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
        T: Queryable + Clone,
    {
        let mut conn = pool.get()?;
        let builder = T::from_params(query_params.0)?;
        QueryExecutor::execute_paginated(builder, &mut conn)
    }

    /// Build a query for a specific model type
    pub fn for_model<T>() -> QueryBuilder<T>
    where
        T: Queryable + Clone,
    {
        T::query()
    }

    /// Execute any query builder
    pub fn execute<T>(
        builder: QueryBuilder<T>,
        pool: &DbPool,
    ) -> Result<PaginationResult<serde_json::Value>>
    where
        T: Queryable + Clone,
    {
        let mut conn = pool.get()?;
        QueryExecutor::execute_paginated(builder, &mut conn)
    }

    /// Build an advanced query with filtering, sorting, and includes for enhanced models
    pub fn advanced_query<T>(
        query_params: Query<QueryParams>,
        pool: &DbPool,
    ) -> Result<PaginationResult<serde_json::Value>>
    where
        T: Queryable + Filterable + Sortable + Includable + Clone,
    {
        let mut conn = pool.get()?;
        let params = query_params.0;

        // Build advanced query using enhanced traits
        let mut builder = T::query();

        // Apply advanced filters using the new Filterable trait
        let advanced_filters = params.get_advanced_filters(&T::allowed_filters());
        for filter in advanced_filters {
            builder = builder.filter(filter);
        }

        // Apply multi-column sorting using the new Sortable trait
        if let Some(ref sort_str) = params.sort {
            builder = builder.order_by_string(sort_str.clone());
        }

        // Apply relationship includes using the new Includable trait
        let validated_includes = params.parse_includes(&T::allowed_includes());
        for include in validated_includes {
            builder = builder.include(include);
        }

        // Apply field selection if specified
        if let Some(fields) = params.get_fields(&T::table_name()) {
            builder = builder.select(fields);
        }

        // Apply pagination
        let pagination = params.get_pagination();
        builder = builder.paginate(pagination);

        QueryExecutor::execute_paginated(builder, &mut conn)
    }

    /// Validate and apply filters with enhanced error handling
    pub fn validate_and_filter<T>(
        builder: QueryBuilder<T>,
        filters: &[crate::app::query_builder::Filter],
        allowed_filters: &[&str],
    ) -> QueryBuilder<T>
    where
        T: Queryable + Filterable + Clone,
    {
        let mut query_builder = builder;

        for filter in filters {
            if allowed_filters.contains(&filter.field.as_str()) {
                query_builder = query_builder.filter(filter.clone());
            } else {
                tracing::warn!("Attempted to filter on disallowed field: {}", filter.field);
            }
        }

        query_builder
    }

    /// Apply advanced sorting with validation
    pub fn validate_and_sort<T>(
        builder: QueryBuilder<T>,
        sorts: &[(String, crate::app::query_builder::SortDirection)],
        allowed_sorts: &[&str],
    ) -> QueryBuilder<T>
    where
        T: Queryable + Sortable + Clone,
    {
        let mut query_builder = builder;

        // Validate sorts using the Sortable trait
        let validated_sorts = T::apply_validated_sort(sorts, allowed_sorts);
        if !validated_sorts.is_empty() {
            // Apply multi-column sorting
            let sort_tuples = crate::app::query_builder::Sort::vec_to_tuples(
                &crate::app::query_builder::Sort::from_tuples(sorts)
            );
            for (field, direction) in sort_tuples {
                query_builder = match direction {
                    crate::app::query_builder::SortDirection::Asc => query_builder.order_by(field),
                    crate::app::query_builder::SortDirection::Desc => query_builder.order_by_desc(field),
                };
            }
        }

        query_builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::query_builder::{SortDirection, Pagination};

    // Mock model for testing
    #[derive(Debug, Clone)]
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

    // Enhanced test model implementing all new traits
    impl crate::app::query_builder::Filterable for TestModel {
        fn apply_basic_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
            match (column, operator) {
                ("name", "contains") => {
                    format!("LOWER({}) LIKE LOWER('%{}%')", column, value.as_str().unwrap_or("").replace('\'', "''"))
                }
                ("name", "starts_with") => {
                    format!("LOWER({}) LIKE LOWER('{}%')", column, value.as_str().unwrap_or("").replace('\'', "''"))
                }
                ("name", "ends_with") => {
                    format!("LOWER({}) LIKE LOWER('%{}')", column, value.as_str().unwrap_or("").replace('\'', "''"))
                }
                _ => {
                    match value {
                        serde_json::Value::String(s) => format!("{} {} '{}'", column, operator, s.replace('\'', "''")),
                        serde_json::Value::Number(n) => format!("{} {} {}", column, operator, n),
                        serde_json::Value::Bool(b) => format!("{} {} {}", column, operator, b),
                        serde_json::Value::Null => format!("{} IS NULL", column),
                        _ => format!("{} {} '{}'", column, operator, value.to_string().replace('\'', "''"))
                    }
                }
            }
        }
    }

    impl crate::app::query_builder::Sortable for TestModel {
        fn apply_basic_sort(column: &str, direction: &str) -> String {
            format!("{} {}", column, direction)
        }
    }

    impl crate::app::query_builder::Includable for TestModel {
        fn load_relationships(_ids: &[String], _includes: &[String], _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<()> {
            Ok(())
        }

        fn load_relationship(_ids: &[String], relationship: &str, _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<serde_json::Value> {
            match relationship {
                "profile" => Ok(serde_json::json!({})),
                "posts" => Ok(serde_json::json!([])),
                _ => Ok(serde_json::json!({}))
            }
        }

        fn build_join_clause(relationship: &str, main_table: &str) -> Option<String> {
            match relationship {
                "profile" => Some(format!("LEFT JOIN profiles ON {}.id = profiles.user_id", main_table)),
                "posts" => Some(format!("LEFT JOIN posts ON {}.id = posts.user_id", main_table)),
                _ => None
            }
        }

        fn get_foreign_key(relationship: &str) -> Option<String> {
            match relationship {
                "profile" => Some("user_id".to_string()),
                "posts" => Some("user_id".to_string()),
                _ => None
            }
        }
    }

    #[test]
    fn test_query_service_for_model() {
        let builder = QueryService::for_model::<TestModel>();
        assert_eq!(builder.get_filters().len(), 0);
        assert_eq!(builder.get_sorts().len(), 0);
    }

    #[test]
    fn test_query_builder_service_query() {
        let builder = <TestModel as QueryBuilderExt>::query();
        assert_eq!(builder.get_filters().len(), 0);
        assert_eq!(builder.get_sorts().len(), 0);
    }

    #[test]
    fn test_enhanced_filterable_trait() {
        use crate::app::query_builder::Filterable;

        // Test contains filter
        let sql = TestModel::apply_basic_filter("name", "contains", &serde_json::json!("john"));
        assert_eq!(sql, "LOWER(name) LIKE LOWER('%john%')");

        // Test starts_with filter
        let sql = TestModel::apply_basic_filter("name", "starts_with", &serde_json::json!("test"));
        assert_eq!(sql, "LOWER(name) LIKE LOWER('test%')");

        // Test ends_with filter
        let sql = TestModel::apply_basic_filter("name", "ends_with", &serde_json::json!("ing"));
        assert_eq!(sql, "LOWER(name) LIKE LOWER('%ing')");

        // Test basic equality with different value types
        let sql = TestModel::apply_basic_filter("id", "=", &serde_json::json!(123));
        assert_eq!(sql, "id = 123");

        let sql = TestModel::apply_basic_filter("active", "=", &serde_json::json!(true));
        assert_eq!(sql, "active = true");

        let sql = TestModel::apply_basic_filter("deleted_at", "=", &serde_json::json!(null));
        assert_eq!(sql, "deleted_at IS NULL");
    }

    #[test]
    fn test_enhanced_sortable_trait() {
        use crate::app::query_builder::Sortable;

        // Test basic sorting
        let sql = TestModel::apply_basic_sort("name", "ASC");
        assert_eq!(sql, "name ASC");

        let sql = TestModel::apply_basic_sort("created_at", "DESC");
        assert_eq!(sql, "created_at DESC");

        // Test multi-column sorting
        let sorts = vec![
            ("name".to_string(), SortDirection::Asc),
            ("created_at".to_string(), SortDirection::Desc)
        ];
        let sql = TestModel::apply_multi_sort(&sorts);
        assert_eq!(sql, "name ASC, created_at DESC");

        // Test sort validation
        let allowed_sorts = vec!["name", "created_at"];
        let sorts = vec![
            ("name".to_string(), SortDirection::Asc),
            ("invalid_field".to_string(), SortDirection::Desc),
            ("created_at".to_string(), SortDirection::Asc)
        ];
        let sql = TestModel::apply_validated_sort(&sorts, &allowed_sorts);
        assert_eq!(sql, "name ASC, created_at ASC");
    }

    #[test]
    fn test_enhanced_includable_trait() {
        use crate::app::query_builder::Includable;

        // Test join clause building
        let join = TestModel::build_join_clause("profile", "test_models").unwrap();
        assert_eq!(join, "LEFT JOIN profiles ON test_models.id = profiles.user_id");

        let join = TestModel::build_join_clause("posts", "test_models").unwrap();
        assert_eq!(join, "LEFT JOIN posts ON test_models.id = posts.user_id");

        // Test foreign key resolution
        let fk = TestModel::get_foreign_key("profile").unwrap();
        assert_eq!(fk, "user_id");

        let fk = TestModel::get_foreign_key("posts").unwrap();
        assert_eq!(fk, "user_id");

        // Test relationship query building
        let parent_ids = vec!["id1".to_string(), "id2".to_string()];
        let query = TestModel::build_relationship_query("profile", &parent_ids).unwrap();
        assert!(query.contains("SELECT * FROM profile"));
        assert!(query.contains("WHERE user_id IN ('id1', 'id2')"));
    }

    #[test]
    fn test_parse_sort_string() {
        use crate::app::query_builder::Sortable;

        // Test simple ascending sort
        let sorts = TestModel::parse_sort_string("name");
        assert_eq!(sorts.len(), 1);
        assert_eq!(sorts[0].0, "name");
        assert_eq!(sorts[0].1, SortDirection::Asc);

        // Test descending with minus prefix
        let sorts = TestModel::parse_sort_string("-created_at");
        assert_eq!(sorts.len(), 1);
        assert_eq!(sorts[0].0, "created_at");
        assert_eq!(sorts[0].1, SortDirection::Desc);

        // Test colon syntax
        let sorts = TestModel::parse_sort_string("name:desc");
        assert_eq!(sorts.len(), 1);
        assert_eq!(sorts[0].0, "name");
        assert_eq!(sorts[0].1, SortDirection::Desc);

        // Test multi-column sorting
        let sorts = TestModel::parse_sort_string("name,-created_at,id:asc");
        assert_eq!(sorts.len(), 3);
        assert_eq!(sorts[0], ("name".to_string(), SortDirection::Asc));
        assert_eq!(sorts[1], ("created_at".to_string(), SortDirection::Desc));
        assert_eq!(sorts[2], ("id".to_string(), SortDirection::Asc));
    }

    #[test]
    fn test_validate_includes() {
        use crate::app::query_builder::Includable;

        let includes = vec!["profile".to_string(), "posts".to_string(), "invalid".to_string()];
        let allowed_includes = vec!["profile", "posts"];

        let validated = TestModel::validate_includes(&includes, &allowed_includes);
        assert_eq!(validated.len(), 2);
        assert!(validated.contains(&"profile".to_string()));
        assert!(validated.contains(&"posts".to_string()));
        assert!(!validated.contains(&"invalid".to_string()));
    }

    #[test]
    fn test_parse_nested_includes() {
        use crate::app::query_builder::Includable;

        let nested = TestModel::parse_nested_includes("user.profile.avatar,posts.comments");
        assert_eq!(nested.len(), 2);
        assert_eq!(nested[0], vec!["user".to_string(), "profile".to_string(), "avatar".to_string()]);
        assert_eq!(nested[1], vec!["posts".to_string(), "comments".to_string()]);
    }
}