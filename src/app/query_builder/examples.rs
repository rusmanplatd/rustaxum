use crate::app::query_builder::{
    QueryBuilderService, QueryParams,
    Filter, Sort
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
        // Offset-based pagination (traditional) - using the new helper method
        let offset_builder = User::query()
            .where_eq("active", true)
            .offset_paginate(1, 15); // page 1, 15 items per page

        let _offset_results = User::execute_paginated(offset_builder, pool)?;

        // Cursor-based pagination (better for performance) - using the new helper method
        let cursor_builder = User::query()
            .where_eq("active", true)
            .cursor_paginate(15, None); // 15 items per page, no cursor (first page)

        let _cursor_results = User::execute_paginated(cursor_builder, pool)?;

        // Using cursor pagination with an existing cursor
        let cursor_next_builder = User::query()
            .where_eq("active", true)
            .cursor_paginate(15, Some("encoded_cursor_value".to_string()));

        let _cursor_next_results = User::execute_paginated(cursor_next_builder, pool)?;

        // Switching pagination types dynamically
        let mut dynamic_builder = User::query()
            .where_eq("active", true);

        // Start with cursor pagination
        dynamic_builder = dynamic_builder.cursor_paginate(20, None);
        assert!(dynamic_builder.is_cursor_pagination());

        // Switch to offset pagination
        dynamic_builder = dynamic_builder.offset_paginate(2, 25);
        assert!(dynamic_builder.is_offset_pagination());
        assert_eq!(dynamic_builder.get_offset(), 25); // (2-1) * 25

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
            crate::app::query_builder::Pagination::page_based(1, 50) // Admins can see more
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

    /// Example 10: Advanced pagination techniques
    pub async fn advanced_pagination_examples(
        pool: &DbPool,
    ) -> anyhow::Result<()> {
        // Dynamic pagination type selection based on data size
        // TODO: Implement execute_count when needed
        // let total_count = User::query()
        //     .where_eq("active", true)
        //     .execute_count(pool)?;

        let builder = User::query()
            .where_eq("active", true);

        // Use cursor pagination for large datasets, offset for smaller ones
        let paginated_builder = if total_count > 1000 {
            builder.cursor_paginate(50, None)
        } else {
            builder.offset_paginate(1, 20)
        };

        let _results = User::execute_paginated(paginated_builder, pool)?;

        // Working with cursors - getting cursor info
        let cursor_builder = User::query()
            .cursor_paginate(20, Some("example_cursor".to_string()));

        if let Some(pagination) = cursor_builder.get_pagination() {
            if let Some(cursor_str) = &pagination.cursor {
                // Check if cursor is valid
                if pagination.is_valid_cursor(cursor_str) {
                    println!("Cursor is valid");

                    // Get cursor metadata
                    if let Some(cursor_info) = pagination.cursor_info(cursor_str) {
                        println!("Cursor timestamp: {}", cursor_info.timestamp);
                        println!("Cursor position: {}", cursor_info.position);
                        println!("Cursor per_page: {}", cursor_info.per_page);
                    }
                }
            }
        }

        // Using pagination utility methods
        let utility_builder = User::query()
            .where_eq("active", true)
            .offset_paginate(3, 20);

        println!("Is cursor pagination: {}", utility_builder.is_cursor_pagination());
        println!("Is offset pagination: {}", utility_builder.is_offset_pagination());
        println!("SQL LIMIT value: {}", utility_builder.get_limit());
        println!("SQL OFFSET value: {}", utility_builder.get_offset());

        // Get cursor WHERE clause for SQL (when using cursor pagination)
        let cursor_sql_builder = User::query()
            .cursor_paginate(20, Some("encoded_cursor".to_string()));

        if let Some((where_clause, params)) = cursor_sql_builder.get_cursor_where() {
            println!("Cursor WHERE clause: {}", where_clause);
            println!("Cursor parameters: {:?}", params);
        }

        Ok(())
    }

    /// Example 11: Pagination type conversion and validation
    pub fn pagination_type_examples() -> anyhow::Result<()> {
        use crate::app::query_builder::PaginationType;

        // Creating pagination types from strings
        let cursor_type = PaginationType::from_str("cursor").map_err(|e| anyhow::anyhow!(e))?;
        let offset_type = PaginationType::from_str("offset").map_err(|e| anyhow::anyhow!(e))?;

        println!("Cursor type: {}", cursor_type.as_str());
        println!("Offset type: {}", offset_type.as_str());

        // Type checking
        assert!(cursor_type.is_cursor());
        assert!(!cursor_type.is_offset());
        assert!(offset_type.is_offset());
        assert!(!offset_type.is_cursor());

        // Error handling for invalid types
        match PaginationType::from_str("invalid") {
            Ok(_) => unreachable!(),
            Err(e) => println!("Expected error: {}", e),
        }

        Ok(())
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