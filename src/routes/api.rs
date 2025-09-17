use axum::{
    routing::{get, post},
    Router,
};

use crate::app::controllers::{auth_controller, user_controller};

pub fn routes() -> Router {
    Router::new()
        .route("/api/auth/login", post(auth_controller::login))
        .route("/api/auth/register", post(auth_controller::register))
        .route("/api/users", get(user_controller::index))
        .route("/api/users/:id", get(user_controller::show))
}