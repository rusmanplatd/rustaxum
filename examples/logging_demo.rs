use rustaxum::{config, logging::Log};
use std::collections::HashMap;
use serde_json::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = config::Config::load()?;

    // Initialize the logging system
    Log::init(config.logging)?;

    println!("Laravel-style Logging Demo");
    println!("=========================\n");

    // Basic logging without context
    println!("1. Basic logging methods:");
    Log::emergency("This is an emergency!");
    Log::alert("This is an alert!");
    Log::critical("This is critical!");
    Log::error("This is an error!");
    Log::warning("This is a warning!");
    Log::notice("This is a notice!");
    Log::info("This is info!");
    Log::debug("This is debug!");

    // Logging with context
    println!("\n2. Logging with context:");
    let mut context = HashMap::new();
    context.insert("user_id".to_string(), Value::String("12345".to_string()));
    context.insert("action".to_string(), Value::String("login".to_string()));
    context.insert("ip_address".to_string(), Value::String("192.168.1.100".to_string()));

    Log::info_with_context("User logged in successfully", context);

    let mut error_context = HashMap::new();
    error_context.insert("error_code".to_string(), Value::Number(serde_json::Number::from(500)));
    error_context.insert("file".to_string(), Value::String("/src/controllers/auth.rs".to_string()));
    error_context.insert("line".to_string(), Value::Number(serde_json::Number::from(42)));

    Log::error_with_context("Database connection failed", error_context);

    // Channel-specific logging
    println!("\n3. Channel-specific logging:");
    let daily_logger = Log::channel("daily");
    daily_logger.info("This message goes to the daily channel");

    let stderr_logger = Log::channel("stderr");
    stderr_logger.warning("This warning goes to stderr");

    let stack_logger = Log::channel("stack");
    stack_logger.error("This error goes to multiple outputs");

    println!("\nCheck the 'storage/logs/' directory for log files!");

    Ok(())
}