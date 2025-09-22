use axum::{
    extract::{Json, State},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json as ResponseJson},
};
use serde::Serialize;
use crate::database::DbPool;

use crate::app::models::user::{
    CreateUser, LoginRequest, ForgotPasswordRequest,
    ResetPasswordRequest, ChangePasswordRequest, RefreshTokenRequest
};
use crate::app::services::auth_service::AuthService;
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
    match AuthService::login(&pool, payload) {
        Ok(response) => (StatusCode::OK, ResponseJson(response)).into_response(),
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

pub async fn logout(State(pool): State<DbPool>, headers: HeaderMap) -> impl IntoResponse {
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

    match AuthService::revoke_token(&pool, token, claims.sub, Some("Logout".to_string())) {
        Ok(response) => (StatusCode::OK, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

pub async fn revoke_token(State(pool): State<DbPool>, headers: HeaderMap) -> impl IntoResponse {
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

    match AuthService::revoke_token(&pool, token, claims.sub, Some("Manual revocation".to_string())) {
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

pub async fn revoke_all_tokens(State(pool): State<DbPool>, headers: HeaderMap) -> impl IntoResponse {
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
    match AuthService::revoke_all_tokens(&pool, claims.sub) {
        Ok(response) => (StatusCode::OK, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}