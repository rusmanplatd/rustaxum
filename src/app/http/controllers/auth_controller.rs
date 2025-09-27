use axum::{
    extract::{Json, State, Extension},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json as ResponseJson},
};
use serde::Serialize;
use serde_json::{json, Value};
use crate::database::DbPool;
use crate::app::services::session::SessionStore;

use crate::app::models::user::{
    CreateUser, LoginRequest, ForgotPasswordRequest,
    ResetPasswordRequest, ChangePasswordRequest, RefreshTokenRequest
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct MfaLoginRequest {
    pub user_id: String,
    pub mfa_code: String,
}
use crate::app::services::auth_service::{AuthService, LoginResponse};
use crate::app::utils::token_utils::TokenUtils;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn register(State(pool): State<DbPool>, Json(payload): Json<CreateUser>) -> impl IntoResponse {
    match AuthService::register(&pool, payload).await {
        Ok(response) => (StatusCode::CREATED, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

pub async fn login(State(pool): State<DbPool>, Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    match AuthService::login(&pool, payload).await {
        Ok(LoginResponse::Success(auth_response)) => {
            (StatusCode::OK, ResponseJson(auth_response)).into_response()
        },
        Ok(LoginResponse::MfaRequired(mfa_response)) => {
            (StatusCode::OK, ResponseJson(mfa_response)).into_response()
        },
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response()
        }
    }
}

pub async fn forgot_password(State(pool): State<DbPool>, Json(payload): Json<ForgotPasswordRequest>) -> impl IntoResponse {
    match AuthService::forgot_password(&pool, payload).await {
        Ok(response) => (StatusCode::OK, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

pub async fn reset_password(State(pool): State<DbPool>, Json(payload): Json<ResetPasswordRequest>) -> impl IntoResponse {
    match AuthService::reset_password(&pool, payload) {
        Ok(response) => (StatusCode::OK, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

pub async fn change_password(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    // Extract user ID from JWT token
    let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());

    let token = match TokenUtils::extract_token_from_header(auth_header) {
        Ok(token) => token,
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    // Decode token to get user ID
    let claims = match AuthService::decode_token(token, "jwt-secret") {
        Ok(claims) => claims,
        Err(_e) => {
            let error = ErrorResponse {
                error: "Invalid token".to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    match AuthService::change_password(&pool, claims.sub, payload) {
        Ok(response) => (StatusCode::OK, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

pub async fn refresh_token(State(pool): State<DbPool>, Json(payload): Json<RefreshTokenRequest>) -> impl IntoResponse {
    match AuthService::refresh_token(&pool, payload) {
        Ok(response) => (StatusCode::OK, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response()
        }
    }
}

// Session-based authentication endpoints

pub async fn login_session(
    State(pool): State<DbPool>,
    Extension(session): Extension<SessionStore>,
    Json(payload): Json<LoginRequest>
) -> impl IntoResponse {
    match AuthService::login(&pool, payload).await {
        Ok(LoginResponse::Success(response)) => {
            // Store user ID in session instead of returning JWT
            session.put("user_id", Value::String(response.user.id.to_string())).await;
            session.put("authenticated", Value::Bool(true)).await;
            session.regenerate().await.ok();

            let session_response = json!({
                "message": "Login successful",
                "user": response.user,
                "session_id": session.get_session_id().await
            });
            (StatusCode::OK, ResponseJson(session_response)).into_response()
        }
        Ok(LoginResponse::MfaRequired(mfa_response)) => {
            // Store MFA state in session
            session.put("mfa_user_id", Value::String(mfa_response.user_id.clone())).await;
            session.put("mfa_required", Value::Bool(true)).await;
            session.regenerate().await.ok();

            let mfa_session_response = json!({
                "requires_mfa": true,
                "message": mfa_response.message,
                "user_id": mfa_response.user_id,
                "mfa_methods": mfa_response.mfa_methods,
                "session_id": session.get_session_id().await
            });
            (StatusCode::OK, ResponseJson(mfa_session_response)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response()
        }
    }
}

pub async fn register_session(
    State(pool): State<DbPool>,
    Extension(session): Extension<SessionStore>,
    Json(payload): Json<CreateUser>
) -> impl IntoResponse {
    match AuthService::register(&pool, payload).await {
        Ok(response) => {
            // Store user ID in session instead of returning JWT
            session.put("user_id", Value::String(response.user.id.to_string())).await;
            session.put("authenticated", Value::Bool(true)).await;
            session.regenerate().await.ok();

            let session_response = json!({
                "message": "Registration successful",
                "user": response.user,
                "session_id": session.get_session_id().await
            });
            (StatusCode::CREATED, ResponseJson(session_response)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

pub async fn logout_session(
    Extension(session): Extension<SessionStore>
) -> impl IntoResponse {
    // Clear all session data
    session.flush().await;
    session.regenerate().await.ok();

    let response = json!({
        "message": "Logout successful"
    });
    (StatusCode::OK, ResponseJson(response)).into_response()
}

pub async fn user_session(
    State(pool): State<DbPool>,
    Extension(session): Extension<SessionStore>
) -> impl IntoResponse {
    if let Some(user_id) = session.get_string("user_id").await {
        match crate::app::services::user_service::UserService::find_by_id(&pool, user_id) {
            Ok(Some(user)) => {
                let response = json!({
                    "user": user.to_response()
                });
                (StatusCode::OK, ResponseJson(response)).into_response()
            }
            Ok(None) => {
                // User not found, clear session
                session.flush().await;
                let error = ErrorResponse {
                    error: "User not found".to_string(),
                };
                (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response()
            }
            Err(e) => {
                let error = ErrorResponse {
                    error: e.to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
            }
        }
    } else {
        let error = ErrorResponse {
            error: "Not authenticated".to_string(),
        };
        (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response()
    }
}

pub async fn complete_mfa_login(
    State(pool): State<DbPool>,
    Json(payload): Json<MfaLoginRequest>
) -> impl IntoResponse {
    match AuthService::complete_mfa_login(&pool, payload.user_id, &payload.mfa_code).await {
        Ok(response) => (StatusCode::OK, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response()
        }
    }
}
