# Laravel-Style QueryBuilder for Rust/Axum

This implementation provides a powerful, Laravel-inspired QueryBuilder for Rust applications using Axum and SQLx. It supports filtering, sorting, field selection, and pagination similar to Spatie's Laravel QueryBuilder.

## Features

✅ **Advanced Filtering** - Support for multiple operators (eq, ne, gt, gte, lt, lte, like, in, not_in, is_null, is_not_null)
✅ **Flexible Sorting** - Multiple sort formats with validation
✅ **Field Selection** - Choose which fields to include in responses
✅ **Pagination** - Built-in pagination support with metadata
✅ **Type Safety** - Full Rust type safety with trait-based approach
✅ **Security** - Whitelist-based field validation to prevent SQL injection

## Quick Start

### 1. Make your model queryable

```rust
use rustaxum::query_builder::{Queryable, SortDirection};
use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Queryable for User {
    fn table_name() -> &'static str {
        "users"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec!["id", "name", "email", "created_at", "updated_at"]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec!["id", "name", "email", "created_at", "updated_at"]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec!["id", "name", "email", "created_at", "updated_at"]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }
}
```

### 2. Use in your controller

```rust
use axum::{
    extract::{State, Query},
    response::{IntoResponse, Json},
    http::StatusCode,
};
use rustaxum::query_builder::{QueryBuilder, QueryParams};

pub async fn index(
    State(pool): State<PgPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    let request = params.parse();
    let query_builder = QueryBuilder::<User>::new(pool, request);

    match query_builder.paginate().await {
        Ok(result) => {
            (StatusCode::OK, Json(result)).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch users"
            }))).into_response()
        }
    }
}
```

## Usage Examples

### Basic Filtering

```bash
# Equal filter (default operator)
GET /users?name=John

# Not equal
GET /users?name[ne]=John

# Greater than / Less than
GET /users?created_at[gt]=2024-01-01
GET /users?id[lt]=100

# Like (case-insensitive)
GET /users?email[like]=@gmail.com

# In / Not In
GET /users?id[in]=1,2,3,4,5
GET /users?status[not_in]=banned,suspended

# Null checks
GET /users?deleted_at[is_null]=true
GET /users?email_verified_at[is_not_null]=true
```

### Advanced Filtering

```bash
# Combine multiple filters
GET /users?name[like]=john&created_at[gte]=2024-01-01&status[ne]=inactive

# Date range filtering
GET /users?created_at[gte]=2024-01-01&created_at[lte]=2024-12-31
```

### Sorting

```bash
# Single sort (ascending by default)
GET /users?sort=name

# Descending sort
GET /users?sort=-created_at

# Multiple sorts
GET /users?sort=name,-created_at,email

# Explicit direction
GET /users?sort=name:asc,created_at:desc
```

### Field Selection

```bash
# Select specific fields
GET /users?fields=id,name,email

# Combine with filtering and sorting
GET /users?fields=id,name,email&sort=-created_at&status=active
```

### Pagination

```bash
# Basic pagination
GET /users?page=2&per_page=10

# Combined with other parameters
GET /users?page=3&per_page=5&sort=-created_at&status=active&fields=id,name,email
```

## Supported Filter Operators

| Operator | Usage | Description |
|----------|-------|-------------|
| `eq` (default) | `field=value` | Equal to |
| `ne` | `field[ne]=value` | Not equal to |
| `gt` | `field[gt]=value` | Greater than |
| `gte` | `field[gte]=value` | Greater than or equal |
| `lt` | `field[lt]=value` | Less than |
| `lte` | `field[lte]=value` | Less than or equal |
| `like` | `field[like]=value` | Case-insensitive LIKE |
| `not_like` | `field[not_like]=value` | Case-insensitive NOT LIKE |
| `in` | `field[in]=val1,val2,val3` | In list |
| `not_in` | `field[not_in]=val1,val2` | Not in list |
| `is_null` | `field[is_null]=true` | Is NULL |
| `is_not_null` | `field[is_not_null]=true` | Is NOT NULL |

## Response Format

### Paginated Response

```json
{
  "data": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "name": "John Doe",
      "email": "john@example.com",
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-15T10:30:00Z"
    }
  ],
  "meta": {
    "current_page": 1,
    "per_page": 15,
    "total": 42,
    "last_page": 3,
    "from": 1,
    "to": 15
  }
}
```

### Non-paginated Response

```rust
// Use .get() instead of .paginate() for simple arrays
let users = query_builder.get().await?;
```

## Security Features

### Whitelist Validation

All filterable fields, sortable fields, and selectable fields must be explicitly whitelisted in the `Queryable` implementation. This prevents:

- SQL injection attacks
- Unauthorized access to sensitive fields
- Performance issues from unindexed column filtering

### Example Security Configuration

```rust
impl Queryable for User {
    fn allowed_filters() -> Vec<&'static str> {
        // Only allow filtering on these fields
        vec!["id", "name", "email", "status", "created_at"]
        // "password", "secret_token" etc. are NOT included
    }

    fn allowed_sorts() -> Vec<&'static str> {
        // Only allow sorting on indexed fields for performance
        vec!["id", "created_at", "name"]
    }

    fn allowed_fields() -> Vec<&'static str> {
        // Control which fields can be selected
        vec!["id", "name", "email", "created_at", "updated_at"]
        // Sensitive fields like "password" are excluded
    }
}
```

## Advanced Usage

### Custom Query Building

```rust
// For more complex queries, you can access the internal parts
let (select_clause, where_clause, order_clause, params) = query_builder.build_query_parts()?;

// Then use with custom SQL
let custom_query = format!(
    "SELECT {} FROM {} {} {} HAVING COUNT(*) > 1",
    select_clause, User::table_name(), where_clause, order_clause
);
```

### Error Handling

```rust
match query_builder.paginate().await {
    Ok(result) => Ok(Json(result)),
    Err(e) => {
        tracing::error!("Query failed: {}", e);
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Database query failed"}))
        ))
    }
}
```

## Generated SQL Examples

### Basic Query

```sql
-- GET /users?name=John&sort=-created_at&page=1&per_page=10
SELECT * FROM users
WHERE name = $1
ORDER BY created_at DESC
LIMIT $2 OFFSET $3
```

### Complex Query

```sql
-- GET /users?name[like]=john&status[in]=active,verified&created_at[gte]=2024-01-01&sort=name,-created_at&fields=id,name,email
SELECT id, name, email FROM users
WHERE name ILIKE $1 AND status IN ($2, $3) AND created_at >= $4
ORDER BY name ASC, created_at DESC
LIMIT $5 OFFSET $6
```

## Integration with Existing Laravel/PHP Experience

If you're familiar with Spatie's Laravel QueryBuilder, this implementation provides similar functionality:

| Laravel QueryBuilder | Rust QueryBuilder | Example |
|---------------------|-------------------|---------|
| `allowedFilters()` | `allowed_filters()` | Whitelist filterable fields |
| `allowedSorts()` | `allowed_sorts()` | Whitelist sortable fields |
| `allowedFields()` | `allowed_fields()` | Whitelist selectable fields |
| `defaultSort()` | `default_sort()` | Set default sorting |
| `Filter::exact()` | `field=value` | Exact match filter |
| `Filter::partial()` | `field[like]=value` | Partial match filter |
| `paginate()` | `.paginate()` | Paginated results |
| `get()` | `.get()` | All results |

## Performance Considerations

1. **Indexes**: Ensure database indexes exist for frequently filtered and sorted fields
2. **Field Selection**: Use field selection to reduce data transfer
3. **Pagination**: Always use pagination for large datasets
4. **Validation**: The whitelist approach prevents expensive operations on unindexed columns

## Testing

```rust
#[tokio::test]
async fn test_query_builder_filtering() {
    let pool = setup_test_db().await;

    let mut filters = HashMap::new();
    filters.insert("status".to_string(), "active".to_string());

    let request = QueryBuilderRequest {
        filters,
        sorts: vec!["-created_at".to_string()],
        fields: Some(vec!["id".to_string(), "name".to_string()]),
        page: Some(1),
        per_page: Some(10),
    };

    let query_builder = QueryBuilder::<User>::new(pool, request);
    let result = query_builder.paginate().await.unwrap();

    assert!(!result.data.is_empty());
    assert_eq!(result.meta.current_page, 1);
    assert_eq!(result.meta.per_page, 10);
}
```

This QueryBuilder implementation provides a robust, secure, and familiar API for building complex database queries in Rust applications while maintaining type safety and preventing common security vulnerabilities.
