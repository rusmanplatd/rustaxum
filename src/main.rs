use rustaxum::{create_app, config, logging::Log};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::info!("🚀 Starting RustAxum application...");

    tracing::debug!("Loading application configuration...");
    let config = config::Config::load()?;
    tracing::info!("Configuration loaded - Debug mode: {}", config.app.debug);

    // Initialize Laravel-style logging
    tracing::debug!("Initializing Laravel-style logging...");
    Log::init(config.logging.clone())?;

    let log_level = if config.app.debug {
        format!("rustaxum=debug,tower_http=debug,info")
    } else {
        if let Some(channel) = config.logging.get_default_channel() {
            channel.level.clone()
        } else {
            "info".to_string()
        }
    };

    tracing::debug!("Setting up tracing subscriber with level: {}", log_level);
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_level.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting application creation...");
    let app = create_app().await?;

    let server_addr = config.server_addr();
    tracing::debug!("Binding to server address: {}", server_addr);
    let listener = tokio::net::TcpListener::bind(&server_addr).await?;
    tracing::info!("Server running on {}", server_addr);

    tracing::info!("🎯 Application startup completed successfully");
    tracing::debug!("Starting Axum HTTP server...");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    ).await?;

    tracing::info!("Application shutdown completed");
    Ok(())
}