use anyhow::Result;
use rustaxum::app::demo::communication_demo::{run_communication_demo, demo_laravel_like_usage};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🔧 Testing Communication & Events System");
    println!("========================================\n");

    // Run the complete demo
    run_communication_demo().await?;

    println!("\n");

    // Run Laravel-like usage demo
    demo_laravel_like_usage().await?;

    println!("\n🎉 All tests completed successfully!");

    Ok(())
}