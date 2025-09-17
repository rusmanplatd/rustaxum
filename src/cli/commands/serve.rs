use anyhow::Result;
use crate::create_app;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn handle_serve_command(host: String, port: u16) -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustaxum=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let server_addr = format!("{}:{}", host, port);
    println!("🚀 Starting development server...");
    println!("📡 Server will run on: http://{}", server_addr);

    let app = create_app().await?;
    let listener = tokio::net::TcpListener::bind(&server_addr).await?;

    println!("✅ Server running on http://{}", server_addr);
    println!("Press Ctrl+C to stop the server");

    axum::serve(listener, app).await?;

    Ok(())
}