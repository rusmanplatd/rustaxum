use clap::Parser;
use rustaxum::cli::{run_cli, Cli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let cli = Cli::parse();
    run_cli(cli).await
}