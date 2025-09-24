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
        ("sort" = Option<String>, Query, description = "Sort field and direction. Available fields: id, name, email, status, created_at, updated_at (prefix with '-' for descending)"),
        ("include" = Option<String>, Query, description = "Comma-separated list of relationships to include. Available: organizations, roles"),
        ("filter" = Option<serde_json::Value>, Query, description = "Filter parameters. Available filters: name, email, status (e.g., filter[name]=John, filter[status]=active)"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to select. Available: id, name, email, status, created_at, updated_at"),
        ("cursor" = Option<String>, Query, description = "Cursor for cursor-based pagination"),
        ("pagination_type" = Option<String>, Query, description = "Pagination type: 'offset' or 'cursor' (default: cursor)"),
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