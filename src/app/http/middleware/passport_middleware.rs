use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode, request::Parts},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use crate::database::DbPool;

use crate::app::models::user::User;
use crate::app::services::auth_service::AuthService;
use crate::app::utils::token_utils::TokenUtils;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user: User,
    pub token: String,
}

pub async fn passport_middleware(
    State(pool): State<DbPool>,
    mut request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let headers = request.headers();

    match authenticate_request(headers, &pool).await {
        Ok(authenticated_user) => {
            // Add the authenticated user to request extensions
            request.extensions_mut().insert(authenticated_user);
            Ok(next.run(request).await)
        }
        Err(error_response) => Err(error_response),
    }
}

async fn authenticate_request(
    headers: &HeaderMap,
    pool: &DbPool,
) -> Result<AuthenticatedUser, (StatusCode, Json<serde_json::Value>)> {
    // Extract the Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Missing authorization header"})),
            )
        })?;

    // Extract token from "Bearer <token>" format
    let token = TokenUtils::extract_token_from_header(Some(auth_header))
        .map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": e.to_string()})),
            )
        })?;

    // Decode and validate the JWT token
    let claims = AuthService::decode_token(&token, "jwt-secret")
        .map_err(|_e| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid or expired token"})),
            )
        })?;

    // Fetch user from database using UserService
    let user = crate::app::services::user_service::UserService::find_by_id(pool, claims.sub)
        .map_err(|e| {
            tracing::error!("Database error fetching user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Authentication service error"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "User not found or account deactivated"})),
            )
        })?;

    Ok(AuthenticatedUser {
        user,
        token: token.to_string(),
    })
}


// Helper function that can be used in handlers to get authenticated user
pub fn require_auth(
    parts: &Parts,
) -> Result<&AuthenticatedUser, (StatusCode, Json<serde_json::Value>)> {
    parts
        .extensions
        .get::<AuthenticatedUser>()
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Authentication required"})),
            )
        })
}

// Helper function to get authenticated user from request parts
pub fn get_authenticated_user(parts: &Parts) -> Option<&AuthenticatedUser> {
    parts.extensions.get::<AuthenticatedUser>()
}