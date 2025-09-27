use axum::{
    extract::Request,
    http::{StatusCode},
    middleware::Next,
    response::{Response, Json, Redirect, IntoResponse},
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
    // For web requests (HTML), redirect to login instead of JSON 401
    let path = request.uri().path();
    let wants_html = request
        .headers()
        .get("accept")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.contains("text/html"))
        .unwrap_or(false);

    if wants_html && !path.starts_with("/api") {
        return Ok(Redirect::to("/auth/login").into_response());
    }

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

/// MFA guard that allows access if user is fully authenticated OR in MFA-required session state
/// This is used for MFA setup/verification routes only
pub async fn mfa_guard(
    mut request: Request,
    next: Next,
)-> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let headers = request.headers();

    // Allow if valid JWT
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

    // Allow if authenticated session
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

        // Allow if in MFA-required state
        if session.get_bool("mfa_required").await.unwrap_or(false) {
            if let Some(mfa_user_id) = session.get_string("mfa_user_id").await {
                let auth_user = AuthUser {
                    user_id: mfa_user_id,
                    auth_method: "session-mfa".to_string(),
                };
                request.extensions_mut().insert(auth_user);
                return Ok(next.run(request).await);
            }
        }
    }

    // Otherwise, unauthorized
    let error_response = json!({
        "error": "Unauthorized",
        "message": "Authentication or pending MFA required to access this resource."
    });
    Err((StatusCode::UNAUTHORIZED, Json(error_response)))
}

/// Helper function to get authenticated user from request extensions
pub fn get_auth_user(request: &Request) -> Option<&AuthUser> {
    request.extensions().get::<AuthUser>()
}