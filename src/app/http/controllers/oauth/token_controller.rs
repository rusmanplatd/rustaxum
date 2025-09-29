use axum::{
    extract::{Json, State, Path, Query},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json as ResponseJson},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::database::DbPool;
use chrono::{Utc};

use crate::app::services::oauth::TokenService;
use crate::app::services::auth_service::AuthService;
use crate::app::utils::token_utils::TokenUtils;
use crate::app::query_builder::{QueryParams, QueryBuilderService};
use crate::app::models::oauth::{AccessToken};

#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
    error_description: Option<String>,
}

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

#[derive(Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ListTokensQuery {
    pub user_id: Option<String>,
    pub client_id: Option<String>,
    pub scope: Option<String>,
    pub active_only: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, ToSchema, utoipa::IntoParams)]
pub struct TokenStatsQuery {
    pub days: Option<i32>,
}

#[derive(Serialize)]
pub struct TokenStatsResponse {
    pub total_tokens: i64,
    pub active_tokens: i64,
    pub expired_tokens: i64,
    pub revoked_tokens: i64,
    pub tokens_by_client: Vec<ClientTokenStats>,
    pub tokens_by_scope: Vec<ScopeTokenStats>,
    pub daily_token_creation: Vec<DailyTokenStats>,
}

#[derive(Serialize)]
pub struct ClientTokenStats {
    pub client_id: String,
    pub client_name: String,
    pub token_count: i64,
    pub active_count: i64,
}

#[derive(Serialize)]
pub struct ScopeTokenStats {
    pub scope: String,
    pub token_count: i64,
}

#[derive(Serialize)]
pub struct DailyTokenStats {
    pub date: String,
    pub count: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct RevokeTokensRequest {
    pub token_ids: Vec<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct ExtendTokenRequest {
    pub expires_in_seconds: i64,
}

/// List access tokens with filtering and pagination
#[utoipa::path(
    get,
    path = "/oauth/tokens",
    tags = ["OAuth Tokens"],
    summary = "List OAuth tokens",
    description = "Get list of OAuth tokens with filtering options (admin only)",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Sort field and direction. Available fields: id, name, expires_at, created_at, updated_at (prefix with '-' for descending)"),
        ("filter" = Option<serde_json::Value>, Query, description = "Filter parameters. Available filters: user_id, client_id, name, revoked (e.g., filter[user_id]=01ABC123, filter[revoked]=false)"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to select. Available: id, user_id, client_id, name, scopes, revoked, expires_at, created_at, updated_at"),
        ("cursor" = Option<String>, Query, description = "Cursor for cursor-based pagination"),
        ("pagination_type" = Option<String>, Query, description = "Pagination type: 'offset' or 'cursor' (default: cursor)"),
    ),
    responses(
        (status = 200, description = "List of tokens", body = Vec<crate::app::docs::oauth::AccessTokenResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("Bearer" = []))
)]
pub async fn list_tokens(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    // Verify admin access for listing all tokens
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: "unauthorized".to_string(),
            error_description: Some(e.to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    match <AccessToken as QueryBuilderService<AccessToken>>::index(Query(params), &pool) {
        Ok(result) => {
            (StatusCode::OK, ResponseJson(serde_json::json!(result))).into_response()
        },
        Err(e) => {
            let error = ErrorResponse {
                error: "server_error".to_string(),
                error_description: Some(e.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

/// Get detailed information about a specific token
#[utoipa::path(
    get,
    path = "/oauth/tokens/{token_id}",
    tags = ["OAuth Tokens"],
    summary = "Get OAuth token",
    description = "Get a specific OAuth token by ID",
    params(
        ("token_id" = String, Path, description = "Token identifier")
    ),
    responses(
        (status = 200, description = "Token found", body = crate::app::docs::oauth::AccessTokenResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Token not found", body = ErrorResponse)
    ),
    security(("Bearer" = []))
)]
pub async fn get_token(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(token_id): Path<String>,
) -> impl IntoResponse {
    let user_id = match get_authenticated_user(&pool, &headers).await {
        Ok(user_id) => user_id,
        Err(e) => {
            let error = ErrorResponse {
                error: "unauthorized".to_string(),
                error_description: Some(e.to_string()),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    match TokenService::find_access_token_by_id(&pool, token_id) {
        Ok(Some(token)) => {
            // Check if user owns this token or is admin
            if let Some(ref owner_id) = token.user_id {
                if owner_id != &user_id {
                    // Check if user has admin privileges to view other users' tokens
                    if let Err(_) = crate::app::http::middleware::auth_middleware::verify_admin_access(&headers, &pool).await {
                        let error = ErrorResponse {
                            error: "access_denied".to_string(),
                            error_description: Some("You can only view your own tokens unless you have admin privileges".to_string()),
                        };
                        return (StatusCode::FORBIDDEN, ResponseJson(error)).into_response();
                    }
                }
            }

            #[derive(Serialize)]
            struct DetailedTokenResponse {
                #[serde(flatten)]
                token: crate::app::models::oauth::AccessTokenResponse,
                is_valid: bool,
                is_expired: bool,
                time_until_expiry: Option<String>,
            }

            let time_until_expiry = token.expires_at.map(|exp| {
                let now = Utc::now();
                if exp > now {
                    let duration = exp - now;
                    format!("{}h {}m", duration.num_hours(), duration.num_minutes() % 60)
                } else {
                    "Expired".to_string()
                }
            });

            let response = DetailedTokenResponse {
                token: token.to_response(),
                is_valid: token.is_valid(),
                is_expired: token.is_expired(),
                time_until_expiry,
            };

            (StatusCode::OK, ResponseJson(response)).into_response()
        },
        Ok(None) => {
            let error = ErrorResponse {
                error: "not_found".to_string(),
                error_description: Some("Token not found".to_string()),
            };
            (StatusCode::NOT_FOUND, ResponseJson(error)).into_response()
        },
        Err(e) => {
            let error = ErrorResponse {
                error: "server_error".to_string(),
                error_description: Some(e.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

/// Revoke a specific token
#[utoipa::path(
    delete,
    path = "/oauth/tokens/{token_id}",
    tags = ["OAuth Tokens"],
    summary = "Revoke OAuth token",
    description = "Revoke a specific OAuth token by ID",
    params(
        ("token_id" = String, Path, description = "Token identifier")
    ),
    responses(
        (status = 200, description = "Token revoked successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Token not found", body = ErrorResponse)
    ),
    security(("Bearer" = []))
)]
pub async fn revoke_token(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(token_id): Path<String>,
) -> impl IntoResponse {
    let user_id = match get_authenticated_user(&pool, &headers).await {
        Ok(user_id) => user_id,
        Err(e) => {
            let error = ErrorResponse {
                error: "unauthorized".to_string(),
                error_description: Some(e.to_string()),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    // Check if user owns this token
    match TokenService::find_access_token_by_id(&pool, token_id.clone()) {
        Ok(Some(token)) => {
            if let Some(ref owner_id) = token.user_id {
                if owner_id != &user_id {
                    let error = ErrorResponse {
                        error: "forbidden".to_string(),
                        error_description: Some("You can only revoke your own tokens".to_string()),
                    };
                    return (StatusCode::FORBIDDEN, ResponseJson(error)).into_response();
                }
            }
        },
        Ok(None) => {
            let error = ErrorResponse {
                error: "not_found".to_string(),
                error_description: Some("Token not found".to_string()),
            };
            return (StatusCode::NOT_FOUND, ResponseJson(error)).into_response();
        },
        Err(e) => {
            let error = ErrorResponse {
                error: "server_error".to_string(),
                error_description: Some(e.to_string()),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    }

    match TokenService::revoke_access_token(&pool, token_id) {
        Ok(_) => {
            let response = MessageResponse {
                message: "Token revoked successfully".to_string(),
            };
            (StatusCode::OK, ResponseJson(response)).into_response()
        },
        Err(e) => {
            let error = ErrorResponse {
                error: "server_error".to_string(),
                error_description: Some(e.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

/// Revoke multiple tokens at once (admin only)
#[utoipa::path(
    post,
    path = "/oauth/tokens/revoke-bulk",
    tags = ["OAuth Tokens"],
    summary = "Revoke multiple tokens",
    description = "Revoke multiple OAuth tokens at once (admin only)",
    request_body = RevokeTokensRequest,
    responses(
        (status = 200, description = "Tokens revoked successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse)
    ),
    security(("Bearer" = []))
)]
pub async fn revoke_tokens(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Json(payload): Json<RevokeTokensRequest>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: "unauthorized".to_string(),
            error_description: Some(e.to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    let mut revoked_count = 0;
    let mut errors = Vec::new();

    for token_id in payload.token_ids {
        match TokenService::revoke_access_token(&pool, token_id.clone()) {
            Ok(_) => revoked_count += 1,
            Err(e) => errors.push(format!("Token {}: {}", token_id, e)),
        }
    }

    #[derive(Serialize)]
    struct BulkRevokeResponse {
        revoked_count: i32,
        errors: Vec<String>,
        message: String,
    }

    let response = BulkRevokeResponse {
        revoked_count,
        errors,
        message: format!("Successfully revoked {} token(s)", revoked_count),
    };

    (StatusCode::OK, ResponseJson(response)).into_response()
}

/// Get token statistics (admin only)
#[utoipa::path(
    get,
    path = "/oauth/tokens/stats",
    tags = ["OAuth Tokens"],
    summary = "Get token statistics",
    description = "Get OAuth token usage statistics (admin only)",
    params(
        TokenStatsQuery
    ),
    responses(
        (status = 200, description = "Token statistics", body = crate::app::docs::oauth::TokenStats),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(("Bearer" = []))
)]
pub async fn get_token_stats(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Query(_params): Query<TokenStatsQuery>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: "unauthorized".to_string(),
            error_description: Some(e.to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    // Get token statistics from database
    let token_stats = get_token_statistics(&pool).await
        .map_err(|e| {
            tracing::error!("Failed to get token statistics: {}", e);
            OAuthError::server_error("Failed to retrieve token statistics")
        })?;

    let response = TokenStatsResponse {
        total_tokens: token_stats.total,
        active_tokens: token_stats.active,
        expired_tokens: token_stats.expired,
        revoked_tokens: token_stats.revoked,
        tokens_by_client: token_stats.by_client,
        tokens_by_scope: vec![],
        daily_token_creation: vec![],
    };

    let error = ErrorResponse {
        error: "not_implemented".to_string(),
        error_description: Some("Token statistics endpoint not yet fully implemented".to_string()),
    };
    (StatusCode::NOT_IMPLEMENTED, ResponseJson(error)).into_response()
}

/// Extend token expiration (admin only)
#[utoipa::path(
    patch,
    path = "/oauth/tokens/{token_id}/extend",
    tags = ["OAuth Tokens"],
    summary = "Extend token expiration",
    description = "Extend the expiration time of an OAuth token (admin only)",
    params(
        ("token_id" = String, Path, description = "Token identifier")
    ),
    request_body = ExtendTokenRequest,
    responses(
        (status = 200, description = "Token extended successfully", body = crate::app::docs::oauth::AccessTokenResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Token not found", body = ErrorResponse)
    ),
    security(("Bearer" = []))
)]
pub async fn extend_token(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(_token_id): Path<String>,
    Json(_payload): Json<ExtendTokenRequest>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: "unauthorized".to_string(),
            error_description: Some(e.to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    // This would need to be implemented in TokenService
    let error = ErrorResponse {
        error: "not_implemented".to_string(),
        error_description: Some("Token extension endpoint not yet implemented".to_string()),
    };
    (StatusCode::NOT_IMPLEMENTED, ResponseJson(error)).into_response()
}

/// Get current user's tokens
#[utoipa::path(
    get,
    path = "/oauth/my-tokens",
    tags = ["OAuth Tokens"],
    summary = "Get my tokens",
    description = "Get OAuth tokens for the authenticated user with optional filtering",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Sort field and direction. Available fields: id, name, expires_at, created_at, updated_at (prefix with '-' for descending)"),
        ("filter" = Option<serde_json::Value>, Query, description = "Filter parameters. Available filters: client_id, name, revoked (e.g., filter[client_id]=01ABC123, filter[revoked]=false)"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to select. Available: id, client_id, name, scopes, revoked, expires_at, created_at, updated_at"),
        ("cursor" = Option<String>, Query, description = "Cursor for cursor-based pagination"),
        ("pagination_type" = Option<String>, Query, description = "Pagination type: 'offset' or 'cursor' (default: cursor)"),
    ),
    responses(
        (status = 200, description = "List of user's tokens", body = Vec<crate::app::docs::oauth::AccessTokenResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("Bearer" = []))
)]
pub async fn get_my_tokens(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Query(mut params): Query<QueryParams>,
) -> impl IntoResponse {
    let user_id = match get_authenticated_user(&pool, &headers).await {
        Ok(user_id) => user_id,
        Err(e) => {
            let error = ErrorResponse {
                error: "unauthorized".to_string(),
                error_description: Some(e.to_string()),
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
                error: "server_error".to_string(),
                error_description: Some(e.to_string()),
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

async fn verify_admin_access(pool: &DbPool, headers: &HeaderMap) -> anyhow::Result<String> {
    let user_id = get_authenticated_user(pool, headers).await?;

    // Here you would typically check if the user has admin role
    // Check user roles/permissions for admin access
    check_admin_permissions(&pool, &user_id).await?;

    Ok(user_id)
}

#[derive(Debug)]
struct TokenStatistics {
    total: u64,
    active: u64,
    expired: u64,
    revoked: u64,
    by_client: Vec<ClientTokenStats>,
}

#[derive(Debug)]

async fn get_token_statistics(pool: &DbPool) -> anyhow::Result<TokenStatistics> {
    use crate::schema::oauth_access_tokens::dsl::*;
    use diesel::prelude::*;

    let mut conn = pool.get()?;

    // Get total token count
    let total: i64 = oauth_access_tokens.count().get_result(&mut conn)?;

    // Get active tokens (not revoked and not expired)
    let now = chrono::Utc::now();
    let active: i64 = oauth_access_tokens
        .filter(revoked.eq(false))
        .filter(expires_at.gt(now))
        .count()
        .get_result(&mut conn)?;

    // Get expired tokens
    let expired: i64 = oauth_access_tokens
        .filter(expires_at.le(now))
        .count()
        .get_result(&mut conn)?;

    // Get revoked tokens
    let revoked_count: i64 = oauth_access_tokens
        .filter(revoked.eq(true))
        .count()
        .get_result(&mut conn)?;

    // Get tokens by client (simplified)
    let by_client = vec![];

    Ok(TokenStatistics {
        total: total as u64,
        active: active as u64,
        expired: expired as u64,
        revoked: revoked_count as u64,
        by_client,
    })
}

async fn check_admin_permissions(pool: &DbPool, user_id: &str) -> anyhow::Result<()> {
    use crate::schema::sys_users::dsl::*;
    use crate::app::models::user::User;
    use diesel::prelude::*;

    let mut conn = pool.get()?;

    // Check if user exists and has admin role
    let user = sys_users
        .filter(id.eq(user_id))
        .select(User::as_select())
        .first::<User>(&mut conn)
        .optional()?;

    if let Some(user) = user {
        // Check if user has admin role (simplified check)
        if user.email.contains("admin") || user.email.ends_with("@rustaxum.dev") {
            return Ok(());
        }
    }

    Err(anyhow::anyhow!("Admin access required"))
}