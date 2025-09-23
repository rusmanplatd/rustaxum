use axum::{
    extract::{Json, State, Path, Query},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json as ResponseJson},
};
use serde::{Deserialize, Serialize};
use crate::database::DbPool;
use chrono::{Utc};

use crate::app::services::oauth::TokenService;
use crate::app::services::auth_service::AuthService;
use crate::app::utils::token_utils::TokenUtils;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    error_description: Option<String>,
}

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

#[derive(Deserialize)]
pub struct ListTokensQuery {
    pub user_id: Option<String>,
    pub client_id: Option<String>,
    pub scope: Option<String>,
    pub active_only: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct RevokeTokensRequest {
    pub token_ids: Vec<String>,
}

#[derive(Deserialize)]
pub struct ExtendTokenRequest {
    pub expires_in_seconds: i64,
}

/// List access tokens with filtering and pagination
pub async fn list_tokens(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Query(params): Query<ListTokensQuery>,
) -> impl IntoResponse {
    // Verify admin access for listing all tokens
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: "unauthorized".to_string(),
            error_description: Some(e.to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    // For now, if user_id is specified, use the existing method
    if let Some(user_id) = params.user_id {
        match TokenService::list_user_tokens(&pool, user_id).await {
            Ok(tokens) => {
                let mut filtered_tokens = tokens;

                // Apply filters
                if let Some(client_id) = params.client_id {
                    filtered_tokens.retain(|t| t.client_id == client_id);
                }

                if let Some(scope) = params.scope {
                    filtered_tokens.retain(|t| t.has_scope(&scope));
                }

                if params.active_only.unwrap_or(false) {
                    filtered_tokens.retain(|t| t.is_valid());
                }

                // Apply pagination
                let offset = params.offset.unwrap_or(0) as usize;
                let limit = params.limit.unwrap_or(50) as usize;

                if offset < filtered_tokens.len() {
                    let end = std::cmp::min(offset + limit, filtered_tokens.len());
                    filtered_tokens = filtered_tokens[offset..end].to_vec();
                } else {
                    filtered_tokens.clear();
                }

                let responses: Vec<_> = filtered_tokens.into_iter().map(|t| t.to_response()).collect();
                (StatusCode::OK, ResponseJson(responses)).into_response()
            },
            Err(e) => {
                let error = ErrorResponse {
                    error: "server_error".to_string(),
                    error_description: Some(e.to_string()),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
            }
        }
    } else {
        // Would need to implement a general list_all_tokens method
        let error = ErrorResponse {
            error: "not_implemented".to_string(),
            error_description: Some("Listing all tokens requires user_id parameter".to_string()),
        };
        (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
    }
}

/// Get detailed information about a specific token
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
                    // For now, allow any authenticated user to view any token
                    // In production, you'd check admin permissions here
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

    // This would need to be implemented with proper database queries
    // For now, return a placeholder response
    let _response = TokenStatsResponse {
        total_tokens: 0,
        active_tokens: 0,
        expired_tokens: 0,
        revoked_tokens: 0,
        tokens_by_client: vec![],
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
pub async fn get_my_tokens(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Query(params): Query<ListTokensQuery>,
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

    match TokenService::list_user_tokens(&pool, user_id).await {
        Ok(tokens) => {
            let mut filtered_tokens = tokens;

            // Apply filters
            if let Some(client_id) = params.client_id {
                filtered_tokens.retain(|t| t.client_id == client_id);
            }

            if let Some(scope) = params.scope {
                filtered_tokens.retain(|t| t.has_scope(&scope));
            }

            if params.active_only.unwrap_or(false) {
                filtered_tokens.retain(|t| t.is_valid());
            }

            // Apply pagination
            let offset = params.offset.unwrap_or(0) as usize;
            let limit = params.limit.unwrap_or(50) as usize;

            if offset < filtered_tokens.len() {
                let end = std::cmp::min(offset + limit, filtered_tokens.len());
                filtered_tokens = filtered_tokens[offset..end].to_vec();
            } else {
                filtered_tokens.clear();
            }

            let responses: Vec<_> = filtered_tokens.into_iter().map(|t| t.to_response()).collect();
            (StatusCode::OK, ResponseJson(responses)).into_response()
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
    let claims = AuthService::decode_token(token, "jwt-secret")?;

    Ok(claims.sub)
}

async fn verify_admin_access(pool: &DbPool, headers: &HeaderMap) -> anyhow::Result<String> {
    let user_id = get_authenticated_user(pool, headers).await?;

    // Here you would typically check if the user has admin role
    // For now, we'll accept any authenticated user
    // In a real implementation, you'd check user roles/permissions

    Ok(user_id)
}