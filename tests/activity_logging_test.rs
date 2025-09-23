use rustaxum::app::traits::ServiceActivityLogger;
use rustaxum::app::models::user::User;
use rustaxum::app::http::middleware::activity_logging_middleware::ActivityLogger as MiddlewareActivityLogger;
use serde_json::json;

/// Test that the ServiceActivityLogger trait can be implemented
#[test]
fn test_service_activity_logger_trait_implementation() {
    struct TestLogger;

    impl ServiceActivityLogger for TestLogger {}

    let logger = TestLogger;

    // Test that we can create activity loggers
    let basic_logger = logger.get_activity_logger();
    assert_eq!(basic_logger.log_name, "service_operation");

    let logger_with_causer = logger.get_activity_logger_with_causer("User", "123");
    assert_eq!(logger_with_causer.causer_type, Some("User".to_string()));
    assert_eq!(logger_with_causer.causer_id, Some("123".to_string()));
}

/// Test that we can create middleware ActivityLogger instances
#[test]
fn test_middleware_activity_logger() {
    let logger = MiddlewareActivityLogger::new("test_log");
    assert_eq!(logger.log_name, "test_log");
    assert!(logger.correlation_id.is_none());
    assert!(logger.causer_type.is_none());

    let logger_with_causer = logger.with_causer("User", "user123");
    assert_eq!(logger_with_causer.causer_type, Some("User".to_string()));
    assert_eq!(logger_with_causer.causer_id, Some("user123".to_string()));
}

/// Test User model methods for activity logging
#[test]
fn test_user_model_for_activity_logging() {
    let user = User::new(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "hashed_password".to_string(),
    );

    // Test HasId trait implementation
    use rustaxum::app::models::activity_log::HasId;
    assert_eq!(user.id(), user.id.to_string());

    // Test HasModelType trait implementation
    use rustaxum::app::models::HasModelType;
    assert_eq!(User::model_type(), "User");
}

/// Test activity log builder pattern
#[test]
fn test_activity_log_builder() {
    use rustaxum::app::models::activity_log::ActivityLog;

    let builder = ActivityLog::builder()
        .log_name("test_log")
        .description("Test description")
        .event("test_event");

    let new_activity = builder.build().expect("Should build successfully");

    assert_eq!(new_activity.log_name, Some("test_log".to_string()));
    assert_eq!(new_activity.description, "Test description");
    assert_eq!(new_activity.event, Some("test_event".to_string()));
}

/// Test activity log properties handling
#[test]
fn test_activity_log_properties() {
    use rustaxum::app::models::activity_log::ActivityLog;

    let builder = ActivityLog::builder()
        .description("Test with properties")
        .with_property("key1", "value1")
        .with_property("key2", 42)
        .with_property("key3", true);

    let new_activity = builder.build().expect("Should build successfully");

    let properties = new_activity.properties.expect("Should have properties");
    assert_eq!(properties["key1"], "value1");
    assert_eq!(properties["key2"], 42);
    assert_eq!(properties["key3"], true);
}