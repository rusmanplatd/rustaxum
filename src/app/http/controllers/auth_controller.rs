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
    ResetPasswordRequest, ChangePasswordRequest, RefreshTokenRequest, UserResponse
};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct MfaLoginRequest {
    pub user_id: String,
    pub mfa_code: String,
}
use crate::app::services::auth_service::{AuthService, LoginResponse};
use crate::app::utils::token_utils::TokenUtils;

#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "Authentication",
    summary = "Register a new user",
    description = "Create a new user account with email and password",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User registered successfully", body = UserResponse),
        (status = 400, description = "Registration failed", body = ErrorResponse)
    )
)]
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

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Authentication",
    summary = "Login user",
    description = "Authenticate user with email and password. May require MFA if enabled.",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful or MFA required"),
        (status = 401, description = "Authentication failed", body = ErrorResponse)
    )
)]
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

#[utoipa::path(
    post,
    path = "/api/auth/forgot-password",
    tag = "Authentication",
    summary = "Request password reset",
    description = "Send password reset email to user",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Password reset email sent"),
        (status = 400, description = "Request failed", body = ErrorResponse)
    )
)]
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
    let claims = match AuthService::decode_token(token) {
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

#[utoipa::path(
    get,
    path = "/api/me",
    tag = "Authentication",
    summary = "Get current user with organizations",
    description = "Get authenticated user profile information including all organization relationships and their complete details (organization, position, level)",
    responses(
        (status = 200, description = "User profile with organizations retrieved", body = serde_json::Value),
        (status = 401, description = "Not authenticated", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn me(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<crate::app::http::middleware::auth_guard::AuthUser>
) -> impl IntoResponse {
    match crate::app::services::user_service::UserService::find_by_id_with_organizations(&pool, auth_user.user_id) {
        Ok(Some((user, user_organizations))) => {
            let response = json!({
                "user": user.to_response(),
                "organizations": user_organizations
            });
            (StatusCode::OK, ResponseJson(response)).into_response()
        }
        Ok(None) => {
            let error = ErrorResponse {
                error: "User not found".to_string(),
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
