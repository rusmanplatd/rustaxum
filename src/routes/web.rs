use axum::{routing::get, Router};
use sqlx::PgPool;

pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/", get(home))
        .route("/health", get(health_check))
}

async fn home() -> &'static str {
    "Welcome to RustAxum - A Laravel-inspired Rust web framework"
}

async fn health_check() -> &'static str {
    "OK"
}