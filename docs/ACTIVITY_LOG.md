# Activity Log System

A comprehensive activity logging system for RustAxum, inspired by Spatie's Laravel ActivityLog package. This system provides correlation tracking, flexible querying, and easy integration with your models.

## Features

- 🔗 **Correlation Support**: Track related activities across requests
- 🏷️ **Log Names**: Organize activities into different logs (auth, user_management, etc.)
- 🎯 **Subject/Causer Tracking**: Know what was acted upon and who performed the action
- 📦 **Batch Operations**: Group related activities together
- 🔍 **Flexible Querying**: Query activities by various criteria
- ⚙️ **Configurable**: Environment-based configuration
- 🧪 **Macro Support**: Convenient macros for easy logging

## Quick Start

### 1. Run Migrations

First, run the migration to create the activity_log table:

```bash
cargo run --bin artisan -- migrate
```

### 2. Add Middleware (Optional)

To enable automatic correlation tracking, add the correlation middleware to your application:

```rust
use rustaxum::app::http::middleware::correlation::correlation_middleware;

// In your main.rs or route setup
let app = Router::new()
    .layer(middleware::from_fn(correlation_middleware))
    // ... other middleware and routes
```

### 3. Basic Usage

```rust
use rustaxum::app::activity_log::prelude::*;

// Simple activity logging
activity!("User logged in").await?;

// Log with a specific log name
activity_log!("auth", "User login successful").await?;

// Log with correlation ID (automatically extracted from request context)
activity_correlated!(correlation_id, "Payment processed").await?;
```

## Configuration

Add these environment variables to your `.env` file:

```env
# Activity Log Configuration
ACTIVITY_LOG_ENABLED=true
ACTIVITY_LOG_DEFAULT_NAME=default
ACTIVITY_LOG_AUTO_MODEL_EVENTS=false
ACTIVITY_LOG_AUTO_CORRELATION=true
ACTIVITY_LOG_INCLUDE_CAUSER=true
ACTIVITY_LOG_INCLUDE_SUBJECT=true
ACTIVITY_LOG_MAX_COUNT=0
ACTIVITY_LOG_RETENTION_DAYS=0
ACTIVITY_LOG_AUTO_CLEANUP=false
ACTIVITY_LOG_EXCLUDED_EVENTS=
ACTIVITY_LOG_EXCLUDED_MODELS=
ACTIVITY_LOG_PROPERTIES=true
ACTIVITY_LOG_MAX_PROPERTIES_SIZE=65536
```

## Usage Examples

### Basic Activity Logging

```rust
use rustaxum::app::activity_log::prelude::*;

// Simple activity
let log = activity!("User updated profile").await?;

// Activity with properties
let log = ActivityLog::builder()
    .description("User updated email")
    .with_property("old_email", "old@example.com")
    .with_property("new_email", "new@example.com")
    .log()
    .await?;
```

### Using Log Names

```rust
// Log to specific logs
activity_log!("auth", "User logged in").await?;
activity_log!("user_management", "Profile updated").await?;
activity_log!("payments", "Payment processed").await?;

// Using builder with log name
ActivityLog::for_log("orders")
    .description("Order created")
    .with_property("order_id", "ORD-123")
    .event("created")
    .log()
    .await?;
```

### Model Activity Logging

```rust
use rustaxum::app::models::user::User;

// Log activity performed on a model
let user = get_user().await?;
activity_on!(user)
    .description("Profile updated")
    .event("updated")
    .log()
    .await?;

// Log activity caused by a model
activity_by!(admin_user)
    .description("User account suspended")
    .event("suspended")
    .log()
    .await?;

// Using the LogsActivity trait
user.log_activity("Password changed").log().await?;
user.log_activity_as_causer("Invited team member").log().await?;
```

### Correlation Tracking

```rust
// In a request handler
async fn checkout_handler(
    correlation_id: CorrelationId, // Extracted from middleware
    // ... other parameters
) -> Result<impl IntoResponse> {
    // All these activities will share the same correlation ID
    activity_correlated!(correlation_id, "Checkout started").await?;

    // Process payment
    activity_correlated!(correlation_id, "Payment validated").await?;

    // Create order
    activity_correlated!(correlation_id, "Order created").await?;

    Ok("Checkout complete")
}

// Query all activities for this correlation
let service = ActivityLogService::new();
let related_activities = service
    .find_by_correlation_id(correlation_id)
    .await?;
```

### Batch Operations

```rust
// Log bulk operations with the same batch UUID
let batch_uuid = uuid::Uuid::new_v4().to_string();

activity_batch!(batch_uuid, "Bulk user import started").await?;

// Create multiple activities in batch
let activities = vec![
    NewActivityLog {
        log_name: Some("bulk_import".to_string()),
        description: "User imported: user1@example.com".to_string(),
        batch_uuid: Some(batch_uuid.clone()),
        event: Some("imported".to_string()),
        // ... other fields
    },
    // ... more activities
];

let service = ActivityLogService::new();
let batch_logs = service.create_batch(activities).await?;
```

### Advanced Builder Pattern

```rust
let log = ActivityLog::builder()
    .log_name("complex_operation")
    .description("Data processing completed")
    .performed_on(&user)           // What was acted upon
    .caused_by(&admin_user)        // Who performed the action
    .correlation_id(correlation_id) // Link to other activities
    .event("processing_completed")
    .with_property("records_processed", 1500)
    .with_property("duration_ms", 2340)
    .with_properties(json!({
        "metadata": {
            "version": "1.0",
            "environment": "production"
        }
    }))
    .log()
    .await?;
```

### Querying Activities

```rust
let service = ActivityLogService::new();

// Query by log name
let auth_logs = service.find_by_log_name("auth").await?;

// Query by event type
let created_events = service.find_by_event("created").await?;

// Query by subject (what was acted upon)
let user_activities = service.find_by_subject::<User>("user_id").await?;

// Query by causer (who performed the action)
let admin_activities = service.find_by_causer::<User>("admin_id").await?;

// Advanced query builder
let activities = service.query()
    .log_name("payments")
    .for_event("payment_processed")
    .performed_on::<User>("user_123")
    .limit(50)
    .offset(0)
    .get()
    .await?;

// Count activities
let total = service.query()
    .log_name("auth")
    .count()
    .await?;

// Get activities in a specific log
let payment_logs = service.find_in_log("payments").await
    .for_event("processed")
    .limit(10)
    .get()
    .await?;
```

### Helper Functions

```rust
use rustaxum::app::activity_log::helpers;

// Log authentication events
helpers::log_auth_event("user_123", "login", Some(correlation_id)).await?;

// Log model lifecycle events
helpers::log_created(&user, Some("admin_123"), Some(correlation_id)).await?;
helpers::log_updated(&user, Some("admin_123"), Some(correlation_id), Some(vec!["email", "name"])).await?;
helpers::log_deleted(&user, Some("admin_123"), Some(correlation_id)).await?;

// Log batch operations
helpers::log_batch_operation("user_import", 100, Some(correlation_id)).await?;
```

## Middleware Integration

### Correlation Middleware

The correlation middleware automatically tracks requests with correlation IDs:

```rust
use rustaxum::app::http::middleware::correlation::{correlation_middleware, CorrelationExt};

// Add to your middleware stack
let app = Router::new()
    .layer(middleware::from_fn(correlation_middleware))
    .route("/api/users", post(create_user));

// In your handlers
async fn create_user(request: Request) -> Result<impl IntoResponse> {
    // Get correlation ID from request
    let correlation_id = request.correlation_id().unwrap_or_else(DieselUlid::new);

    // Use in activity logging
    activity_correlated!(correlation_id, "User creation started").await?;

    // ... rest of handler
}
```

## Model Integration

Implement the required traits on your models to enable activity logging:

```rust
use rustaxum::app::models::{HasModelType, HasId};
use rustaxum::app::activity_log::LogsActivity;

// Your model needs to implement HasModelType and HasId
impl HasModelType for MyModel {
    fn model_type() -> &'static str {
        "MyModel"
    }
}

impl HasId for MyModel {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

// LogsActivity is automatically implemented through blanket implementation
// Now you can use:
my_model.log_activity("Something happened").log().await?;
```

## Testing

Run the activity log tests:

```bash
# Run specific activity log tests
cargo test activity_log

# Run all tests
cargo test
```

## Database Schema

The activity log table has the following structure:

```sql
CREATE TABLE activity_log (
    id CHAR(26) PRIMARY KEY,
    correlation_id CHAR(26) REFERENCES correlation(id) ON DELETE SET NULL,
    log_name VARCHAR(255),
    description TEXT NOT NULL,
    subject_type VARCHAR(255),
    subject_id VARCHAR(255),
    causer_type VARCHAR(255),
    causer_id VARCHAR(255),
    properties JSONB,
    batch_uuid VARCHAR(255),
    event VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

## Performance Considerations

- **Indexes**: The migration creates indexes on commonly queried fields
- **Properties Size**: Configure `ACTIVITY_LOG_MAX_PROPERTIES_SIZE` to limit JSON payload size
- **Retention**: Use `ACTIVITY_LOG_RETENTION_DAYS` and `ACTIVITY_LOG_AUTO_CLEANUP` for automatic cleanup
- **Excluded Models/Events**: Use exclusion lists for high-frequency events that don't need logging

## Examples

See `examples/activity_log_usage.rs` for comprehensive usage examples.

## Laravel ActivityLog Comparison

This implementation provides similar functionality to Spatie's Laravel ActivityLog:

| Feature | Laravel ActivityLog | RustAxum ActivityLog |
|---------|-------------------|---------------------|
| Basic activity logging | ✅ | ✅ |
| Log names | ✅ | ✅ |
| Subject/Causer tracking | ✅ | ✅ |
| Properties | ✅ | ✅ |
| Events | ✅ | ✅ |
| Batch operations | ❌ | ✅ |
| Correlation tracking | ❌ | ✅ |
| Flexible querying | ✅ | ✅ |
| Configuration | ✅ | ✅ |
| Automatic cleanup | ✅ | ✅ |

## API Endpoints

The activity log system provides RESTful API endpoints for fetching and managing activity data:

### List Activity Logs

```http
GET /api/activity-logs
```

Query parameters (using the framework's query builder):
- `filter[log_name]=auth` - Filter by log name
- `filter[event]=created` - Filter by event type
- `filter[subject_type]=User` - Filter by subject type
- `filter[subject_id]=user123` - Filter by subject ID
- `filter[causer_type]=User` - Filter by causer type
- `filter[causer_id]=admin123` - Filter by causer ID
- `filter[correlation_id]=01ARZ3NDEKTSV4RRFFQ69G5FAV` - Filter by correlation ID
- `filter[batch_uuid]=batch-uuid-123` - Filter by batch UUID
- `sort=created_at` or `sort=-created_at` - Sort by creation date
- `page=1&per_page=50` - Pagination

Example:
```bash
curl "http://localhost:3000/api/activity-logs?filter[log_name]=auth&sort=-created_at&per_page=10"
```

### Get Single Activity Log

```http
GET /api/activity-logs/{id}
```

Example:
```bash
curl "http://localhost:3000/api/activity-logs/01ARZ3NDEKTSV4RRFFQ69G5FAV"
```

### Get Activities by Correlation ID

```http
GET /api/activity-logs/correlation/{correlation_id}
```

Returns all activities linked by the same correlation ID.

Example:
```bash
curl "http://localhost:3000/api/activity-logs/correlation/01ARZ3NDEKTSV4RRFFQ69G5FAV"
```

### Get Activities by Batch UUID

```http
GET /api/activity-logs/batch/{batch_uuid}
```

Returns all activities in the same batch operation.

Example:
```bash
curl "http://localhost:3000/api/activity-logs/batch/550e8400-e29b-41d4-a716-446655440000"
```

### Get Activities by Subject

```http
GET /api/activity-logs/subject/{subject_type}/{subject_id}
```

Returns all activities performed on a specific subject (model instance).

Example:
```bash
curl "http://localhost:3000/api/activity-logs/subject/User/01ARZ3NDEKTSV4RRFFQ69G5FAV"
```

### Get Activities by Causer

```http
GET /api/activity-logs/causer/{causer_type}/{causer_id}
```

Returns all activities caused by a specific causer (who performed the action).

Example:
```bash
curl "http://localhost:3000/api/activity-logs/causer/User/01ARZ3NDEKTSV4RRFFQ69G5FAV"
```

### Create Activity Log

```http
POST /api/activity-logs
Content-Type: application/json
```

Request body:
```json
{
  "log_name": "user_management",
  "description": "User profile updated",
  "subject_type": "User",
  "subject_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "causer_type": "User",
  "causer_id": "01BRZ3NDEKTSV4RRFFQ69G5FAV",
  "event": "updated",
  "properties": {
    "fields_changed": ["email", "name"],
    "old_email": "old@example.com",
    "new_email": "new@example.com"
  }
}
```

### Get Activity Statistics

```http
GET /api/activity-logs/stats
```

Returns basic statistics about activity logs:
```json
{
  "total_activities": 1500,
  "by_log_name": {},
  "by_event": {},
  "by_subject_type": {},
  "recent_count": 0
}
```

### Example API Usage

```bash
# List recent authentication activities
curl "http://localhost:3000/api/activity-logs?filter[log_name]=auth&sort=-created_at&per_page=20"

# Get all activities for a specific user
curl "http://localhost:3000/api/activity-logs/subject/User/01ARZ3NDEKTSV4RRFFQ69G5FAV"

# Find all activities in a correlation chain
curl "http://localhost:3000/api/activity-logs/correlation/01ARZ3NDEKTSV4RRFFQ69G5FAV"

# Get activities by multiple filters
curl "http://localhost:3000/api/activity-logs?filter[event]=created&filter[subject_type]=User&sort=-created_at"

# Create a new activity log entry
curl -X POST "http://localhost:3000/api/activity-logs" \
  -H "Content-Type: application/json" \
  -d '{
    "log_name": "api_access",
    "description": "API endpoint accessed",
    "event": "api_call",
    "properties": {
      "endpoint": "/api/users",
      "method": "GET",
      "ip_address": "192.168.1.1"
    }
  }'
```

### Response Format

All endpoints return JSON responses with consistent structure:

**Single Activity Log:**
```json
{
  "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "log_name": "auth",
  "description": "User logged in",
  "subject_type": "User",
  "subject_id": "01BRZ3NDEKTSV4RRFFQ69G5FAV",
  "causer_type": null,
  "causer_id": null,
  "properties": {
    "ip_address": "192.168.1.1",
    "user_agent": "Mozilla/5.0..."
  },
  "correlation_id": "01CRZ3NDEKTSV4RRFFQ69G5FAV",
  "batch_uuid": null,
  "event": "login",
  "created_at": "2025-09-23T10:30:00Z",
  "updated_at": "2025-09-23T10:30:00Z"
}
```

**Paginated List:**
```json
{
  "data": [
    // ... activity log objects
  ],
  "meta": {
    "current_page": 1,
    "per_page": 50,
    "total": 1500,
    "last_page": 30,
    "from": 1,
    "to": 50
  }
}
```

### Error Responses

```json
{
  "error": "Activity log not found"
}
```

## Contributing

When contributing to the activity log system:

1. Add tests for new features
2. Update this documentation
3. Follow the existing code patterns
4. Ensure backward compatibility
