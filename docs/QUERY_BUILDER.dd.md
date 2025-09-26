# Query Builder System

The Query Builder is a comprehensive, Laravel-inspired data querying system for Rust applications built with Axum and Diesel. It provides a fluent, type-safe API for constructing database queries with filtering, sorting, pagination, field selection, and relationship management.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Core Components](#core-components)
- [Usage](#usage)
- [Filtering](#filtering)
- [Sorting](#sorting)
- [Pagination](#pagination)
- [Field Selection](#field-selection)
- [Relationships](#relationships)
- [Query Execution](#query-execution)
- [Service Integration](#service-integration)
- [Examples](#examples)
- [Performance Considerations](#performance-considerations)
- [Security Features](#security-features)
- [Testing](#testing)
- [Best Practices](#best-practices)

## Overview

The Query Builder system allows developers to construct complex database queries using a fluent, chainable API similar to Laravel's Eloquent ORM. It supports both HTTP query parameter parsing and programmatic query construction.

### Key Features

- **Fluent API**: Chainable methods for building queries
- **Type Safety**: Compile-time checks for field and operation validity
- **HTTP Integration**: Automatic parsing of URL query parameters
- **Multiple Pagination Types**: Cursor-based and offset-based pagination
- **Advanced Filtering**: Support for all common SQL operators
- **Field Selection**: Choose specific fields to return
- **Relationship Loading**: Include related data in responses
- **Security**: Built-in protection against SQL injection
- **Performance**: Optimized queries with intelligent caching

## Architecture

The Query Builder system consists of several interconnected components:

```
QueryParams -> QueryBuilder -> QueryExecutor -> Database
     |              |               |
     v              v               v
  Filter         Sort         PaginationResult
  Include       Pagination
  Fields
```

### Core Modules

- **`mod.rs`**: Main entry point and re-exports
- **`traits.rs`**: Core traits defining queryable behavior
- **`builder.rs`**: Main QueryBuilder implementation
- **`filter.rs`**: Filtering logic and operators
- **`sort.rs`**: Sorting specifications
- **`pagination.rs`**: Pagination configurations
- **`include.rs`**: Relationship inclusion
- **`executor.rs`**: SQL query execution
- **`service.rs`**: High-level service interface

## Core Components

### QueryParams

HTTP query parameters are automatically parsed into a structured format:

```rust
#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct QueryParams {
    pub filter: HashMap<String, serde_json::Value>,
    pub sort: Option<String>,
    pub include: Option<String>,
    pub fields: HashMap<String, String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub pagination_type: Option<PaginationType>,
    pub cursor: Option<String>,
    pub append: HashMap<String, String>,
}
```

### Queryable Trait

Models must implement the `Queryable` trait to be used with the Query Builder:

```rust
pub trait Queryable: Sized {
    fn table_name() -> &'static str;
    fn allowed_filters() -> Vec<&'static str>;
    fn allowed_sorts() -> Vec<&'static str>;
    fn allowed_fields() -> Vec<&'static str>;
    fn allowed_includes() -> Vec<&'static str>;
    fn default_sort() -> Option<(&'static str, SortDirection)>;
    fn default_fields() -> Vec<&'static str>;
}
```

## Usage

### Basic Query Builder Creation

```rust
use crate::app::query_builder::{QueryBuilder, QueryBuilderExt};

// Create a new query builder
let query = User::query();

// Or from HTTP parameters
let query = User::from_params(query_params)?;
```

### Service Integration

The most common usage is through the `QueryBuilderService` trait:

```rust
use crate::app::query_builder::QueryBuilderService;

// In your controller
pub async fn index(
    Query(params): Query<QueryParams>,
    State(pool): State<DbPool>,
) -> Result<Json<PaginationResult<serde_json::Value>>, AppError> {
    let result = User::index(Query(params), &pool)?;
    Ok(Json(result))
}
```

## Filtering

The Query Builder supports a comprehensive set of filtering operators:

### Filter Operators

| Operator      | SQL           | Description                       |
| ------------- | ------------- | --------------------------------- |
| `eq`          | `=`           | Equals                            |
| `ne`          | `!=`          | Not equals                        |
| `gt`          | `>`           | Greater than                      |
| `gte`         | `>=`          | Greater than or equal             |
| `lt`          | `<`           | Less than                         |
| `lte`         | `<=`          | Less than or equal                |
| `like`        | `LIKE`        | Pattern matching                  |
| `ilike`       | `ILIKE`       | Case-insensitive pattern matching |
| `in`          | `IN`          | Value in list                     |
| `not_in`      | `NOT IN`      | Value not in list                 |
| `is_null`     | `IS NULL`     | Field is null                     |
| `is_not_null` | `IS NOT NULL` | Field is not null                 |
| `between`     | `BETWEEN`     | Value between two values          |

### HTTP Query Parameter Format

Filters can be specified in URL query parameters using Laravel-style syntax:

```
# Simple equality
?filter[name]=John

# Operator-based filtering
?filter[age][gte]=18
?filter[status][in]=active,pending
?filter[created_at][between]=2023-01-01,2023-12-31

# Multiple filters
?filter[name]=John&filter[age][gte]=18&filter[status][ne]=deleted
```

### Programmatic Filtering

```rust
let query = User::query()
    .where_eq("name", "John")
    .where_gte("age", 18)
    .where_in("status", vec!["active", "pending"])
    .where_between("created_at", "2023-01-01", "2023-12-31");
```

### Advanced Filtering

```rust
// OR conditions
let query = User::query()
    .where_eq("status", "active")
    .or_where("status", "pending");

// Null checks
let query = User::query()
    .where_not_null("email")
    .where_null("deleted_at");

// JSON field filtering
let query = User::query()
    .where_json("metadata", "preferences.theme", "dark");

// Search across multiple fields
let query = User::query()
    .search("john", vec!["name".to_string(), "email".to_string()]);

// Date-based filtering
let query = User::query()
    .where_date("created_at", "2023-01-01")
    .where_year("created_at", 2023)
    .where_month("created_at", 1);
```

## Sorting

### HTTP Query Parameter Format

```
# Single field ascending
?sort=name

# Single field descending
?sort=-created_at

# Multiple fields
?sort=name,-created_at,email
```

### Programmatic Sorting

```rust
let query = User::query()
    .order_by("name")                    // Ascending
    .order_by_desc("created_at")         // Descending
    .sort(Sort::asc("email"));           // Using Sort struct
```

### Default Sorting

Models can specify default sorting behavior:

```rust
impl Queryable for User {
    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }
}
```

## Pagination

The Query Builder supports two pagination strategies:

### Cursor-Based Pagination (Default)

Cursor-based pagination provides consistent results and better performance for large datasets:

```rust
// HTTP query parameters
?per_page=20&cursor=eyJ0aW1lc3RhbXAiOjE2...

// Programmatic
let query = User::query()
    .cursor_paginate(20, Some("cursor_string".to_string()));
```

#### Cursor Structure

Cursors are base64-encoded JSON containing:

```rust
pub struct CursorData {
    pub timestamp: i64,    // Timestamp for ordering consistency
    pub position: u32,     // Position within the current page
    pub per_page: u32,     // Page size for consistency checks
}
```

### Offset-Based Pagination

Traditional page-based pagination:

```rust
// HTTP query parameters
?page=2&per_page=20&pagination_type=offset

// Programmatic
let query = User::query()
    .offset_paginate(2, 20);
```

### Pagination Response

Both pagination types return a structured response:

```rust
pub struct PaginationResult<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

pub struct PaginationInfo {
    pub pagination_type: PaginationType,
    pub current_page: Option<u32>,      // Offset only
    pub per_page: u32,
    pub total: Option<u64>,             // Offset only
    pub total_pages: Option<u32>,       // Offset only
    pub has_more_pages: bool,
    pub next_cursor: Option<String>,    // Cursor only
    pub prev_cursor: Option<String>,    // Cursor only
    // ... URL fields for navigation
}
```

## Field Selection

### HTTP Query Parameter Format

```
# Select specific fields for the main resource
?fields[users]=id,name,email

# Select fields for included relationships
?fields[users]=id,name&fields[organization]=name,slug
```

### Programmatic Field Selection

```rust
let query = User::query()
    .select(vec!["id", "name", "email"])
    .fields(vec!["id".to_string(), "name".to_string()]);
```

### Default Fields

Models can specify which fields to select by default:

```rust
impl Queryable for User {
    fn default_fields() -> Vec<&'static str> {
        vec!["id", "name", "email", "created_at"]
    }
}
```

## Relationships

### HTTP Query Parameter Format

```
# Include single relationship
?include=organization

# Include multiple relationships
?include=organization,roles,permissions

# Nested relationships (if supported)
?include=organization.users,roles.permissions
```

### Programmatic Relationships

```rust
let query = User::query()
    .with("organization")
    .with("roles")
    .include(Include::new("permissions"));
```

### Relationship Configuration

Models specify which relationships can be included:

```rust
impl Queryable for User {
    fn allowed_includes() -> Vec<&'static str> {
        vec!["organization", "roles", "permissions"]
    }
}
```

## Query Execution

### Service Methods

The `QueryBuilderService` trait provides high-level methods:

```rust
// Paginated results
let result = User::index(Query(params), &pool)?;

// All results (no pagination)
let users = User::all(Query(params), &pool)?;

// First result only
let user = User::first(Query(params), &pool)?;

// Count only
let count = User::count(Query(params), &pool)?;
```

### Custom Execution

```rust
let query = User::query()
    .where_eq("status", "active")
    .order_by_desc("created_at");

let result = User::execute_paginated(query, &pool)?;
```

### Direct QueryExecutor Usage

```rust
use crate::app::query_builder::QueryExecutor;

let result = QueryExecutor::execute_paginated(query, &mut conn)?;
```

## Service Integration

### Implementing QueryBuilderService

Use the convenience macro to implement the service:

```rust
use crate::impl_query_builder_service;

impl_query_builder_service!(User);
```

### Manual Implementation

```rust
impl QueryBuilderService<User> for User {}
```

### Controller Integration

```rust
use axum::{extract::Query, Json, State};
use crate::app::query_builder::{QueryParams, QueryBuilderService};

pub async fn users_index(
    Query(params): Query<QueryParams>,
    State(pool): State<DbPool>,
) -> Result<Json<PaginationResult<serde_json::Value>>, AppError> {
    let result = User::index(Query(params), &pool)?;
    Ok(Json(result))
}
```

## Examples

### Model Implementation

```rust
use crate::app::query_builder::{Queryable, SortDirection};

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Queryable for User {
    fn table_name() -> &'static str {
        "users"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec!["id", "name", "email", "status", "created_at"]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec!["id", "name", "email", "created_at"]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec!["id", "name", "email", "created_at", "updated_at"]
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec!["organization", "roles"]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }
}

impl_query_builder_service!(User);
```

### Complex Query Example

```rust
let query = User::query()
    .where_eq("status", "active")
    .where_gte("created_at", "2023-01-01")
    .where_in("role", vec!["admin", "moderator"])
    .where_not_null("email_verified_at")
    .search("john", vec!["name".to_string(), "email".to_string()])
    .with("organization")
    .with("roles")
    .select(vec!["id", "name", "email"])
    .order_by("name")
    .order_by_desc("created_at")
    .cursor_paginate(25, None);

let result = User::execute_paginated(query, &pool)?;
```

### HTTP API Usage

```bash
# Get active users with search, sorting, and pagination
GET /api/users?filter[status]=active&filter[created_at][gte]=2023-01-01&search=john&sort=name,-created_at&include=organization,roles&fields[users]=id,name,email&per_page=25&cursor=eyJ0aW1lc3RhbXAi...

# Get users by role with offset pagination
GET /api/users?filter[role][in]=admin,moderator&pagination_type=offset&page=2&per_page=20

# Count users by status
GET /api/users/count?filter[status]=active
```

## Performance Considerations

### Query Optimization

1. **Field Selection**: Always specify only required fields to reduce data transfer
2. **Cursor Pagination**: Use cursor-based pagination for large datasets
3. **Index Usage**: Ensure filtered and sorted fields are properly indexed
4. **Limit Inclusions**: Only include necessary relationships

### Caching

The Query Builder supports intelligent caching:

```rust
// Results are automatically cached based on query signature
let result = User::index(Query(params), &pool)?;
```

### Batch Operations

For bulk operations, consider using batch methods:

```rust
// Instead of multiple single queries
let users = User::query()
    .where_in("id", user_ids)
    .all(&pool)?;
```

## Security Features

### SQL Injection Prevention

- All values are properly escaped and parameterized
- Raw SQL filters require explicit opt-in
- Field names are validated against allowed lists

### Field Validation

```rust
// Only allowed fields can be filtered/sorted
impl Queryable for User {
    fn allowed_filters() -> Vec<&'static str> {
        vec!["id", "name", "email"] // "password" not included
    }
}
```

### Rate Limiting

Consider implementing rate limiting for complex queries:

```rust
// Limit per_page to reasonable bounds
let per_page = params.per_page.unwrap_or(15).min(100);
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::query_builder::*;

    #[test]
    fn test_query_builder_filters() {
        let query = TestModel::query()
            .where_eq("name", "John")
            .where_gte("age", 18);

        assert_eq!(query.get_filters().len(), 2);
    }

    #[test]
    fn test_pagination() {
        let pagination = Pagination::cursor(20, None);
        assert!(pagination.is_cursor());
        assert_eq!(pagination.per_page, 20);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_user_query_service() {
    let pool = setup_test_db().await;
    let params = QueryParams::default();

    let result = User::index(Query(params), &pool).unwrap();
    assert!(!result.data.is_empty());
}
```

## Best Practices

### Model Design

1. **Explicit Allowlists**: Always specify allowed filters, sorts, and fields
2. **Sensible Defaults**: Provide reasonable default sorting and field selection
3. **Security First**: Never expose sensitive fields in allowed lists

### Query Construction

1. **Chain Efficiently**: Build queries step by step with clear intent
2. **Validate Input**: Always validate HTTP parameters before processing
3. **Use Services**: Prefer service methods over direct QueryExecutor usage

### Performance

1. **Index Strategy**: Ensure database indexes match your query patterns
2. **Field Selection**: Only select needed fields to reduce bandwidth
3. **Pagination**: Use cursor pagination for large datasets
4. **Relationship Loading**: Only include necessary relationships

### Error Handling

```rust
pub async fn users_index(
    Query(params): Query<QueryParams>,
    State(pool): State<DbPool>,
) -> Result<Json<PaginationResult<serde_json::Value>>, AppError> {
    match User::index(Query(params), &pool) {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            tracing::error!("Failed to query users: {}", e);
            Err(AppError::DatabaseError(e.to_string()))
        }
    }
}
```

### Documentation

1. **Document Queryable**: Always document which fields are queryable and why
2. **API Documentation**: Use OpenAPI/Swagger annotations for HTTP endpoints
3. **Examples**: Provide working examples for complex queries

## Advanced Features

### Conditional Queries

```rust
let query = User::query()
    .when(include_deleted, |q| q.with_trashed())
    .when_some(organization_id, |q, org_id| {
        q.where_eq("organization_id", org_id)
    });
```

### Query Inspection

```rust
let info = query.get_query_info();
println!("Filters: {}, Sorts: {}", info.filters_count, info.sorts_count);
```

### Custom Executors

Implement custom execution logic for specific needs:

```rust
impl QueryExecutor {
    pub fn execute_with_cache<T>(
        builder: QueryBuilder<T>,
        conn: &mut DbConnection,
        cache_key: &str,
    ) -> Result<PaginationResult<serde_json::Value>>
    where
        T: Queryable + Clone,
    {
        // Custom caching logic
        todo!()
    }
}
```

This Query Builder system provides a powerful, type-safe, and secure foundation for building complex database queries in Rust applications while maintaining the familiar Laravel-style API that developers love.
