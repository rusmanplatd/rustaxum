//! Activity Log Usage Examples
//!
//! This file demonstrates how to use the activity log system
//! in various scenarios similar to Spatie's Laravel ActivityLog.

use anyhow::Result;
use rustaxum::app::activity_log::prelude::*;
use rustaxum::app::models::{user::User, DieselUlid};
use rustaxum::config::Config;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::load()?;

    // Create database pool
    let pool = rustaxum::database::create_pool(&config)?;

    println!("🎯 Activity Log Usage Examples");
    println!("==============================\n");

    // Example 1: Simple activity logging
    simple_activity_logging().await?;

    // Example 2: Logging with log names
    logging_with_log_names().await?;

    // Example 3: Logging activities on models
    logging_on_models().await?;

    // Example 4: Using correlation IDs
    using_correlation_ids().await?;

    // Example 5: Batch operations
    batch_operations().await?;

    // Example 6: Advanced builder pattern
    advanced_builder_pattern().await?;

    // Example 7: Querying activity logs
    querying_activity_logs().await?;

    println!("✅ All examples completed successfully!");
    Ok(())
}

/// Example 1: Simple activity logging
async fn simple_activity_logging() -> Result<()> {
    println!("1️⃣ Simple Activity Logging");
    println!("--------------------------");

    // Log a simple activity
    let log = activity!("User logged in").await?;
    println!("✅ Logged activity: {}", log.description);

    // Log activity with properties
    let log = ActivityLog::builder()
        .description("User updated profile")
        .with_property("field", "email")
        .with_property("old_value", "old@example.com")
        .with_property("new_value", "new@example.com")
        .log()
        .await?;

    println!("✅ Logged activity with properties: {}", log.description);
    println!("   Properties: {:?}\n", log.properties);

    Ok(())
}

/// Example 2: Logging with log names
async fn logging_with_log_names() -> Result<()> {
    println!("2️⃣ Logging with Log Names");
    println!("-------------------------");

    // Log to a specific log
    let log = activity_log!("auth", "User successful login").await?;
    println!("✅ Logged to 'auth' log: {}", log.description);

    let log = activity_log!("user_management", "Profile picture updated").await?;
    println!("✅ Logged to 'user_management' log: {}", log.description);

    // Using the builder with log names
    let log = ActivityLog::for_log("payments")
        .description("Payment processed successfully")
        .with_property("amount", 99.99)
        .with_property("currency", "USD")
        .event("payment_processed")
        .log()
        .await?;

    println!("✅ Logged payment event: {}", log.description);
    println!("   Event: {:?}\n", log.event);

    Ok(())
}

/// Example 3: Logging activities on models
async fn logging_on_models() -> Result<()> {
    println!("3️⃣ Logging Activities on Models");
    println!("-------------------------------");

    // Mock user for demonstration
    let user = create_mock_user();

    // Log activity performed on a user
    let log = activity_on!(user)
        .description("Profile updated")
        .event("updated")
        .log()
        .await?;

    println!("✅ Activity on user: {}", log.description);
    println!("   Subject: {} ({})", log.subject_type.unwrap_or_default(), log.subject_id.unwrap_or_default());

    // Log activity caused by a user
    let log = activity_by!(user)
        .description("Created new organization")
        .event("created")
        .log()
        .await?;

    println!("✅ Activity by user: {}", log.description);
    println!("   Causer: {} ({})", log.causer_type.unwrap_or_default(), log.causer_id.unwrap_or_default());

    // Using the LogsActivity trait
    let log = user.log_activity("Password changed").log().await?;
    println!("✅ Activity using trait: {}", log.description);

    let log = user.log_activity_as_causer("Invited team member").log().await?;
    println!("✅ Activity as causer using trait: {}", log.description);
    println!();

    Ok(())
}

/// Example 4: Using correlation IDs
async fn using_correlation_ids() -> Result<()> {
    println!("4️⃣ Using Correlation IDs");
    println!("------------------------");

    let correlation_id = DieselUlid::new();
    println!("🔗 Correlation ID: {}", correlation_id.to_string());

    // Log multiple related activities with the same correlation ID
    let log1 = activity_correlated!(correlation_id, "Started checkout process").await?;
    let log2 = activity_correlated!(correlation_id, "Validated payment method").await?;
    let log3 = activity_correlated!(correlation_id, "Order confirmed").await?;

    println!("✅ Logged correlated activities:");
    println!("   1. {}", log1.description);
    println!("   2. {}", log2.description);
    println!("   3. {}", log3.description);
    println!();

    Ok(())
}

/// Example 5: Batch operations
async fn batch_operations() -> Result<()> {
    println!("5️⃣ Batch Operations");
    println!("-------------------");

    let batch_uuid = uuid::Uuid::new_v4().to_string();
    println!("📦 Batch UUID: {}", batch_uuid);

    // Log batch operation start
    let log = activity_batch!(batch_uuid, "Bulk user import started").await?;
    println!("✅ Batch started: {}", log.description);

    // Create multiple activities in the same batch
    let activities = vec![
        NewActivityLog {
            log_name: Some("bulk_import".to_string()),
            description: "User imported: user1@example.com".to_string(),
            subject_type: Some("User".to_string()),
            subject_id: Some("user1_id".to_string()),
            causer_type: None,
            causer_id: None,
            properties: Some(json!({"email": "user1@example.com"})),
            correlation_id: None,
            batch_uuid: Some(batch_uuid.clone()),
            event: Some("imported".to_string()),
        },
        NewActivityLog {
            log_name: Some("bulk_import".to_string()),
            description: "User imported: user2@example.com".to_string(),
            subject_type: Some("User".to_string()),
            subject_id: Some("user2_id".to_string()),
            causer_type: None,
            causer_id: None,
            properties: Some(json!({"email": "user2@example.com"})),
            correlation_id: None,
            batch_uuid: Some(batch_uuid.clone()),
            event: Some("imported".to_string()),
        },
    ];

    let service = ActivityLogService::new();
    let batch_logs = service.create_batch(activities).await?;

    println!("✅ Created {} activities in batch", batch_logs.len());
    println!();

    Ok(())
}

/// Example 6: Advanced builder pattern
async fn advanced_builder_pattern() -> Result<()> {
    println!("6️⃣ Advanced Builder Pattern");
    println!("---------------------------");

    let user = create_mock_user();
    let correlation_id = DieselUlid::new();

    // Complex activity with all features
    let log = ActivityLog::builder()
        .log_name("complex_operation")
        .description("Complex business operation completed")
        .performed_on(&user)
        .caused_by(&user)
        .correlation_id(correlation_id)
        .event("complex_operation_completed")
        .with_property("operation_type", "data_processing")
        .with_property("records_processed", 1500)
        .with_property("duration_ms", 2340)
        .with_property("errors", 0)
        .with_properties(json!({
            "metadata": {
                "version": "1.0",
                "environment": "production"
            }
        }))
        .log()
        .await?;

    println!("✅ Complex activity logged:");
    println!("   Description: {}", log.description);
    println!("   Log Name: {:?}", log.log_name);
    println!("   Event: {:?}", log.event);
    println!("   Subject: {} ({})",
        log.subject_type.unwrap_or_default(),
        log.subject_id.unwrap_or_default()
    );
    println!("   Causer: {} ({})",
        log.causer_type.unwrap_or_default(),
        log.causer_id.unwrap_or_default()
    );
    println!("   Correlation ID: {:?}", log.correlation_id);
    println!("   Properties: {:?}", log.properties);
    println!();

    Ok(())
}

/// Example 7: Querying activity logs
async fn querying_activity_logs() -> Result<()> {
    println!("7️⃣ Querying Activity Logs");
    println!("-------------------------");

    let service = ActivityLogService::new();

    // Query by log name
    let auth_logs = service.find_by_log_name("auth").await?;
    println!("✅ Found {} activities in 'auth' log", auth_logs.len());

    // Query by event
    let created_logs = service.find_by_event("created").await?;
    println!("✅ Found {} 'created' events", created_logs.len());

    // Query by subject (User type)
    let user_logs = service.find_by_subject::<User>("user1_id").await?;
    println!("✅ Found {} activities for user1_id", user_logs.len());

    // Advanced query builder
    let logs = service.query()
        .log_name("payments")
        .for_event("payment_processed")
        .limit(10)
        .get()
        .await?;

    println!("✅ Found {} payment processing activities (limited to 10)", logs.len());

    // Count activities
    let count = service.query()
        .log_name("auth")
        .count()
        .await?;

    println!("✅ Total auth activities: {}", count);
    println!();

    Ok(())
}

/// Helper function to create a mock user
fn create_mock_user() -> User {
    use chrono::Utc;

    User {
        id: DieselUlid::new(),
        name: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        email_verified_at: Some(Utc::now()),
        username: Some("johndoe".to_string()),
        password: "hashed_password".to_string(),
        remember_token: None,
        password_reset_token: None,
        password_reset_expires_at: None,
        refresh_token: None,
        refresh_token_expires_at: None,
        avatar: None,
        birthdate: None,
        failed_login_attempts: 0,
        google_id: None,
        last_login_at: Some(Utc::now()),
        last_seen_at: Utc::now(),
        locale: Some("en".to_string()),
        locked_until: None,
        phone_number: None,
        phone_verified_at: None,
        zoneinfo: Some("UTC".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        created_by: None,
        updated_by: None,
        deleted_by: None,
    }
}