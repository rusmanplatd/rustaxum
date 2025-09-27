use axum::{
    extract::{Path, State, Query, Request},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use crate::database::DbPool;
use crate::app::query_builder::{QueryParams, QueryBuilderService};
use crate::app::models::user::User;
use crate::app::services::user_service::UserService;
use crate::app::http::middleware::activity_logging_middleware::activity_logger_from_request;

#[utoipa::path(
    get,
    path = "/api/users",
    tag = "Users",
    summary = "List all users",
    description = "Get all users with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Multi-column sorting. Available fields: id, name, email, status, created_at, updated_at. Syntax: 'field1,-field2,field3:desc'. Example: 'name,-created_at,status:asc'"),
        ("include" = Option<String>, Query, description = "Eager load relationships. Available: organizations, roles, organizations.position, organizations.position.level, roles.permissions, createdBy, updatedBy, deletedBy, createdBy.organizations.position.level, updatedBy.organizations.position.level, deletedBy.organizations.position.level. Example: 'organizations.position,roles,createdBy.organizations.position.level'"),
        ("filter" = Option<serde_json::Value>, Query, description = "Advanced filtering with 15+ operators. Available filters: name, email, status, created_at, updated_at, email_verified_at. Operators: eq, ne, gt, gte, lt, lte, like, ilike, contains, starts_with, ends_with, in, not_in, is_null, is_not_null, between. Examples: filter[name][contains]=john, filter[status][in]=active,verified, filter[email_verified_at][is_not_null]=true"),
        ("fields" = Option<String>, Query, description = "Field selection for performance optimization. Available: id, name, email, status, created_at, updated_at, email_verified_at. Example: fields[users]=id,name,email"),
        ("cursor" = Option<String>, Query, description = "Cursor for high-performance pagination. Base64-encoded JSON cursor from previous response"),
        ("pagination_type" = Option<String>, Query, description = "Pagination strategy: 'offset' (traditional) or 'cursor' (high-performance, default)"),
    ),
    responses(
        (status = 200, description = "List of users", body = Vec<crate::app::models::user::UserResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    match <User as QueryBuilderService<User>>::index(Query(params), &pool) {
        Ok(result) => {
            (StatusCode::OK, Json(serde_json::json!(result))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch users"
            }))).into_response()
        }
    }
}

/// Get a specific user by ID
///
/// Retrieve detailed information about a specific user using their unique identifier.
/// Sensitive information like passwords and tokens are excluded from the response.
///
/// # Path Parameters
/// - `id`: The unique identifier of the user (ULID format)
#[utoipa::path(
    get,
    path = "/api/users/{id}",
    tag = "Users",
    summary = "Get user by ID",
    description = "Retrieve a specific user by their unique identifier",
    params(
        ("id" = String, Path, description = "User unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "User details", body = crate::app::models::user::UserResponse),
        (status = 404, description = "User not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn show(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    request: Request,
) -> impl IntoResponse {

    match UserService::find_by_id(&pool, id.clone())
    {
        Ok(Some(user)) => {
            // Log user view activity
            let logger = activity_logger_from_request(&request, "user_access");
            if let Err(e) = logger.log_view("User", &id, Some(json!({
                "user_name": user.name,
                "user_email": user.email
            }))).await {
                eprintln!("Failed to log user view activity: {}", e);
            }

            (StatusCode::OK, Json(serde_json::json!(user.to_response()))).into_response()
        },
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "User not found"
            }))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch user"
            }))).into_response()
        }
    }
}