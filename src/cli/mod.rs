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
    /// Broadcasting and WebSocket commands
    #[command(subcommand)]
    Broadcast(BroadcastCommands),
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

#[derive(Subcommand)]
pub enum BroadcastCommands {
    /// Test broadcasting to a channel
    Test {
        /// Channel to broadcast to
        #[arg(long)]
        channel: Option<String>,
        /// Message to broadcast
        #[arg(long)]
        message: Option<String>,
    },
    /// Start a standalone WebSocket server
    #[command(name = "websocket")]
    WebSocket {
        /// Port to run WebSocket server on
        #[arg(long, default_value = "8080")]
        port: u16,
    },
    /// Show broadcasting system statistics
    Stats,
    /// Send ping messages to a channel
    Ping {
        /// Channel to ping
        #[arg(long)]
        channel: Option<String>,
        /// Interval in seconds between pings
        #[arg(long, default_value = "5")]
        interval: u64,
    },
    /// List active broadcast channels
    Channels,
    /// Broadcast notification to a user
    #[command(name = "notify:user")]
    NotifyUser {
        /// User ID to notify
        user_id: String,
        /// Notification title
        title: String,
        /// Notification message
        message: String,
        /// Optional action URL
        #[arg(long)]
        action_url: Option<String>,
    },
    /// Broadcast system alert
    #[command(name = "system:alert")]
    SystemAlert {
        /// Alert level (info, warning, error, critical)
        level: String,
        /// Alert message
        message: String,
        /// Whether action is required from users
        #[arg(long)]
        action_required: Option<bool>,
    },
    /// Monitor broadcast activity
    Monitor {
        /// Duration to monitor in seconds
        #[arg(long, default_value = "30")]
        duration: u64,
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
        Commands::Broadcast(broadcast_cmd) => match broadcast_cmd {
            BroadcastCommands::Test { channel, message } => commands::broadcast::handle_broadcast_test_command(channel, message).await,
            BroadcastCommands::WebSocket { port } => commands::broadcast::handle_websocket_serve_command(Some(port)).await,
            BroadcastCommands::Stats => commands::broadcast::handle_broadcast_stats_command().await,
            BroadcastCommands::Ping { channel, interval } => commands::broadcast::handle_broadcast_ping_command(channel, Some(interval)).await,
            BroadcastCommands::Channels => commands::broadcast::handle_broadcast_channels_command().await,
            BroadcastCommands::NotifyUser { user_id, title, message, action_url } => commands::broadcast::handle_broadcast_to_user_command(user_id, title, message, action_url).await,
            BroadcastCommands::SystemAlert { level, message, action_required } => commands::broadcast::handle_system_alert_command(level, message, action_required).await,
            BroadcastCommands::Monitor { duration } => commands::broadcast::handle_broadcast_monitor_command(Some(duration)).await,
        },
    }
}