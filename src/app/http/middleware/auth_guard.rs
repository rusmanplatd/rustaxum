use axum::{
    extract::Request,
    http::{StatusCode},
    middleware::Next,
    response::Response,
    response::Json,
};
use serde_json::json;
use crate::app::http::middleware::auth_middleware::{validate_jwt_token, extract_bearer_token};
use crate::app::services::session::SessionStore;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub auth_method: String, // "jwt" or "session"
}

/// Unified auth guard middleware that supports both JWT and Session authentication
/// Similar to Laravel's 'auth' middleware
pub async fn auth_guard(
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let headers = request.headers();

    // Try JWT authentication first
    if let Ok(token) = extract_bearer_token(&headers) {
        if let Some(user_id) = validate_jwt_token(&token) {
            let auth_user = AuthUser {
                user_id,
                auth_method: "jwt".to_string(),
            };
            request.extensions_mut().insert(auth_user);
            return Ok(next.run(request).await);
        }
    }

    // Try session authentication
    if let Some(session) = request.extensions().get::<SessionStore>() {
        if let Some(user_id) = session.get_string("user_id").await {
            if session.get_bool("authenticated").await.unwrap_or(false) {
                let auth_user = AuthUser {
                    user_id,
                    auth_method: "session".to_string(),
                };
                request.extensions_mut().insert(auth_user);
                return Ok(next.run(request).await);
            }
        }
    }

    // No valid authentication found
    let error_response = json!({
        "error": "Unauthorized",
        "message": "Authentication required. Please provide a valid Bearer token or valid session."
    });
    Err((StatusCode::UNAUTHORIZED, Json(error_response)))
}

/// Guest middleware that allows only unauthenticated users
/// Similar to Laravel's 'guest' middleware
pub async fn guest_guard(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let headers = request.headers();

    // Check JWT authentication
    if let Ok(token) = extract_bearer_token(&headers) {
        if validate_jwt_token(&token).is_some() {
            let error_response = json!({
                "error": "Forbidden",
                "message": "Already authenticated via JWT. This endpoint is for guests only."
            });
            return Err((StatusCode::FORBIDDEN, Json(error_response)));
        }
    }

    // Check session authentication
    if let Some(session) = request.extensions().get::<SessionStore>() {
        if let Some(_user_id) = session.get_string("user_id").await {
            if session.get_bool("authenticated").await.unwrap_or(false) {
                let error_response = json!({
                    "error": "Forbidden",
                    "message": "Already authenticated via session. This endpoint is for guests only."
                });
                return Err((StatusCode::FORBIDDEN, Json(error_response)));
            }
        }
    }

    Ok(next.run(request).await)
}

/// Optional auth middleware that adds user info if authenticated but doesn't require it
/// Similar to Laravel's optional auth
pub async fn optional_auth(
    mut request: Request,
    next: Next,
) -> Response {
    let headers = request.headers();

    // Try JWT authentication first
    if let Ok(token) = extract_bearer_token(&headers) {
        if let Some(user_id) = validate_jwt_token(&token) {
            let auth_user = AuthUser {
                user_id,
                auth_method: "jwt".to_string(),
            };
            request.extensions_mut().insert(auth_user);
            return next.run(request).await;
        }
    }

    // Try session authentication
    if let Some(session) = request.extensions().get::<SessionStore>() {
        if let Some(user_id) = session.get_string("user_id").await {
            if session.get_bool("authenticated").await.unwrap_or(false) {
                let auth_user = AuthUser {
                    user_id,
                    auth_method: "session".to_string(),
                };
                request.extensions_mut().insert(auth_user);
            }
        }
    }

    next.run(request).await
}

/// Helper function to get authenticated user from request extensions
pub fn get_auth_user(request: &Request) -> Option<&AuthUser> {
    request.extensions().get::<AuthUser>()
}