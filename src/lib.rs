pub mod app;
pub mod config;
pub mod routes;

use axum::Router;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub async fn create_app() -> anyhow::Result<Router> {
    let app = Router::new()
        .merge(routes::api::routes())
        .merge(routes::web::routes())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        );

    Ok(app)
}