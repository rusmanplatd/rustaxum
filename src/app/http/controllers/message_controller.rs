use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::Serialize;
use crate::database::DbPool;

use crate::app::models::message::{Message};
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
    path = "/api/messages",
    tag = "Messages",
    summary = "List all messages with conversation filtering and secure messaging support",
    description = "Retrieve messages with Laravel-style querying, conversation filtering, end-to-end encryption support, and message thread organization. Essential for secure messaging applications and chat functionality.",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Items per page (default: 15, max: 100). Use 25-50 for chat interfaces, 100 for message exports"),
        ("sort" = Option<String>, Query, description = "Multi-column sorting with conversation and timestamp support. Available fields: id, conversation_id, sender_user_id, message_type, is_edited, is_deleted, expires_at, sent_at, created_at, updated_at, deleted_at. Syntax: 'field1,-field2,field3:desc'. Examples: 'conversation_id,sent_at', '-sent_at,conversation_id', 'is_edited:desc,sent_at'"),
        ("include" = Option<String>, Query, description = "Eager load relationships for complete message context. Available: conversation, sender, sender_device, reply_to, forward_from, edit_of, mentions, createdBy, updatedBy, deletedBy, createdBy.organizations.position.level, updatedBy.organizations.position.level, deletedBy.organizations.position.level. Examples: 'conversation,sender', 'reply_to,forward_from,createdBy.organizations.position.level'"),
        ("filter" = Option<serde_json::Value>, Query, description = "Advanced filtering with 15+ operators for message management. Available filters: id, conversation_id, sender_user_id, sender_device_id, message_type, reply_to_message_id, forward_from_message_id, edit_of_message_id, is_edited, is_deleted, expires_at, sent_at, created_at, updated_at, deleted_at. Operators: eq, ne, gt, gte, lt, lte, like, ilike, contains, starts_with, ends_with, in, not_in, is_null, is_not_null, between. Examples: filter[conversation_id][eq]=conv123, filter[message_type][contains]=text, filter[is_deleted][eq]=false, filter[sent_at][between]=2023-01-01,2023-12-31"),
        ("fields" = Option<String>, Query, description = "Field selection for optimized message queries and security. Available: id, conversation_id, sender_user_id, sender_device_id, message_type, encrypted_content, content_algorithm, reply_to_message_id, forward_from_message_id, edit_of_message_id, is_edited, is_deleted, expires_at, sent_at, created_at, updated_at, deleted_at. Examples: fields[messages]=id,conversation_id,message_type,sent_at"),
        ("cursor" = Option<String>, Query, description = "Cursor for high-performance pagination with timestamp indexing"),
        ("pagination_type" = Option<String>, Query, description = "Pagination strategy: 'offset' (traditional) or 'cursor' (high-performance for large message datasets, recommended default)"),
    ),
    responses(
        (status = 200, description = "List of messages with metadata", body = Vec<crate::app::models::message::MessageResponse>),
        (status = 400, description = "Invalid query parameters", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    match <Message as QueryBuilderService<Message>>::index(Query(params), &pool) {
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
    path = "/api/messages/{id}",
    tag = "Messages",
    summary = "Get message by ID",
    description = "Retrieve a specific message by its unique identifier",
    params(
        ("id" = String, Path, description = "Message unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Message details", body = crate::app::models::message::MessageResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Message not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn show(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    use diesel::prelude::*;
    use crate::schema::messages;

    match pool.get() {
        Ok(mut conn) => {
            match messages::table
                .find(id)
                .select(Message::as_select())
                .first::<Message>(&mut conn)
                .optional()
            {
                Ok(Some(message)) => (StatusCode::OK, ResponseJson(message.to_response())).into_response(),
                Ok(None) => {
                    let error = ErrorResponse {
                        error: "Message not found".to_string(),
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