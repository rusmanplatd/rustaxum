use axum::{
    extract::{Json, State, Path, Query},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json as ResponseJson},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::database::DbPool;

use crate::app::services::oauth::TokenService;
use crate::app::services::auth_service::AuthService;
use crate::app::utils::token_utils::TokenUtils;
use crate::app::query_builder::{QueryParams, QueryBuilderService};
use crate::app::models::oauth::{AccessToken};

#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreatePersonalAccessTokenRequest {
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_in_seconds: Option<i64>,
}

#[utoipa::path(
    post,
    path = "/oauth/personal-access-tokens",
    tags = ["Personal Access Tokens"],
    summary = "Create personal access token",
    description = "Create a new personal access token for API access",
    request_body = CreatePersonalAccessTokenRequest,
    responses(
        (status = 201, description = "Token created successfully", body = crate::app::docs::oauth::PersonalAccessTokenResponse),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn create_personal_access_token(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Json(payload): Json<CreatePersonalAccessTokenRequest>,
) -> impl IntoResponse {
    let user_id = match get_authenticated_user(&pool, &headers).await {
        Ok(user_id) => user_id,
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    match TokenService::create_personal_access_token(
        &pool,
        user_id,
        payload.name,
        payload.scopes,
        payload.expires_in_seconds,
    ).await {
        Ok(response) => (StatusCode::CREATED, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/oauth/personal-access-tokens",
    tags = ["Personal Access Tokens"],
    summary = "List personal access tokens",
    description = "Get list of personal access tokens for authenticated user with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Sort field and direction. Available fields: id, name, created_at, updated_at (prefix with '-' for descending)"),
        ("filter" = Option<serde_json::Value>, Query, description = "Filter parameters. Available filters: name, revoked (e.g., filter[name]=MyToken, filter[revoked]=false)"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to select. Available: id, name, scopes, revoked, created_at, updated_at"),
        ("cursor" = Option<String>, Query, description = "Cursor for cursor-based pagination"),
        ("pagination_type" = Option<String>, Query, description = "Pagination type: 'offset' or 'cursor' (default: cursor)"),
    ),
    responses(
        (status = 200, description = "List of personal access tokens", body = Vec<crate::app::docs::oauth::PersonalAccessTokenResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn list_personal_access_tokens(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Query(mut params): Query<QueryParams>,
) -> impl IntoResponse {
    let user_id = match get_authenticated_user(&pool, &headers).await {
        Ok(user_id) => user_id,
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    // Add user_id filter to ensure users only see their own tokens
    params.filter.insert("user_id".to_string(), serde_json::json!(user_id));

    match <AccessToken as QueryBuilderService<AccessToken>>::index(Query(params), &pool) {
        Ok(result) => {
            (StatusCode::OK, ResponseJson(serde_json::json!(result))).into_response()
        },
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/oauth/personal-access-tokens/{token_id}",
    tags = ["Personal Access Tokens"],
    summary = "Revoke personal access token",
    description = "Revoke a personal access token by its ID",
    params(
        ("token_id" = String, Path, description = "Token identifier")
    ),
    responses(
        (status = 200, description = "Token revoked successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Token not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn revoke_personal_access_token(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(token_id): Path<String>,
) -> impl IntoResponse {
    let user_id = match get_authenticated_user(&pool, &headers).await {
        Ok(user_id) => user_id,
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    // Verify the token belongs to the user
    match TokenService::find_access_token_by_id(&pool, token_id.clone()) {
        Ok(Some(token)) => {
            if let Some(owner_id) = token.user_id {
                if owner_id != user_id.to_string() {
                    let error = ErrorResponse {
                        error: "Access denied".to_string(),
                    };
                    return (StatusCode::FORBIDDEN, ResponseJson(error)).into_response();
                }
            }
        },
        Ok(None) => {
            let error = ErrorResponse {
                error: "Token not found".to_string(),
            };
            return (StatusCode::NOT_FOUND, ResponseJson(error)).into_response();
        },
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    }

    match TokenService::revoke_access_token(&pool, token_id) {
        Ok(_) => {
            #[derive(Serialize)]
            struct RevokeResponse {
                message: String,
            }

            let response = RevokeResponse {
                message: "Token revoked successfully".to_string(),
            };

            (StatusCode::OK, ResponseJson(response)).into_response()
        },
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

async fn get_authenticated_user(_pool: &DbPool, headers: &HeaderMap) -> anyhow::Result<String> {
    let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());
    let token = TokenUtils::extract_token_from_header(auth_header)?;
    let claims = AuthService::decode_token(token)?;

    Ok(claims.sub)
}