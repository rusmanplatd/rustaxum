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

use app::http::middleware::session_middleware::session_middleware;
// use app::http::middleware::csrf_middleware::csrf_middleware;

pub async fn create_app() -> anyhow::Result<Router> {
    tracing::debug!("Starting application creation process");

    tracing::debug!("Loading configuration...");
    let config = config::Config::load()?;
    tracing::info!("Configuration loaded successfully");

    // Create database pool
    tracing::debug!("Creating database connection pool...");
    let pool = database::create_pool(&config)?;
    tracing::info!("Database pool created successfully");

    // Initialize global pool for legacy compatibility
    database::connection::initialize_pool(pool.clone());

    // Run migrations
    // database::run_migrations(&pool).await?; // Temporarily disabled - already applied
    tracing::debug!("Database migrations skipped (already applied)");


    // Initialize broadcasting system
    tracing::debug!("Initializing broadcasting system...");
    let broadcasting_config = config::Config::load()?.broadcasting;
    let broadcast_manager = app::broadcasting::init_broadcast_manager(broadcasting_config.default_driver).await;

    // Register broadcast drivers
    {
        let mut manager = broadcast_manager.write().await;

        // Register WebSocket driver
        if broadcasting_config.websocket_enabled {
            let websocket_driver = app::broadcasting::WebSocketDriver::new();
            manager.register_driver("websocket".to_string(), Box::new(websocket_driver));
            tracing::info!("WebSocket broadcast driver registered");
        }

        // Register Redis driver
        if broadcasting_config.redis_enabled {
            let redis_driver = app::broadcasting::RedisDriver::new(
                broadcasting_config.redis_host.clone(),
                broadcasting_config.redis_port,
            );
            manager.register_driver("redis".to_string(), Box::new(redis_driver));
            tracing::info!("Redis broadcast driver registered");
        }

        // Always register log driver for fallback
        let log_driver = app::broadcasting::LogDriver::new();
        manager.register_driver("log".to_string(), Box::new(log_driver));
        tracing::info!("Log broadcast driver registered");
    }

    // Get WebSocket manager for routes
    let websocket_manager = app::broadcasting::websocket::websocket_manager().await;

    tracing::debug!("Building router with routes...");
    let app = Router::new()
        .merge(routes::api::routes())
        .merge(routes::web::routes())
        .merge(routes::oauth::oauth_routes())
        // Add WebSocket routes
        .nest("/ws", app::broadcasting::websocket::websocket_routes().with_state(websocket_manager))
        .with_state(pool.clone())
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn_with_state(pool, session_middleware))
                // .layer(middleware::from_fn(csrf_middleware))
                .layer(middleware::from_fn(app::http::middleware::correlation_middleware::correlation_middleware))
                .layer(middleware::from_fn(app::http::middleware::activity_logging_middleware::activity_logging_middleware))
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        );

    tracing::info!("Application router created with all routes and middleware");
    tracing::debug!("Application creation completed successfully");

    Ok(app)
}