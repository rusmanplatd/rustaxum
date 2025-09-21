use crate::app::query_builder::{
    QueryBuilder, QueryBuilderService, QueryParams, PaginationType,
    Filter, Sort, SortDirection
};
use crate::app::models::user::User;
use crate::database::DbPool;
use axum::extract::Query;

/// Example usage of the Spatie-like Query Builder
pub struct QueryBuilderExamples;

impl QueryBuilderExamples {
    /// Example 1: Using QueryParams from HTTP request
    ///
    /// URL examples:
    /// - Basic: `/api/users?page=1&per_page=10`
    /// - With filters: `/api/users?filter[name]=John&filter[age][gte]=18`
    /// - With sorting: `/api/users?sort=name,-created_at`
    /// - With includes: `/api/users?include=roles,organization`
    /// - With cursor pagination: `/api/users?pagination_type=cursor&per_page=20`
    /// - With offset pagination: `/api/users?pagination_type=offset&page=2&per_page=15`
    /// - With field selection: `/api/users?fields[users]=id,name,email`
    pub async fn from_http_params(
        query_params: Query<QueryParams>,
        pool: &DbPool,
    ) -> anyhow::Result<crate::app::query_builder::PaginationResult<serde_json::Value>> {
        // Use the service trait for easy querying
        User::index(query_params, pool)
    }

    /// Example 2: Building queries programmatically
    pub async fn programmatic_query(
        pool: &DbPool,
    ) -> anyhow::Result<crate::app::query_builder::PaginationResult<serde_json::Value>> {
        let builder = User::query()
            .where_eq("active", true)
            .where_gte("age", 18)
            .where_in("role", vec!["admin", "editor"])
            .where_like("name", "%John%")
            .order_by_desc("created_at")
            .order_by("name")
            .with("roles")
            .with("organization")
            .select(vec!["id", "name", "email", "created_at"])
            .paginate(crate::app::query_builder::Pagination::cursor(20, None));

        User::execute_paginated(builder, pool)
    }

    /// Example 3: Using different pagination types
    pub async fn pagination_examples(
        pool: &DbPool,
    ) -> anyhow::Result<()> {
        // Offset-based pagination (traditional)
        let offset_builder = User::query()
            .where_eq("active", true)
            .paginate(crate::app::query_builder::Pagination::offset(1, 15));

        let _offset_results = User::execute_paginated(offset_builder, pool)?;

        // Cursor-based pagination (better for performance)
        let cursor_builder = User::query()
            .where_eq("active", true)
            .paginate(crate::app::query_builder::Pagination::cursor(15, None));

        let _cursor_results = User::execute_paginated(cursor_builder, pool)?;

        Ok(())
    }

    /// Example 4: Complex filtering with different operators
    pub async fn advanced_filtering(
        pool: &DbPool,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let builder = User::query()
            .where_eq("status", "active")
            .where_ne("role", "guest")
            .where_gte("login_count", 5)
            .where_lt("failed_attempts", 3)
            .where_between("created_at", "2023-01-01", "2023-12-31")
            .where_not_null("email_verified_at")
            .where_null("deleted_at")
            .where_ilike("email", "%@company.com");

        User::execute_all(builder, pool)
    }

    /// Example 5: Sorting examples
    pub async fn sorting_examples(
        pool: &DbPool,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let builder = User::query()
            .order_by("name")                    // Ascending by name
            .order_by_desc("created_at")         // Descending by created_at
            .sort(Sort::asc("email"))           // Another way to add ascending sort
            .sort(Sort::desc("last_login_at")); // Another way to add descending sort

        User::execute_all(builder, pool)
    }

    /// Example 6: Building custom filters
    pub fn custom_filters() -> Vec<Filter> {
        vec![
            Filter::eq("active", true),
            Filter::gte("age", 18),
            Filter::in_values("role", vec!["admin", "editor", "author"]),
            Filter::like("name", "%John%"),
            Filter::between("created_at", "2023-01-01", "2023-12-31"),
            Filter::is_not_null("email_verified_at"),
        ]
    }

    /// Example 7: Using the query builder with custom logic
    pub async fn custom_logic(
        pool: &DbPool,
        user_role: &str,
        search_term: Option<&str>,
    ) -> anyhow::Result<crate::app::query_builder::PaginationResult<serde_json::Value>> {
        let mut builder = User::query()
            .where_eq("active", true)
            .order_by_desc("last_login_at");

        // Add role-based filtering
        if user_role != "admin" {
            builder = builder.where_ne("role", "admin");
        }

        // Add search functionality
        if let Some(search) = search_term {
            builder = builder
                .where_ilike("name", format!("%{}%", search))
                .where_ilike("email", format!("%{}%", search));
        }

        // Different pagination based on role
        let pagination = if user_role == "admin" {
            crate::app::query_builder::Pagination::offset(1, 50) // Admins can see more
        } else {
            crate::app::query_builder::Pagination::cursor(20, None) // Regular users get cursor pagination
        };

        builder = builder.paginate(pagination);

        User::execute_paginated(builder, pool)
    }

    /// Example 8: Count queries
    pub async fn count_examples(
        pool: &DbPool,
    ) -> anyhow::Result<i64> {
        let builder = User::query()
            .where_eq("active", true)
            .where_gte("age", 18);

        User::execute_count(builder, pool)
    }

    /// Example 9: First result queries
    pub async fn first_result(
        pool: &DbPool,
        email: &str,
    ) -> anyhow::Result<Option<serde_json::Value>> {
        let builder = User::query()
            .where_eq("email", email)
            .where_eq("active", true);

        User::execute_first(builder, pool)
    }
}

/// URL Parameter Examples:
///
/// Basic usage:
/// ```
/// GET /api/users
/// GET /api/users?page=2&per_page=25
/// ```
///
/// Filtering:
/// ```
/// GET /api/users?filter[name]=John
/// GET /api/users?filter[age][gte]=18
/// GET /api/users?filter[role][in]=admin,editor
/// GET /api/users?filter[email][like]=%@company.com
/// GET /api/users?filter[created_at][between]=2023-01-01,2023-12-31
/// ```
///
/// Sorting:
/// ```
/// GET /api/users?sort=name
/// GET /api/users?sort=-created_at
/// GET /api/users?sort=name,-created_at,email
/// ```
///
/// Including relationships:
/// ```
/// GET /api/users?include=roles
/// GET /api/users?include=roles,organization
/// GET /api/users?include=organization.positions
/// ```
///
/// Field selection:
/// ```
/// GET /api/users?fields[users]=id,name,email
/// GET /api/users?fields[users]=id,name&fields[roles]=name,permissions
/// ```
///
/// Pagination types:
/// ```
/// GET /api/users?pagination_type=offset&page=2&per_page=15
/// GET /api/users?pagination_type=cursor&per_page=20
/// GET /api/users?pagination_type=cursor&per_page=20&cursor=some_cursor_value
/// ```
///
/// Combined example:
/// ```
/// GET /api/users?filter[active]=true&filter[age][gte]=18&sort=-created_at,name&include=roles&fields[users]=id,name,email&pagination_type=cursor&per_page=25
/// ```
pub struct _Documentation;