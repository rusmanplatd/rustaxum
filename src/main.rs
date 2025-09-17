use rustaxum::{create_app, config};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustaxum=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from_env()?;

    let app = create_app().await?;

    let listener = tokio::net::TcpListener::bind(&config.server_addr()).await?;
    tracing::info!("Server running on {}", config.server_addr());

    axum::serve(listener, app).await?;

    Ok(())
}