use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::Serialize;
use crate::database::DbPool;

use crate::app::models::session::{SessionModel};
use crate::app::query_builder::{QueryParams, QueryBuilderService};

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

#[utoipa::path(
    get,
    path = "/api/session-models",
    tag = "Session Models",
    summary = "List all database sessions with user tracking and activity monitoring",
    description = "Retrieve session models with Laravel-style querying, user activity tracking, IP-based filtering, and session security management. Essential for user session management and security auditing.",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Items per page (default: 15, max: 100). Use 50-100 for admin dashboards, 10-25 for user interfaces"),
        ("sort" = Option<String>, Query, description = "Multi-column sorting with activity and user support. Available fields: id, user_id, ip_address, last_activity. Syntax: 'field1,-field2,field3:desc'. Examples: 'last_activity:desc', 'user_id,last_activity', '-last_activity,ip_address'"),
        ("include" = Option<String>, Query, description = "Eager load relationships for complete session context. Available: user, createdBy, updatedBy, deletedBy, createdBy.organizations.position.level, updatedBy.organizations.position.level, deletedBy.organizations.position.level. Examples: 'user,createdBy.organizations.position.level'"),
        ("filter" = Option<serde_json::Value>, Query, description = "Advanced filtering with 15+ operators for session management. Available filters: id, user_id, ip_address, user_agent, last_activity. Operators: eq, ne, gt, gte, lt, lte, like, ilike, contains, starts_with, ends_with, in, not_in, is_null, is_not_null, between. Examples: filter[user_id][is_not_null]=true, filter[ip_address][contains]=192.168, filter[last_activity][gte]=1640995200, filter[user_agent][contains]=Chrome"),
        ("fields" = Option<String>, Query, description = "Field selection for optimized session queries and security. Available: id, user_id, ip_address, user_agent, payload, last_activity. Examples: fields[sessions]=id,user_id,ip_address,last_activity"),
        ("cursor" = Option<String>, Query, description = "Cursor for high-performance pagination with activity indexing"),
        ("pagination_type" = Option<String>, Query, description = "Pagination strategy: 'offset' (traditional) or 'cursor' (high-performance for large session datasets, recommended default)"),
    ),
    responses(
        (status = 200, description = "List of sessions with metadata", body = Vec<crate::app::models::session::SessionResponse>),
        (status = 400, description = "Invalid query parameters", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    match <SessionModel as QueryBuilderService<SessionModel>>::index(Query(params), &pool) {
        Ok(result) => {
            (StatusCode::OK, ResponseJson(serde_json::json!(result))).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/session-models/{id}",
    tag = "Session Models",
    summary = "Get session by ID",
    description = "Retrieve a specific session by its unique identifier",
    params(
        ("id" = String, Path, description = "Session unique identifier")
    ),
    responses(
        (status = 200, description = "Session details", body = crate::app::models::session::SessionResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Session not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn show(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    use diesel::prelude::*;
    use crate::schema::sessions;

    match pool.get() {
        Ok(mut conn) => {
            match sessions::table
                .find(id)
                .select(SessionModel::as_select())
                .first::<SessionModel>(&mut conn)
                .optional()
            {
                Ok(Some(session)) => (StatusCode::OK, ResponseJson(session.to_response())).into_response(),
                Ok(None) => {
                    let error = ErrorResponse {
                        error: "Session not found".to_string(),
                    };
                    (StatusCode::NOT_FOUND, ResponseJson(error)).into_response()
                }
                Err(e) => {
                    let error = ErrorResponse {
                        error: e.to_string(),
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
                }
            }
        }
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}