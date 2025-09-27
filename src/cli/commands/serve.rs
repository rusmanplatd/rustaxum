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
    tracing::info!("Serve command initiated - Host: {}, Port: {}", host, port);

    tracing::debug!("Creating application instance...");
    let app = create_app().await?;
    tracing::info!("Application created successfully");

    tracing::debug!("Binding to address: {}", server_addr);
    let listener = tokio::net::TcpListener::bind(&server_addr).await?;
    tracing::info!("TCP listener bound successfully to {}", server_addr);

    println!("✅ Server running on http://{}", server_addr);
    println!("Press Ctrl+C to stop the server");
    tracing::info!("Development server started successfully on http://{}", server_addr);

    tracing::debug!("Starting Axum server...");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    ).await?;
    tracing::info!("Server shutdown completed");

    Ok(())
}