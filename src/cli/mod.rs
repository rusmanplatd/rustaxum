pub mod commands;
pub mod generators;

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "artisan")]
#[command(about = "Laravel-like CLI for Rust Axum application")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate application components
    #[command(subcommand)]
    Make(MakeCommands),
    /// Run database migrations
    Migrate {
        #[arg(long)]
        fresh: bool,
    },
    /// Start the development server
    Serve {
        #[arg(short, long, default_value = "3000")]
        port: u16,
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
}

#[derive(Subcommand)]
pub enum MakeCommands {
    /// Generate a new controller
    Controller {
        /// Name of the controller (e.g., UserController)
        name: String,
        #[arg(long)]
        resource: bool,
    },
    /// Generate a new model
    Model {
        /// Name of the model (e.g., User)
        name: String,
        #[arg(long)]
        migration: bool,
    },
    /// Generate a new service
    Service {
        /// Name of the service (e.g., UserService)
        name: String,
    },
    /// Generate a new middleware
    Middleware {
        /// Name of the middleware (e.g., AuthMiddleware)
        name: String,
    },
    /// Generate a new migration
    Migration {
        /// Name of the migration (e.g., create_users_table)
        name: String,
    },
}

pub async fn run_cli(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Make(make_cmd) => commands::make::handle_make_command(make_cmd).await,
        Commands::Migrate { fresh } => commands::migrate::handle_migrate_command(fresh).await,
        Commands::Serve { port, host } => commands::serve::handle_serve_command(host, port).await,
    }
}