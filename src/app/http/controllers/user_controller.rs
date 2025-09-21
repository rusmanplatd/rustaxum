use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use ulid::Ulid;
use crate::database::DbPool;
use crate::app::query_builder::{QueryParams, QueryBuilderService};
use crate::app::models::user::User;
use crate::app::services::user_service::UserService;

#[utoipa::path(
    get,
    path = "/api/users",
    tag = "Users",
    summary = "List all users",
    description = "Get all users with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination"),
        ("limit" = Option<u32>, Query, description = "Number of items per page (max 100)"),
        ("sort" = Option<String>, Query, description = "Sort field"),
        ("direction" = Option<String>, Query, description = "Sort direction (asc/desc)"),
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
) -> impl IntoResponse {
    let user_ulid = match Ulid::from_string(&id) {
        Ok(ulid) => ulid,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid user ID format"
            }))).into_response();
        }
    };

    match UserService::find_by_id(&pool, user_ulid.to_string())
    {
        Ok(Some(user)) => {
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