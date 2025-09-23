use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Json, IntoResponse},
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use std::collections::HashMap;

use crate::app::activity_log::prelude::*;
use crate::app::http::middleware::correlation_middleware::CorrelationContext;
use crate::app::query_builder::{QueryParams, QueryBuilderService};
use crate::database::DbPool;

/// Query parameters for activity log listing
#[derive(Debug, Deserialize, ToSchema)]
pub struct ActivityLogQuery {
    /// Log name to filter by
    pub log_name: Option<String>,
    /// Subject type to filter by
    pub subject_type: Option<String>,
    /// Subject ID to filter by
    pub subject_id: Option<String>,
    /// Causer type to filter by
    pub causer_type: Option<String>,
    /// Causer ID to filter by
    pub causer_id: Option<String>,
    /// Event to filter by
    pub event: Option<String>,
    /// Correlation ID to filter by
    pub correlation_id: Option<String>,
    /// Batch UUID to filter by
    pub batch_uuid: Option<String>,
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Items per page (default: 50, max: 100)
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    /// Sort field (created_at, updated_at, description)
    #[serde(default = "default_sort")]
    pub sort: String,
    /// Sort direction (asc, desc)
    #[serde(default = "default_order")]
    pub order: String,
    /// Date range start (ISO 8601)
    pub date_from: Option<String>,
    /// Date range end (ISO 8601)
    pub date_to: Option<String>,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 50 }
fn default_sort() -> String { "created_at".to_string() }
fn default_order() -> String { "desc".to_string() }

/// Response for activity log listing
#[derive(Debug, Serialize, ToSchema)]
pub struct ActivityLogListResponse {
    /// Activity log items
    pub data: Vec<ActivityLogResponse>,
    /// Pagination metadata
    pub meta: PaginationMeta,
}

/// Single activity log response
#[derive(Debug, Serialize, ToSchema)]
pub struct ActivityLogResponse {
    /// Unique identifier
    pub id: String,
    /// Log name/category
    pub log_name: Option<String>,
    /// Activity description
    pub description: String,
    /// Subject type (e.g., "User", "Organization")
    pub subject_type: Option<String>,
    /// Subject ID
    pub subject_id: Option<String>,
    /// Causer type (who performed the action)
    pub causer_type: Option<String>,
    /// Causer ID
    pub causer_id: Option<String>,
    /// Additional properties as JSON
    pub properties: Option<Value>,
    /// Correlation ID for related activities
    pub correlation_id: Option<String>,
    /// Batch UUID for grouped activities
    pub batch_uuid: Option<String>,
    /// Event type
    pub event: Option<String>,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
}

/// Pagination metadata
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginationMeta {
    /// Current page number
    pub current_page: u32,
    /// Items per page
    pub per_page: u32,
    /// Total number of items
    pub total: u64,
    /// Total number of pages
    pub last_page: u32,
    /// First item number on current page
    pub from: u32,
    /// Last item number on current page
    pub to: u32,
}

/// Activity log statistics
#[derive(Debug, Serialize, ToSchema)]
pub struct ActivityLogStats {
    /// Total activity count
    pub total_activities: u64,
    /// Activity count by log name
    pub by_log_name: HashMap<String, u64>,
    /// Activity count by event type
    pub by_event: HashMap<String, u64>,
    /// Activity count by subject type
    pub by_subject_type: HashMap<String, u64>,
    /// Recent activity count (last 24 hours)
    pub recent_count: u64,
}

/// Request body for creating activity log
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateActivityLogRequest {
    /// Log name/category
    pub log_name: Option<String>,
    /// Activity description
    pub description: String,
    /// Subject type
    pub subject_type: Option<String>,
    /// Subject ID
    pub subject_id: Option<String>,
    /// Causer type
    pub causer_type: Option<String>,
    /// Causer ID
    pub causer_id: Option<String>,
    /// Additional properties
    pub properties: Option<Value>,
    /// Event type
    pub event: Option<String>,
}

impl From<ActivityLog> for ActivityLogResponse {
    fn from(log: ActivityLog) -> Self {
        Self {
            id: log.id.to_string(),
            log_name: log.log_name,
            description: log.description,
            subject_type: log.subject_type,
            subject_id: log.subject_id,
            causer_type: log.causer_type,
            causer_id: log.causer_id,
            properties: log.properties,
            correlation_id: log.correlation_id.map(|id| id.to_string()),
            batch_uuid: log.batch_uuid,
            event: log.event,
            created_at: log.created_at.to_rfc3339(),
            updated_at: log.updated_at.to_rfc3339(),
        }
    }
}

/// List activity logs with filtering and pagination
#[utoipa::path(
    get,
    path = "/api/activity-logs",
    params(QueryParams),
    responses(
        (status = 200, description = "Activity logs retrieved successfully"),
        (status = 400, description = "Invalid query parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Activity Logs"
)]
pub async fn list_activity_logs(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    match <ActivityLog as QueryBuilderService<ActivityLog>>::index(Query(params), &pool) {
        Ok(result) => {
            (StatusCode::OK, Json(serde_json::json!(result))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch activity logs"
            }))).into_response()
        }
    }
}

/// Get a specific activity log by ID
#[utoipa::path(
    get,
    path = "/api/activity-logs/{id}",
    params(
        ("id" = String, Path, description = "Activity log ID")
    ),
    responses(
        (status = 200, description = "Activity log retrieved successfully", body = ActivityLogResponse),
        (status = 404, description = "Activity log not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Activity Logs"
)]
pub async fn get_activity_log(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let query_params = QueryParams {
        filter: {
            let mut filter = std::collections::HashMap::new();
            filter.insert("id".to_string(), serde_json::Value::String(id));
            filter
        },
        sort: None,
        include: None,
        fields: Default::default(),
        page: Some(1),
        per_page: Some(1),
        pagination_type: None,
        cursor: None,
        append: Default::default(),
    };

    match <ActivityLog as QueryBuilderService<ActivityLog>>::first(Query(query_params), &pool) {
        Ok(Some(activity)) => {
            (StatusCode::OK, Json(activity)).into_response()
        },
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "Activity log not found"
            }))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch activity log"
            }))).into_response()
        }
    }
}

/// Get activities by correlation ID
#[utoipa::path(
    get,
    path = "/api/activity-logs/correlation/{correlation_id}",
    params(
        ("correlation_id" = String, Path, description = "Correlation ID")
    ),
    responses(
        (status = 200, description = "Correlated activities retrieved successfully"),
        (status = 400, description = "Invalid correlation ID"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Activity Logs"
)]
pub async fn get_activities_by_correlation(
    State(pool): State<DbPool>,
    Path(correlation_id): Path<String>,
) -> impl IntoResponse {
    // Validate correlation ID format
    if correlation_id.parse::<ulid::Ulid>().is_err() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Invalid correlation ID format"
        }))).into_response();
    }

    let query_params = QueryParams {
        filter: {
            let mut filter = std::collections::HashMap::new();
            filter.insert("correlation_id".to_string(), serde_json::Value::String(correlation_id));
            filter
        },
        sort: Some("created_at".to_string()),
        include: None,
        fields: Default::default(),
        page: Some(1),
        per_page: Some(100),
        pagination_type: None,
        cursor: None,
        append: Default::default(),
    };

    match <ActivityLog as QueryBuilderService<ActivityLog>>::all(Query(query_params), &pool) {
        Ok(activities) => {
            (StatusCode::OK, Json(activities)).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch activities by correlation ID"
            }))).into_response()
        }
    }
}

/// Get activities by batch UUID
#[utoipa::path(
    get,
    path = "/api/activity-logs/batch/{batch_uuid}",
    params(
        ("batch_uuid" = String, Path, description = "Batch UUID")
    ),
    responses(
        (status = 200, description = "Batch activities retrieved successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Activity Logs"
)]
pub async fn get_activities_by_batch(
    State(pool): State<DbPool>,
    Path(batch_uuid): Path<String>,
) -> impl IntoResponse {
    let query_params = QueryParams {
        filter: {
            let mut filter = std::collections::HashMap::new();
            filter.insert("batch_uuid".to_string(), serde_json::Value::String(batch_uuid));
            filter
        },
        sort: Some("created_at".to_string()),
        include: None,
        fields: Default::default(),
        page: Some(1),
        per_page: Some(100),
        pagination_type: None,
        cursor: None,
        append: Default::default(),
    };

    match <ActivityLog as QueryBuilderService<ActivityLog>>::all(Query(query_params), &pool) {
        Ok(activities) => {
            (StatusCode::OK, Json(activities)).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch activities by batch UUID"
            }))).into_response()
        }
    }
}

/// Get activity log statistics
#[utoipa::path(
    get,
    path = "/api/activity-logs/stats",
    responses(
        (status = 200, description = "Activity log statistics", body = ActivityLogStats),
        (status = 500, description = "Internal server error")
    ),
    tag = "Activity Logs"
)]
pub async fn get_activity_stats(
    State(pool): State<DbPool>,
) -> impl IntoResponse {
    // Get total count using QueryBuilderService
    let total_query_params = QueryParams::default();
    let total_activities = match <ActivityLog as QueryBuilderService<ActivityLog>>::count(Query(total_query_params), &pool) {
        Ok(count) => count as u64,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to get total activity count"
        }))).into_response(),
    };

    // For simplicity, return basic stats
    // In a real implementation, you'd want to use custom SQL queries for aggregations
    let stats = ActivityLogStats {
        total_activities,
        by_log_name: HashMap::new(),
        by_event: HashMap::new(),
        by_subject_type: HashMap::new(),
        recent_count: 0, // Would need custom implementation for this
    };

    (StatusCode::OK, Json(stats)).into_response()
}

/// Create a new activity log entry
#[utoipa::path(
    post,
    path = "/api/activity-logs",
    request_body = CreateActivityLogRequest,
    responses(
        (status = 201, description = "Activity log created successfully"),
        (status = 400, description = "Invalid request body"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Activity Logs"
)]
pub async fn create_activity_log(
    State(pool): State<DbPool>,
    correlation_context: Option<Extension<CorrelationContext>>,
    Json(request): Json<CreateActivityLogRequest>,
) -> impl IntoResponse {
    let service = ActivityLogService::with_pool(pool);

    let correlation_id = correlation_context
        .map(|ctx| ctx.correlation_id)
        .unwrap_or_else(|| crate::app::models::DieselUlid::new());

    let new_activity = NewActivityLog {
        log_name: request.log_name,
        description: request.description,
        subject_type: request.subject_type,
        subject_id: request.subject_id,
        causer_type: request.causer_type,
        causer_id: request.causer_id,
        properties: request.properties,
        correlation_id: Some(correlation_id),
        batch_uuid: None,
        event: request.event,
    };

    match service.create(new_activity).await {
        Ok(activity) => {
            (StatusCode::CREATED, Json(ActivityLogResponse::from(activity))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to create activity log"
            }))).into_response()
        }
    }
}

/// Get activities for a specific subject
#[utoipa::path(
    get,
    path = "/api/activity-logs/subject/{subject_type}/{subject_id}",
    params(
        ("subject_type" = String, Path, description = "Subject type (e.g., User, Organization)"),
        ("subject_id" = String, Path, description = "Subject ID")
    ),
    responses(
        (status = 200, description = "Subject activities retrieved successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Activity Logs"
)]
pub async fn get_activities_by_subject(
    State(pool): State<DbPool>,
    Path((subject_type, subject_id)): Path<(String, String)>,
    Query(mut query_params): Query<QueryParams>,
) -> impl IntoResponse {
    // Add subject filters to existing query parameters
    query_params.filter.insert("subject_type".to_string(), serde_json::Value::String(subject_type));
    query_params.filter.insert("subject_id".to_string(), serde_json::Value::String(subject_id));

    // Set default pagination if not provided
    if query_params.per_page.is_none() {
        query_params.per_page = Some(50);
    }

    match <ActivityLog as QueryBuilderService<ActivityLog>>::index(Query(query_params), &pool) {
        Ok(result) => {
            (StatusCode::OK, Json(result)).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch activities by subject"
            }))).into_response()
        }
    }
}

/// Get activities caused by a specific causer
#[utoipa::path(
    get,
    path = "/api/activity-logs/causer/{causer_type}/{causer_id}",
    params(
        ("causer_type" = String, Path, description = "Causer type (e.g., User)"),
        ("causer_id" = String, Path, description = "Causer ID")
    ),
    responses(
        (status = 200, description = "Causer activities retrieved successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Activity Logs"
)]
pub async fn get_activities_by_causer(
    State(pool): State<DbPool>,
    Path((causer_type, causer_id)): Path<(String, String)>,
    Query(mut query_params): Query<QueryParams>,
) -> impl IntoResponse {
    // Add causer filters to existing query parameters
    query_params.filter.insert("causer_type".to_string(), serde_json::Value::String(causer_type));
    query_params.filter.insert("causer_id".to_string(), serde_json::Value::String(causer_id));

    // Set default pagination if not provided
    if query_params.per_page.is_none() {
        query_params.per_page = Some(50);
    }

    match <ActivityLog as QueryBuilderService<ActivityLog>>::index(Query(query_params), &pool) {
        Ok(result) => {
            (StatusCode::OK, Json(result)).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch activities by causer"
            }))).into_response()
        }
    }
}