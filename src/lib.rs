pub mod app;
pub mod config;
pub mod routes;
pub mod database;
pub mod cli;
pub mod storage;
pub mod cache;
pub mod logging;
pub mod schema;

// Re-export validation for convenient access
pub use app::validation;

use axum::{Router, middleware};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub async fn create_app() -> anyhow::Result<Router> {
    tracing::debug!("Starting application creation process");

    tracing::debug!("Loading configuration...");
    let config = config::Config::load()?;
    tracing::info!("Configuration loaded successfully");

    // Create database pool
    tracing::debug!("Creating database connection pool...");
    let pool = database::create_pool(&config)?;
    tracing::info!("Database pool created successfully");

    // Run migrations
    // database::run_migrations(&pool).await?; // Temporarily disabled - already applied
    tracing::debug!("Database migrations skipped (already applied)");

    tracing::debug!("Building router with routes...");
    let app = Router::new()
        .merge(routes::api::routes())
        .merge(routes::web::routes())
        .merge(routes::oauth::oauth_routes())
        .with_state(pool)
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(app::http::middleware::logging::request_logging_middleware))
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        );

    tracing::info!("Application router created with all routes and middleware");
    tracing::debug!("Application creation completed successfully");

    Ok(app)
}