use axum::{routing::get, Router};

pub fn routes() -> Router {
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