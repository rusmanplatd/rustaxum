use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::PgPool;

use crate::app::controllers::{auth_controller, user_controller};

pub fn routes() -> Router<PgPool> {
    Router::new()
        // Authentication routes
        .route("/api/auth/register", post(auth_controller::register))
        .route("/api/auth/login", post(auth_controller::login))
        .route("/api/auth/forgot-password", post(auth_controller::forgot_password))
        .route("/api/auth/reset-password", post(auth_controller::reset_password))
        .route("/api/auth/change-password", put(auth_controller::change_password))
        .route("/api/auth/refresh-token", post(auth_controller::refresh_token))
        .route("/api/auth/logout", post(auth_controller::logout))
        .route("/api/auth/revoke-token", delete(auth_controller::revoke_token))
        .route("/api/auth/revoke-all-tokens", delete(auth_controller::revoke_all_tokens))
        // User routes
        .route("/api/users", get(user_controller::index))
        .route("/api/users/:id", get(user_controller::show))
}