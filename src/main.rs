use rustaxum::{create_app, config, logging::Log};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::Config::load()?;

    // Initialize Laravel-style logging
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

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_level.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = create_app().await?;

    let listener = tokio::net::TcpListener::bind(&config.server_addr()).await?;
    tracing::info!("Server running on {}", config.server_addr());

    axum::serve(listener, app).await?;

    Ok(())
}