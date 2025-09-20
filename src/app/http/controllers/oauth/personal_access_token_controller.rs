use axum::{
    extract::{Json, State, Path},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json as ResponseJson},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use ulid::Ulid;

use crate::app::services::oauth::TokenService;
use crate::app::services::auth_service::AuthService;
use crate::app::utils::token_utils::TokenUtils;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
pub struct CreatePersonalAccessTokenRequest {
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_in_seconds: Option<i64>,
}

pub async fn create_personal_access_token(
    State(pool): State<PgPool>,
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

pub async fn list_personal_access_tokens(
    State(pool): State<PgPool>,
    headers: HeaderMap,
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

    match TokenService::list_user_tokens(&pool, user_id).await {
        Ok(tokens) => {
            let responses: Vec<_> = tokens.into_iter().map(|t| t.to_response()).collect();
            (StatusCode::OK, ResponseJson(responses)).into_response()
        },
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

pub async fn revoke_personal_access_token(
    State(pool): State<PgPool>,
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

    let token_ulid = match Ulid::from_string(&token_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid token ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    // Verify the token belongs to the user
    match TokenService::find_access_token_by_id(&pool, token_ulid).await {
        Ok(Some(token)) => {
            if let Some(owner_id) = token.user_id {
                if owner_id != user_id {
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

    match TokenService::revoke_access_token(&pool, token_ulid).await {
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

async fn get_authenticated_user(_pool: &PgPool, headers: &HeaderMap) -> anyhow::Result<Ulid> {
    let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());
    let token = TokenUtils::extract_token_from_header(auth_header)?;
    let claims = AuthService::decode_token(token, "jwt-secret")?;

    let user_id = Ulid::from_string(&claims.sub)?;
    Ok(user_id)
}