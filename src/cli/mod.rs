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
        /// Run seeders after migrations
        #[arg(long)]
        seed: bool,
    },
    /// Rollback database migrations
    #[command(name = "migrate:rollback")]
    MigrateRollback {
        /// Number of migration batches to rollback
        #[arg(long, default_value = "1")]
        step: i32,
    },
    /// Reset all migrations (rollback all)
    #[command(name = "migrate:reset")]
    MigrateReset,
    /// Reset and re-run all migrations
    #[command(name = "migrate:refresh")]
    MigrateRefresh {
        /// Run seeders after migrations
        #[arg(long)]
        seed: bool,
    },
    /// Show migration status
    #[command(name = "migrate:status")]
    MigrateStatus,
    /// Run database seeders
    #[command(name = "db:seed")]
    DbSeed {
        /// Specific seeder class to run
        #[arg(long)]
        class: Option<String>,
        /// Drop all tables and re-run migrations before seeding
        #[arg(long)]
        fresh: bool,
    },
    /// List all available seeders
    #[command(name = "db:seed:list")]
    DbSeedList,
    /// Start the development server
    Serve {
        #[arg(short, long, default_value = "3000")]
        port: u16,
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    /// OAuth2/Passport commands
    #[command(subcommand)]
    Passport(PassportCommands),
    /// List application routes
    #[command(subcommand)]
    Route(RouteCommands),
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
    /// Generate a new form request
    Request {
        /// Name of the request (e.g., CreateUserRequest)
        name: String,
    },
    /// Generate a new seeder
    Seeder {
        /// Name of the seeder (e.g., UserSeeder)
        name: String,
    },
    /// Generate a new API resource
    Resource {
        /// Name of the resource (e.g., UserResource)
        name: String,
        /// Generate a resource collection
        #[arg(long)]
        collection: bool,
    },
    /// Generate a new mailable
    Mail {
        /// Name of the mail (e.g., OrderShipped)
        name: String,
        /// Generate a markdown template
        #[arg(long)]
        markdown: bool,
    },
    /// Generate a new notification
    Notification {
        /// Name of the notification (e.g., InvoicePaid)
        name: String,
        /// Generate markdown template
        #[arg(long)]
        markdown: bool,
    },
    /// Generate a new job
    Job {
        /// Name of the job (e.g., ProcessPayment)
        name: String,
        /// Mark job as sync
        #[arg(long)]
        sync: bool,
    },
    /// Generate a new event
    Event {
        /// Name of the event (e.g., OrderShipped)
        name: String,
    },
    /// Generate a new event listener
    Listener {
        /// Name of the listener (e.g., SendShipmentNotification)
        name: String,
        /// Event to listen for
        #[arg(long)]
        event: Option<String>,
        /// Mark listener as queued
        #[arg(long)]
        queued: bool,
    },
    /// Generate a new policy
    Policy {
        /// Name of the policy (e.g., PostPolicy)
        name: String,
        /// Model the policy applies to
        #[arg(long)]
        model: Option<String>,
    },
    /// Generate a new validation rule
    Rule {
        /// Name of the rule (e.g., Uppercase)
        name: String,
    },
    /// Generate a new test
    Test {
        /// Name of the test (e.g., UserTest)
        name: String,
        /// Generate a unit test instead of feature test
        #[arg(long)]
        unit: bool,
    },
}

#[derive(Subcommand)]
pub enum PassportCommands {
    /// Install OAuth2/Passport
    Install,
    /// Create an OAuth2 client
    #[command(name = "client")]
    CreateClient {
        /// Name of the client
        #[arg(long)]
        name: String,
        /// Redirect URIs (comma separated)
        #[arg(long)]
        redirect_uris: String,
        /// Create a personal access client
        #[arg(long)]
        personal: bool,
        /// Create a password grant client
        #[arg(long)]
        password: bool,
    },
    /// List OAuth2 clients
    #[command(name = "client:list")]
    ListClients,
    /// Revoke an OAuth2 client
    #[command(name = "client:revoke")]
    RevokeClient {
        /// Client ID to revoke
        client_id: String,
    },
    /// Delete an OAuth2 client
    #[command(name = "client:delete")]
    DeleteClient {
        /// Client ID to delete
        client_id: String,
    },
    /// Regenerate client secret
    #[command(name = "client:secret")]
    RegenerateSecret {
        /// Client ID
        client_id: String,
    },
    /// Create a scope
    #[command(name = "scope:create")]
    CreateScope {
        /// Name of the scope
        name: String,
        /// Description of the scope
        #[arg(long)]
        description: Option<String>,
        /// Make this a default scope
        #[arg(long)]
        default: bool,
    },
    /// List scopes
    #[command(name = "scope:list")]
    ListScopes,
    /// Delete a scope
    #[command(name = "scope:delete")]
    DeleteScope {
        /// Scope name or ID
        scope: String,
    },
    /// List access tokens
    #[command(name = "token:list")]
    ListTokens {
        /// User ID to filter by
        #[arg(long)]
        user_id: Option<String>,
    },
    /// Revoke an access token
    #[command(name = "token:revoke")]
    RevokeToken {
        /// Token ID
        token_id: String,
    },
    /// Revoke all tokens for a user
    #[command(name = "token:revoke-all")]
    RevokeAllUserTokens {
        /// User ID
        user_id: String,
    },
}

#[derive(Subcommand)]
pub enum RouteCommands {
    /// List all application routes
    List {
        /// Filter routes by name
        #[arg(long)]
        name: Option<String>,
        /// Filter routes by HTTP method (GET, POST, etc.)
        #[arg(long)]
        method: Option<String>,
        /// Filter routes by URI pattern
        #[arg(long)]
        uri: Option<String>,
    },
}

pub async fn run_cli(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Make(make_cmd) => commands::make::handle_make_command(make_cmd).await,
        Commands::Migrate { fresh, seed } => commands::migrate::handle_migrate_command(fresh, seed),
        Commands::MigrateRollback { step } => commands::migrate::handle_migrate_rollback_command(step),
        Commands::MigrateReset => commands::migrate::handle_migrate_reset_command(),
        Commands::MigrateRefresh { seed } => commands::migrate::handle_migrate_refresh_command(seed),
        Commands::MigrateStatus => commands::migrate::handle_migrate_status_command(),
        Commands::DbSeed { class, fresh } => commands::seed::handle_seed_command(class, fresh),
        Commands::DbSeedList => commands::seed::handle_seed_list_command(),
        Commands::Serve { port, host } => commands::serve::handle_serve_command(host, port).await,
        Commands::Passport(passport_cmd) => commands::passport::handle_passport_command(passport_cmd).await,
        Commands::Route(route_cmd) => match route_cmd {
            RouteCommands::List { name, method, uri } => commands::route::handle_route_list_command_filtered(name, method, uri).await,
        },
    }
}