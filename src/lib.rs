pub mod app;
pub mod config;
pub mod routes;
pub mod database;

use axum::Router;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use sqlx::PgPool;

pub async fn create_app() -> anyhow::Result<Router> {
    let config = config::Config::from_env()?;

    // Create database pool
    let pool = database::create_pool(&config).await?;

    // Run migrations
    database::run_migrations(&pool).await?;

    let app = Router::new()
        .merge(routes::api::routes())
        .merge(routes::web::routes())
        .with_state(pool)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        );

    Ok(app)
}