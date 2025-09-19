use axum::{routing::get, Router};
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};

use crate::app::docs::ApiDoc;

pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/", get(home))
        .route("/health", get(health_check))
        // Documentation UIs
        .merge(
            SwaggerUi::new("/docs/swagger")
                .url("/api/docs/openapi.json", ApiDoc::openapi())
        )
        .merge(
            RapiDoc::new("/api/docs/openapi.json")
                .path("/docs/rapidoc")
        )
        .merge(
            Redoc::with_url("/docs/redoc", ApiDoc::openapi())
        )
}

async fn home() -> &'static str {
    "Welcome to RustAxum - A Laravel-inspired Rust web framework"
}

async fn health_check() -> &'static str {
    "OK"
}