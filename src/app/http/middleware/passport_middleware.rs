use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode, request::Parts},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use crate::database::DbPool;
use ulid::Ulid;

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

    // Parse user ID from token claims
    let user_id = Ulid::from_string(&claims.sub)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid user ID in token"})),
            )
        })?;

    // Fetch user from database using query_as to avoid SQLx macro issues
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, name, email_verified_at, password_hash, created_at, updated_at FROM sys_users WHERE id = $1 AND deleted_at IS NULL"
    )
    .bind(user_id.to_string())
    .fetch_optional(pool)
    .await
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

// TODO: Implement FromRequestParts trait correctly
// For now, using a different approach with middleware and extension retrieval

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