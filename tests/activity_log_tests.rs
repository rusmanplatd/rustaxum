//! Activity Log Integration Tests
//!
//! These tests verify the functionality of the activity log system
//! in various scenarios.

use anyhow::Result;
use rustaxum::app::activity_log::prelude::*;
use rustaxum::app::models::{user::User, DieselUlid};
use rustaxum::config::Config;
use rustaxum::database::{create_pool, run_migrations};
use serde_json::json;
use serial_test::serial;

// Setup function for tests
async fn setup_test_db() -> Result<rustaxum::database::DbPool> {
    let config = Config::load()?;
    let pool = create_pool(&config)?;
    run_migrations(&pool)?;
    Ok(pool)
}

// Helper function to create a test user
fn create_test_user() -> User {
    use chrono::Utc;

    User {
        id: DieselUlid::new(),
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        email_verified_at: Some(Utc::now()),
        username: Some("testuser".to_string()),
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

#[tokio::test]
#[serial]
async fn test_simple_activity_creation() -> Result<()> {
    let _pool = setup_test_db().await?;

    // Create a simple activity
    let description = "Test activity created";
    let log = ActivityLog::builder()
        .description(description)
        .log()
        .await?;

    assert_eq!(log.description, description);
    assert!(log.id.to_string().len() > 0);
    assert!(log.created_at <= chrono::Utc::now());

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_activity_with_log_name() -> Result<()> {
    let _pool = setup_test_db().await?;

    let log_name = "test_log";
    let description = "Test activity with log name";

    let log = ActivityLog::for_log(log_name)
        .description(description)
        .log()
        .await?;

    assert_eq!(log.description, description);
    assert_eq!(log.log_name, Some(log_name.to_string()));

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_activity_with_subject() -> Result<()> {
    let _pool = setup_test_db().await?;

    let user = create_test_user();
    let description = "Test activity on user";

    let log = ActivityLog::performed_on(&user)
        .description(description)
        .log()
        .await?;

    assert_eq!(log.description, description);
    assert_eq!(log.subject_type, Some("User".to_string()));
    assert_eq!(log.subject_id, Some(user.id.to_string()));

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_activity_with_causer() -> Result<()> {
    let _pool = setup_test_db().await?;

    let user = create_test_user();
    let description = "Test activity caused by user";

    let log = ActivityLog::caused_by(&user)
        .description(description)
        .log()
        .await?;

    assert_eq!(log.description, description);
    assert_eq!(log.causer_type, Some("User".to_string()));
    assert_eq!(log.causer_id, Some(user.id.to_string()));

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_activity_with_correlation_id() -> Result<()> {
    let _pool = setup_test_db().await?;

    let correlation_id = DieselUlid::new();
    let description = "Test activity with correlation";

    let log = ActivityLog::with_correlation_id(correlation_id)
        .description(description)
        .log()
        .await?;

    assert_eq!(log.description, description);
    assert_eq!(log.correlation_id, Some(correlation_id));

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_activity_with_properties() -> Result<()> {
    let _pool = setup_test_db().await?;

    let description = "Test activity with properties";
    let properties = json!({
        "action": "update",
        "field": "email",
        "old_value": "old@example.com",
        "new_value": "new@example.com"
    });

    let log = ActivityLog::builder()
        .description(description)
        .with_properties(properties.clone())
        .log()
        .await?;

    assert_eq!(log.description, description);
    assert_eq!(log.properties, Some(properties));

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_activity_builder_chaining() -> Result<()> {
    let _pool = setup_test_db().await?;

    let user = create_test_user();
    let correlation_id = DieselUlid::new();
    let description = "Complex activity with all features";

    let log = ActivityLog::builder()
        .log_name("test_complex")
        .description(description)
        .performed_on(&user)
        .caused_by(&user)
        .correlation_id(correlation_id)
        .event("test_event")
        .with_property("key1", "value1")
        .with_property("key2", 42)
        .log()
        .await?;

    assert_eq!(log.description, description);
    assert_eq!(log.log_name, Some("test_complex".to_string()));
    assert_eq!(log.subject_type, Some("User".to_string()));
    assert_eq!(log.subject_id, Some(user.id.to_string()));
    assert_eq!(log.causer_type, Some("User".to_string()));
    assert_eq!(log.causer_id, Some(user.id.to_string()));
    assert_eq!(log.correlation_id, Some(correlation_id));
    assert_eq!(log.event, Some("test_event".to_string()));
    assert!(log.properties.is_some());

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_batch_activity_creation() -> Result<()> {
    let _pool = setup_test_db().await?;

    let batch_uuid = uuid::Uuid::new_v4().to_string();
    let activities = vec![
        NewActivityLog {
            log_name: Some("batch_test".to_string()),
            description: "First batch activity".to_string(),
            subject_type: None,
            subject_id: None,
            causer_type: None,
            causer_id: None,
            properties: Some(json!({"order": 1})),
            correlation_id: None,
            batch_uuid: Some(batch_uuid.clone()),
            event: Some("batch_created".to_string()),
        },
        NewActivityLog {
            log_name: Some("batch_test".to_string()),
            description: "Second batch activity".to_string(),
            subject_type: None,
            subject_id: None,
            causer_type: None,
            causer_id: None,
            properties: Some(json!({"order": 2})),
            correlation_id: None,
            batch_uuid: Some(batch_uuid.clone()),
            event: Some("batch_created".to_string()),
        },
    ];

    let service = ActivityLogService::new();
    let logs = service.create_batch(activities).await?;

    assert_eq!(logs.len(), 2);
    assert_eq!(logs[0].batch_uuid, Some(batch_uuid.clone()));
    assert_eq!(logs[1].batch_uuid, Some(batch_uuid.clone()));

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_query_by_log_name() -> Result<()> {
    let _pool = setup_test_db().await?;

    let log_name = "query_test";

    // Create multiple activities with the same log name
    for i in 1..=3 {
        ActivityLog::for_log(log_name)
            .description(&format!("Test activity {}", i))
            .log()
            .await?;
    }

    let service = ActivityLogService::new();
    let logs = service.find_by_log_name(log_name).await?;

    assert!(logs.len() >= 3);
    for log in &logs {
        assert_eq!(log.log_name, Some(log_name.to_string()));
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_query_by_correlation_id() -> Result<()> {
    let _pool = setup_test_db().await?;

    let correlation_id = DieselUlid::new();

    // Create multiple activities with the same correlation ID
    for i in 1..=3 {
        ActivityLog::with_correlation_id(correlation_id)
            .description(&format!("Correlated activity {}", i))
            .log()
            .await?;
    }

    let service = ActivityLogService::new();
    let logs = service.find_by_correlation_id(correlation_id).await?;

    assert!(logs.len() >= 3);
    for log in &logs {
        assert_eq!(log.correlation_id, Some(correlation_id));
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_query_builder() -> Result<()> {
    let _pool = setup_test_db().await?;

    let user = create_test_user();
    let correlation_id = DieselUlid::new();

    // Create a specific activity for testing
    ActivityLog::builder()
        .log_name("builder_test")
        .description("Query builder test activity")
        .performed_on(&user)
        .correlation_id(correlation_id)
        .event("query_test")
        .log()
        .await?;

    let service = ActivityLogService::new();

    // Test query builder with multiple filters
    let logs = service.query()
        .log_name("builder_test")
        .with_correlation_id(correlation_id)
        .for_event("query_test")
        .performed_on::<User>(&user.id.to_string())
        .limit(10)
        .get()
        .await?;

    assert!(logs.len() >= 1);
    let log = &logs[0];
    assert_eq!(log.log_name, Some("builder_test".to_string()));
    assert_eq!(log.correlation_id, Some(correlation_id));
    assert_eq!(log.event, Some("query_test".to_string()));
    assert_eq!(log.subject_type, Some("User".to_string()));
    assert_eq!(log.subject_id, Some(user.id.to_string()));

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_activity_count() -> Result<()> {
    let _pool = setup_test_db().await?;

    let log_name = "count_test";

    // Create activities for counting
    for i in 1..=5 {
        ActivityLog::for_log(log_name)
            .description(&format!("Count test activity {}", i))
            .log()
            .await?;
    }

    let service = ActivityLogService::new();
    let count = service.query()
        .log_name(log_name)
        .count()
        .await?;

    assert!(count >= 5);

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_logs_activity_trait() -> Result<()> {
    let _pool = setup_test_db().await?;

    let user = create_test_user();

    // Test LogsActivity trait methods
    let log1 = user.log_activity("Profile updated").log().await?;
    assert_eq!(log1.description, "Profile updated");
    assert_eq!(log1.subject_type, Some("User".to_string()));
    assert_eq!(log1.subject_id, Some(user.id.to_string()));

    let log2 = user.log_activity_as_causer("Created organization").log().await?;
    assert_eq!(log2.description, "Created organization");
    assert_eq!(log2.causer_type, Some("User".to_string()));
    assert_eq!(log2.causer_id, Some(user.id.to_string()));

    let log3 = user.log_self_activity("Self action").log().await?;
    assert_eq!(log3.description, "Self action");
    assert_eq!(log3.subject_type, Some("User".to_string()));
    assert_eq!(log3.subject_id, Some(user.id.to_string()));
    assert_eq!(log3.causer_type, Some("User".to_string()));
    assert_eq!(log3.causer_id, Some(user.id.to_string()));

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_activity_helpers() -> Result<()> {
    let _pool = setup_test_db().await?;

    let user = create_test_user();
    let correlation_id = DieselUlid::new();

    // Test helper functions
    let log1 = helpers::log_auth_event(
        &user.id.to_string(),
        "login",
        Some(correlation_id)
    ).await?;

    assert_eq!(log1.log_name, Some("auth".to_string()));
    assert_eq!(log1.description, "User login");
    assert_eq!(log1.subject_type, Some("User".to_string()));
    assert_eq!(log1.subject_id, Some(user.id.to_string()));
    assert_eq!(log1.event, Some("login".to_string()));
    assert_eq!(log1.correlation_id, Some(correlation_id));

    let log2 = helpers::log_created(
        &user,
        Some(&user.id.to_string()),
        Some(correlation_id)
    ).await?;

    assert_eq!(log2.description, "User created");
    assert_eq!(log2.event, Some("created".to_string()));
    assert_eq!(log2.subject_type, Some("User".to_string()));
    assert_eq!(log2.causer_type, Some("User".to_string()));

    Ok(())
}

#[test]
fn test_activity_log_config() {
    use rustaxum::config::activity_log::ActivityLogConfig;

    let config = ActivityLogConfig::default();

    assert!(config.is_enabled());
    assert!(config.should_log_event("test"));
    assert!(config.should_log_model("User"));
    assert!(config.should_log_properties());
    assert!(config.is_properties_size_valid(1000));
    assert!(!config.is_properties_size_valid(100000));

    let mut config_with_exclusions = config.clone();
    config_with_exclusions.excluded_events = vec!["excluded_event".to_string()];
    config_with_exclusions.excluded_models = vec!["ExcludedModel".to_string()];

    assert!(!config_with_exclusions.should_log_event("excluded_event"));
    assert!(!config_with_exclusions.should_log_model("ExcludedModel"));
}

#[test]
fn test_correlation_context() {
    use rustaxum::app::http::middleware::correlation_middleware::CorrelationContext;

    let context = CorrelationContext::new();
    assert!(!context.id_string().is_empty());

    let id = DieselUlid::new();
    let context_with_id = CorrelationContext::with_id(id);
    assert_eq!(context_with_id.id(), id);
}