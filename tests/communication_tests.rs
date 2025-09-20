use anyhow::Result;
use serial_test::serial;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use rustaxum::app::events::{Event, EventListener, event_dispatcher};
use rustaxum::app::events::user_registered_event::UserRegisteredEvent;
use rustaxum::app::listeners::send_welcome_email_listener::SendWelcomeEmailListener;
use rustaxum::app::mail::{mail_manager, drivers::LogDriver, Mailable, MailMessage, MailContent};
use rustaxum::app::mail::welcome_mail::WelcomeMail;
use rustaxum::app::broadcasting::{broadcast_manager, Broadcastable, LogDriver as BroadcastLogDriver};
use rustaxum::app::jobs::{job_dispatcher, Job, MemoryQueueDriver, dispatch_job};
use rustaxum::app::jobs::send_email_job::{SendEmailJob, EmailType};
use rustaxum::app::notifications::channels::{ChannelManager, Channel};
use rustaxum::app::notifications::notification::{NotificationChannel, Notifiable};

/// Test the complete event system
#[tokio::test]
#[serial]
async fn test_event_system() -> Result<()> {
    // Create a test event
    let event = UserRegisteredEvent::new(
        "test_user_123".to_string(),
        "test@example.com".to_string(),
        "Test User".to_string(),
    );

    // Test event trait methods
    assert_eq!(event.event_name(), "UserRegistered");
    assert!(event.should_queue());
    assert_eq!(event.queue_name(), Some("events"));

    let json_data = event.to_json();
    assert!(json_data.get("user_id").is_some());
    assert!(json_data.get("email").is_some());

    // Test event broadcasting
    assert_eq!(event.broadcast_channel(), "userregistered");
    let broadcast_data = event.broadcast_data();
    assert_eq!(broadcast_data.get("event").unwrap(), "UserRegistered");

    Ok(())
}

/// Test the mail system
#[tokio::test]
#[serial]
async fn test_mail_system() -> Result<()> {
    // Setup mail manager with log driver
    let manager = mail_manager().await;
    let mut manager_guard = manager.write().await;
    manager_guard.register_driver("log".to_string(), Box::new(LogDriver::new()));
    drop(manager_guard);

    // Create a welcome mail
    let welcome_mail = WelcomeMail::new(
        "test@example.com".to_string(),
        "Test User".to_string(),
    ).with_activation_link("https://example.com/activate/test123".to_string());

    // Test mailable trait methods
    assert_eq!(welcome_mail.to(), vec!["test@example.com".to_string()]);
    assert_eq!(welcome_mail.subject(), "Welcome, Test User!");
    assert!(welcome_mail.should_queue());
    assert_eq!(welcome_mail.queue_name(), Some("emails"));

    // Test building the mail message
    let mail_message = welcome_mail.build().await?;
    assert_eq!(mail_message.to, vec!["test@example.com".to_string()]);
    assert_eq!(mail_message.subject, "Welcome, Test User!");

    // Test sending the mail
    let manager = mail_manager().await;
    let manager_guard = manager.read().await;
    manager_guard.send(&welcome_mail).await?;

    Ok(())
}

/// Test the broadcasting system
#[tokio::test]
#[serial]
async fn test_broadcasting_system() -> Result<()> {
    // Setup broadcast manager
    let manager = broadcast_manager().await;
    let mut manager_guard = manager.write().await;
    manager_guard.register_driver("log".to_string(), Box::new(BroadcastLogDriver::new()));
    drop(manager_guard);

    // Create a test event
    let event = UserRegisteredEvent::new(
        "broadcast_user".to_string(),
        "broadcast@example.com".to_string(),
        "Broadcast User".to_string(),
    );

    // Test broadcasting
    rustaxum::app::broadcasting::broadcast(&event).await?;

    // Test channel-specific broadcasting
    rustaxum::app::broadcasting::broadcast_to_channel(
        "test-channel",
        serde_json::json!({"message": "Test broadcast"})
    ).await?;

    Ok(())
}

/// Test the job system
#[tokio::test]
#[serial]
async fn test_job_system() -> Result<()> {
    // Setup job dispatcher with memory driver
    let dispatcher = job_dispatcher().await;

    // Create a test email job
    let email_job = SendEmailJob::welcome(
        "job-test@example.com".to_string(),
        "Job Test User".to_string(),
        Some("https://example.com/activate/job123".to_string()),
    );

    // Test job trait methods
    assert_eq!(email_job.job_name(), "SendEmailJob");
    assert_eq!(email_job.queue_name(), "emails");
    assert_eq!(email_job.max_attempts(), 5);
    assert_eq!(email_job.priority(), 0);

    // Test job serialization
    let serialized = email_job.serialize()?;
    assert!(serialized.contains("job-test@example.com"));

    // Test job dispatch
    let job_id = dispatch_job(&email_job).await?;
    assert!(!job_id.is_empty());

    Ok(())
}

/// Test notification channels
#[tokio::test]
#[serial]
async fn test_notification_channels() -> Result<()> {
    // Create a test notifiable entity
    let test_user = TestUser {
        id: "test_123".to_string(),
        email: "test@example.com".to_string(),
        phone: Some("+1234567890".to_string()),
        slack_channel: Some("#general".to_string()),
    };

    // Test email routing
    let email = test_user.route_notification_for(&NotificationChannel::Mail).await;
    assert_eq!(email, Some("test@example.com".to_string()));

    // Test SMS routing
    let phone = test_user.route_notification_for(&NotificationChannel::Sms).await;
    assert_eq!(phone, Some("+1234567890".to_string()));

    // Test Slack routing
    let slack = test_user.route_notification_for(&NotificationChannel::Slack).await;
    assert_eq!(slack, Some("#general".to_string()));

    Ok(())
}

/// Test integration between events and listeners
#[tokio::test]
#[serial]
async fn test_event_listener_integration() -> Result<()> {
    // Create a listener
    let listener = Arc::new(SendWelcomeEmailListener::new());

    // Create an event
    let event = UserRegisteredEvent::new(
        "integration_user".to_string(),
        "integration@example.com".to_string(),
        "Integration User".to_string(),
    );

    // Test listener handling
    let event_arc: Arc<dyn Event> = Arc::new(event);
    listener.handle(event_arc).await?;

    Ok(())
}

/// Test complete workflow
#[tokio::test]
#[serial]
async fn test_complete_workflow() -> Result<()> {
    // Setup all systems
    setup_test_systems().await?;

    // Create a user registration event
    let event = UserRegisteredEvent::new(
        "workflow_user".to_string(),
        "workflow@example.com".to_string(),
        "Workflow User".to_string(),
    );

    // 1. Fire the event
    rustaxum::fire_event!(event.clone())?;

    // 2. Send welcome email job
    let email_job = SendEmailJob::welcome(
        "workflow@example.com".to_string(),
        "Workflow User".to_string(),
        None,
    );
    dispatch_job(&email_job).await?;

    // 3. Broadcast the event
    rustaxum::app::broadcasting::broadcast(&event).await?;

    // 4. Send notification
    let welcome_mail = WelcomeMail::new(
        "workflow@example.com".to_string(),
        "Workflow User".to_string(),
    );

    let manager = mail_manager().await;
    let manager_guard = manager.read().await;
    manager_guard.send(&welcome_mail).await?;

    // Give systems time to process
    sleep(Duration::from_millis(100)).await;

    Ok(())
}

/// Test error handling
#[tokio::test]
#[serial]
async fn test_error_handling() -> Result<()> {
    // Test invalid job serialization
    let invalid_job = InvalidJob {};
    let result = invalid_job.serialize();
    assert!(result.is_err());

    // Test missing notification routing
    let user_without_email = TestUser {
        id: "no_email".to_string(),
        email: "".to_string(),
        phone: None,
        slack_channel: None,
    };

    let email = user_without_email.route_notification_for(&NotificationChannel::Mail).await;
    assert_eq!(email, None);

    Ok(())
}

/// Test performance with multiple events
#[tokio::test]
#[serial]
async fn test_performance() -> Result<()> {
    setup_test_systems().await?;

    let start = std::time::Instant::now();
    let mut handles = Vec::new();

    // Fire 100 events concurrently
    for i in 0..100 {
        let event = UserRegisteredEvent::new(
            format!("perf_user_{}", i),
            format!("perf_{}@example.com", i),
            format!("Performance User {}", i),
        );

        let handle = tokio::spawn(async move {
            let _ = rustaxum::fire_event!(event);
        });
        handles.push(handle);
    }

    // Wait for all events to complete
    futures::future::join_all(handles).await;

    let duration = start.elapsed();
    println!("Processed 100 events in {:?}", duration);

    // Should complete within reasonable time
    assert!(duration < Duration::from_secs(5));

    Ok(())
}

// Helper structs and functions

#[derive(Debug, Clone)]
struct TestUser {
    id: String,
    email: String,
    phone: Option<String>,
    slack_channel: Option<String>,
}

impl Notifiable for TestUser {
    async fn route_notification_for(&self, channel: &NotificationChannel) -> Option<String> {
        match channel {
            NotificationChannel::Mail => {
                if self.email.is_empty() { None } else { Some(self.email.clone()) }
            },
            NotificationChannel::Sms => self.phone.clone(),
            NotificationChannel::Slack => self.slack_channel.clone(),
            _ => None,
        }
    }

    fn get_key(&self) -> String {
        self.id.clone()
    }
}

#[derive(Debug)]
struct InvalidJob {}

impl Job for InvalidJob {
    fn job_name(&self) -> &'static str {
        "InvalidJob"
    }

    async fn handle(&self) -> Result<()> {
        Ok(())
    }

    fn serialize(&self) -> Result<String> {
        Err(anyhow::anyhow!("Intentional serialization error"))
    }
}

async fn setup_test_systems() -> Result<()> {
    // Setup mail manager
    let mail_manager = mail_manager().await;
    let mut manager = mail_manager.write().await;
    manager.register_driver("log".to_string(), Box::new(LogDriver::new()));
    drop(manager);

    // Setup broadcast manager
    let broadcast_manager = broadcast_manager().await;
    let mut manager = broadcast_manager.write().await;
    manager.register_driver("log".to_string(), Box::new(BroadcastLogDriver::new()));
    drop(manager);

    // Setup job dispatcher
    let job_dispatcher = job_dispatcher().await;
    // Already initialized with memory driver by default

    Ok(())
}