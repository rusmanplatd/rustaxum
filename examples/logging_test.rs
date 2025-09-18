use rustaxum::{config, logging::Log};
use std::collections::HashMap;
use serde_json::Value;
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Ensure logs directory exists
    fs::create_dir_all("storage/logs")?;

    // Load configuration
    let config = config::Config::load()?;

    // Initialize the logging system
    Log::init(config.logging)?;

    println!("Testing Laravel-style Logging Implementation");
    println!("==========================================\n");

    // Test basic logging
    println!("1. Testing basic log levels:");
    Log::emergency("System is unusable!");
    Log::alert("Action must be taken immediately!");
    Log::critical("Critical conditions!");
    Log::error("Error conditions!");
    Log::warning("Warning conditions!");
    Log::notice("Normal but significant conditions!");
    Log::info("Informational messages!");
    Log::debug("Debug-level messages!");

    // Test logging with context
    println!("\n2. Testing contextual logging:");
    let mut user_context = HashMap::new();
    user_context.insert("user_id".to_string(), Value::String("123".to_string()));
    user_context.insert("username".to_string(), Value::String("john_doe".to_string()));
    user_context.insert("action".to_string(), Value::String("login_attempt".to_string()));
    user_context.insert("ip_address".to_string(), Value::String("192.168.1.100".to_string()));
    user_context.insert("timestamp".to_string(), Value::String("2024-01-15 10:30:00".to_string()));

    Log::info_with_context("User login successful", user_context);

    let mut error_context = HashMap::new();
    error_context.insert("error_code".to_string(), Value::Number(serde_json::Number::from(404)));
    error_context.insert("requested_path".to_string(), Value::String("/api/nonexistent".to_string()));
    error_context.insert("method".to_string(), Value::String("GET".to_string()));

    Log::error_with_context("Resource not found", error_context);

    // Test different channels
    println!("\n3. Testing different logging channels:");

    let daily_channel = Log::channel("daily");
    daily_channel.info("This goes to daily rotating logs");
    daily_channel.warning("Daily channel warning message");

    let stderr_channel = Log::channel("stderr");
    stderr_channel.error("This error goes to stderr");

    let stack_channel = Log::channel("stack");
    stack_channel.critical("Critical message to multiple outputs");

    println!("\n4. Testing performance with multiple log entries:");
    for i in 0..100 {
        Log::debug(&format!("Performance test message #{}", i));
    }

    println!("\nLogging test completed!");
    println!("Check 'storage/logs/' directory for log files.");

    // List created log files
    if let Ok(entries) = fs::read_dir("storage/logs") {
        println!("\nCreated log files:");
        for entry in entries {
            if let Ok(entry) = entry {
                println!("  - {:?}", entry.file_name());
            }
        }
    }

    Ok(())
}