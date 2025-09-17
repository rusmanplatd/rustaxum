use axum::{
    extract::{Json, Path},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json as ResponseJson},
};
use serde::Serialize;
use ulid::Ulid;

use crate::app::models::user::{
    CreateUser, LoginRequest, ForgotPasswordRequest,
    ResetPasswordRequest, ChangePasswordRequest
};
use crate::app::services::auth_service::{AuthService, AuthResponse, MessageResponse};
use crate::app::utils::token_utils::TokenUtils;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn register(Json(payload): Json<CreateUser>) -> impl IntoResponse {
    match AuthService::register(payload).await {
        Ok(response) => (StatusCode::CREATED, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

pub async fn login(Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    match AuthService::login(payload).await {
        Ok(response) => (StatusCode::OK, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response()
        }
    }
}

pub async fn forgot_password(Json(payload): Json<ForgotPasswordRequest>) -> impl IntoResponse {
    match AuthService::forgot_password(payload).await {
        Ok(response) => (StatusCode::OK, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

pub async fn reset_password(Json(payload): Json<ResetPasswordRequest>) -> impl IntoResponse {
    match AuthService::reset_password(payload).await {
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
        Err(e) => {
            let error = ErrorResponse {
                error: "Invalid token".to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    let user_id = match Ulid::from_string(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid user ID in token".to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    // Check if token is blacklisted
    if let Ok(true) = AuthService::is_token_blacklisted(token).await {
        let error = ErrorResponse {
            error: "Token has been revoked".to_string(),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    match AuthService::change_password(user_id, payload).await {
        Ok(response) => (StatusCode::OK, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

pub async fn logout(headers: HeaderMap) -> impl IntoResponse {
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
        Err(e) => {
            let error = ErrorResponse {
                error: "Invalid token".to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    let user_id = match Ulid::from_string(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid user ID in token".to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    match AuthService::revoke_token(token, user_id, Some("Logout".to_string())).await {
        Ok(response) => (StatusCode::OK, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

pub async fn revoke_token(headers: HeaderMap) -> impl IntoResponse {
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
        Err(e) => {
            let error = ErrorResponse {
                error: "Invalid token".to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    let user_id = match Ulid::from_string(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid user ID in token".to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    match AuthService::revoke_token(token, user_id, Some("Manual revocation".to_string())).await {
        Ok(response) => (StatusCode::OK, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}