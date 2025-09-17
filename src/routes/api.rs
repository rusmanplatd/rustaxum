use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::app::controllers::{auth_controller, user_controller};

pub fn routes() -> Router {
    Router::new()
        // Authentication routes
        .route("/api/auth/register", post(auth_controller::register))
        .route("/api/auth/login", post(auth_controller::login))
        .route("/api/auth/forgot-password", post(auth_controller::forgot_password))
        .route("/api/auth/reset-password", post(auth_controller::reset_password))
        .route("/api/auth/change-password", put(auth_controller::change_password))
        .route("/api/auth/logout", post(auth_controller::logout))
        .route("/api/auth/revoke-token", delete(auth_controller::revoke_token))
        // User routes
        .route("/api/users", get(user_controller::index))
        .route("/api/users/:id", get(user_controller::show))
}