use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::Serialize;
use crate::database::DbPool;

use crate::app::models::notification::{UpdateNotification, Notification};
use crate::app::http::requests::{CreateNotificationRequest, UpdateNotificationRequest};
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
    path = "/api/notifications",
    tag = "Notifications",
    summary = "List all notifications with multi-channel filtering and status tracking",
    description = "Retrieve notifications with Laravel-style querying, multi-channel filtering, read status tracking, and priority-based sorting. Essential for notification center functionality and user engagement management.",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Items per page (default: 15, max: 100). Use 50-100 for admin dashboards, 10-25 for user notification centers"),
        ("sort" = Option<String>, Query, description = "Multi-column sorting with priority and timestamp support. Available fields: id, notification_type, notifiable_id, notifiable_type, read_at, created_at, updated_at, sent_at, failed_at, scheduled_at, priority, retry_count, max_retries. Syntax: 'field1,-field2,field3:desc'. Examples: 'priority,-created_at', 'read_at:asc,priority:desc', '-scheduled_at,notification_type'"),
        ("include" = Option<String>, Query, description = "Eager load relationships for complete notification context. Available: notifiable, createdBy, updatedBy, deletedBy, createdBy.organizations.position.level, updatedBy.organizations.position.level, deletedBy.organizations.position.level. Examples: 'notifiable,createdBy.organizations.position.level'"),
        ("filter" = Option<serde_json::Value>, Query, description = "Advanced filtering with 15+ operators for notification management. Available filters: id, notification_type, notifiable_id, notifiable_type, read_at, created_at, updated_at, sent_at, failed_at, scheduled_at, priority, retry_count, max_retries. Operators: eq, ne, gt, gte, lt, lte, like, ilike, contains, starts_with, ends_with, in, not_in, is_null, is_not_null, between. Examples: filter[read_at][is_null]=true, filter[notification_type][contains]=invoice, filter[priority][gte]=3, filter[created_at][between]=2023-01-01,2023-12-31"),
        ("fields" = Option<String>, Query, description = "Field selection for optimized notification queries. Available: id, notification_type, notifiable_id, notifiable_type, data, read_at, created_at, updated_at, sent_at, failed_at, scheduled_at, priority, retry_count, max_retries. Examples: fields[notifications]=id,notification_type,read_at,data"),
        ("cursor" = Option<String>, Query, description = "Cursor for high-performance pagination with timestamp indexing"),
        ("pagination_type" = Option<String>, Query, description = "Pagination strategy: 'offset' (traditional) or 'cursor' (high-performance for large notification datasets, recommended default)"),
    ),
    responses(
        (status = 200, description = "List of notifications with metadata", body = Vec<crate::app::models::notification::NotificationResponse>),
        (status = 400, description = "Invalid query parameters", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    match <Notification as QueryBuilderService<Notification>>::index(Query(params), &pool) {
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
    path = "/api/notifications/{id}",
    tag = "Notifications",
    summary = "Get notification by ID",
    description = "Retrieve a specific notification by its unique identifier",
    params(
        ("id" = String, Path, description = "Notification unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Notification details", body = crate::app::models::notification::NotificationResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Notification not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn show(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    use diesel::prelude::*;
    use crate::schema::notifications;

    match pool.get() {
        Ok(mut conn) => {
            match notifications::table
                .find(id)
                .select(Notification::as_select())
                .first::<Notification>(&mut conn)
                .optional()
            {
                Ok(Some(notification)) => (StatusCode::OK, ResponseJson(notification.to_response())).into_response(),
                Ok(None) => {
                    let error = ErrorResponse {
                        error: "Notification not found".to_string(),
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

/// Create a new notification
///
/// Create a new notification with the provided information. All required fields must be provided
/// and will be validated according to the business rules.
#[utoipa::path(
    post,
    path = "/api/notifications",
    tag = "Notifications",
    summary = "Create new notification",
    description = "Create a new notification with the provided information",
    request_body = crate::app::http::requests::CreateNotificationRequest,
    responses(
        (status = 201, description = "Notification created successfully", body = crate::app::models::notification::NotificationResponse),
        (status = 400, description = "Validation error or bad request", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn store(State(pool): State<DbPool>, request: CreateNotificationRequest) -> impl IntoResponse {
    use diesel::prelude::*;
    use crate::schema::notifications;
    use crate::app::models::notification::NewNotification;

    let new_notification = NewNotification::new(
        request.notification_type,
        request.notifiable_id,
        request.notifiable_type,
        request.data,
    );

    match pool.get() {
        Ok(mut conn) => {
            match diesel::insert_into(notifications::table)
                .values(&new_notification)
                .returning(Notification::as_select())
                .get_result::<Notification>(&mut conn)
            {
                Ok(notification) => (StatusCode::CREATED, ResponseJson(notification.to_response())).into_response(),
                Err(e) => {
                    let error = ErrorResponse {
                        error: e.to_string(),
                    };
                    (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
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

/// Update an existing notification
///
/// Update an existing notification with the provided information. Only provided fields will be updated.
#[utoipa::path(
    put,
    path = "/api/notifications/{id}",
    tag = "Notifications",
    summary = "Update notification",
    description = "Update an existing notification with the provided information",
    params(
        ("id" = String, Path, description = "Notification unique identifier (ULID format)")
    ),
    request_body = crate::app::http::requests::UpdateNotificationRequest,
    responses(
        (status = 200, description = "Notification updated successfully", body = crate::app::models::notification::NotificationResponse),
        (status = 400, description = "Invalid ID format or validation error", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Notification not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    request: UpdateNotificationRequest,
) -> impl IntoResponse {
    let payload = UpdateNotification {
        read_at: request.read_at,
    };

    use diesel::prelude::*;
    use crate::schema::notifications;

    match pool.get() {
        Ok(mut conn) => {
            match diesel::update(notifications::table.find(&id))
                .set(&payload)
                .returning(Notification::as_select())
                .get_result::<Notification>(&mut conn)
            {
                Ok(notification) => (StatusCode::OK, ResponseJson(notification.to_response())).into_response(),
                Err(e) => {
                    let error = ErrorResponse {
                        error: e.to_string(),
                    };
                    (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
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

/// Delete a notification
///
/// Permanently delete a notification from the system. This action cannot be undone.
#[utoipa::path(
    delete,
    path = "/api/notifications/{id}",
    tag = "Notifications",
    summary = "Delete notification",
    description = "Permanently delete a notification from the system",
    params(
        ("id" = String, Path, description = "Notification unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Notification deleted successfully", body = crate::app::docs::MessageResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Notification not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn destroy(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    use diesel::prelude::*;
    use crate::schema::notifications;

    match pool.get() {
        Ok(mut conn) => {
            match diesel::delete(notifications::table.find(&id))
                .execute(&mut conn)
            {
                Ok(_) => {
                    let message = MessageResponse {
                        message: "Notification deleted successfully".to_string(),
                    };
                    (StatusCode::OK, ResponseJson(message)).into_response()
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