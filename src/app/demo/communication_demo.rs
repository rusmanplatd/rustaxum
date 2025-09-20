use anyhow::Result;
use std::sync::Arc;

use crate::app::events::{event_dispatcher, Event};
use crate::app::events::user_registered_event::UserRegisteredEvent;
use crate::app::listeners::send_welcome_email_listener::SendWelcomeEmailListener;
use crate::app::mail::{mail_manager, drivers::{LogDriver, SmtpDriver}};
use crate::app::mail::welcome_mail::WelcomeMail;
use crate::app::broadcasting::{broadcast_manager, LogDriver as BroadcastLogDriver, WebSocketDriver};
use crate::app::notifications::channels::ChannelManager;
use crate::app::jobs::{job_dispatcher, dispatch_job, MemoryQueueDriver};
use crate::app::jobs::send_email_job::{SendEmailJob, EmailType};
use crate::fire_event;
use crate::register_listener;
use crate::dispatch;

/// Comprehensive demo of the Communication & Events system
pub async fn run_communication_demo() -> Result<()> {
    println!("🚀 Communication & Events System Demo");
    println!("=====================================\n");

    // Initialize all systems
    setup_communication_systems().await?;

    println!("1. Testing Event System");
    println!("----------------------");
    test_event_system().await?;

    println!("\n2. Testing Mail System");
    println!("---------------------");
    test_mail_system().await?;

    println!("\n3. Testing Broadcasting System");
    println!("-----------------------------");
    test_broadcasting_system().await?;

    println!("\n4. Testing Job System");
    println!("-------------------");
    test_job_system().await?;

    println!("\n5. Testing WebSocket Broadcasting");
    println!("-------------------------------");
    test_websocket_broadcasting().await?;

    println!("\n6. Testing Complete Workflow");
    println!("---------------------------");
    test_complete_workflow().await?;

    println!("\n7. Testing Performance");
    println!("--------------------");
    test_performance().await?;

    println!("\n✅ Communication & Events Demo Complete!");

    Ok(())
}

async fn setup_communication_systems() -> Result<()> {
    println!("Setting up communication systems...");

    // Setup Mail Manager with Log driver for demo
    let mail_manager = mail_manager().await;
    let mut manager = mail_manager.write().await;
    manager.register_driver("log".to_string(), Box::new(LogDriver::new()));
    drop(manager);

    // Setup Broadcast Manager with Log driver for demo
    let broadcast_manager = broadcast_manager().await;
    let mut manager = broadcast_manager.write().await;
    manager.register_driver("log".to_string(), Box::new(BroadcastLogDriver::new()));
    manager.register_driver("websocket".to_string(), Box::new(WebSocketDriver::new()));
    drop(manager);

    // Setup Job Dispatcher with Memory driver for demo
    let job_dispatcher = job_dispatcher().await;
    // Already initialized with memory driver by default

    // Register event listeners
    let listener = Arc::new(SendWelcomeEmailListener::new());
    register_listener!(UserRegisteredEvent, listener);

    println!("✅ All systems initialized\n");
    Ok(())
}

async fn test_event_system() -> Result<()> {
    println!("Creating and firing a UserRegisteredEvent...");

    let event = UserRegisteredEvent::new(
        "user_123".to_string(),
        "john@example.com".to_string(),
        "John Doe".to_string(),
    )
    .with_metadata("source".to_string(), "web_registration".to_string())
    .with_metadata("ip_address".to_string(), "192.168.1.1".to_string());

    println!("Event data: {:?}", event);

    // Fire the event
    fire_event!(event)?;

    println!("✅ Event fired and processed by listeners");
    Ok(())
}

async fn test_mail_system() -> Result<()> {
    println!("Creating and sending a welcome email...");

    let welcome_mail = WelcomeMail::new(
        "john@example.com".to_string(),
        "John Doe".to_string(),
    )
    .with_activation_link("https://example.com/activate/abc123".to_string());

    // Send the mail using the mail manager
    let manager = mail_manager().await;
    let manager = manager.read().await;
    manager.send(&welcome_mail).await?;

    println!("✅ Welcome email sent successfully");
    Ok(())
}

async fn test_broadcasting_system() -> Result<()> {
    println!("Broadcasting an event...");

    let event = UserRegisteredEvent::new(
        "user_456".to_string(),
        "jane@example.com".to_string(),
        "Jane Smith".to_string(),
    );

    // Broadcast the event
    crate::app::broadcasting::broadcast(&event).await?;

    println!("✅ Event broadcasted successfully");
    Ok(())
}

async fn test_complete_workflow() -> Result<()> {
    println!("Running complete workflow: Registration -> Event -> Listener -> Mail -> Broadcast");

    // Simulate a user registration
    let user_id = "user_789".to_string();
    let email = "alice@example.com".to_string();
    let name = "Alice Johnson".to_string();

    println!("👤 User registered: {} ({}) - {}", name, email, user_id);

    // 1. Create the event
    let event = UserRegisteredEvent::new(user_id, email.clone(), name.clone())
        .with_metadata("registration_method".to_string(), "oauth_google".to_string())
        .with_metadata("user_agent".to_string(), "Mozilla/5.0".to_string());

    // 2. Fire the event (this will trigger listeners)
    println!("📡 Firing UserRegisteredEvent...");
    fire_event!(event.clone())?;

    // 3. Send welcome email directly
    println!("📧 Sending welcome email...");
    let welcome_mail = WelcomeMail::new(email, name)
        .with_activation_link("https://example.com/activate/xyz789".to_string());

    let manager = mail_manager().await;
    let manager = manager.read().await;
    manager.send(&welcome_mail).await?;

    // 4. Broadcast the event
    println!("📢 Broadcasting registration event...");
    crate::app::broadcasting::broadcast(&event).await?;

    println!("✅ Complete workflow executed successfully!");
    Ok(())
}

/// Demo function that can be called from main or tests
pub async fn demo_laravel_like_usage() -> Result<()> {
    println!("🎯 Laravel-like Usage Demo");
    println!("=========================\n");

    // Laravel-like: User::create() -> fires UserRegistered event
    simulate_user_creation().await?;

    // Laravel-like: Mail::to()->send()
    simulate_mail_sending().await?;

    // Laravel-like: Event::broadcast()
    simulate_event_broadcasting().await?;

    println!("✅ Laravel-like demo complete!");
    Ok(())
}

async fn simulate_user_creation() -> Result<()> {
    println!("🆕 Creating user (Laravel-like: User::create())");

    // In Laravel: User::create() would automatically fire UserRegistered event
    let user_event = UserRegisteredEvent::new(
        "user_laravel_demo".to_string(),
        "laravel@example.com".to_string(),
        "Laravel User".to_string(),
    );

    // Automatically fire event (like Laravel's event system)
    fire_event!(user_event)?;

    println!("✅ User created and events fired");
    Ok(())
}

async fn simulate_mail_sending() -> Result<()> {
    println!("📮 Sending mail (Laravel-like: Mail::to()->send())");

    // Laravel-like: Mail::to('user@example.com')->send(new WelcomeMail($user))
    let mail = WelcomeMail::new(
        "laravel@example.com".to_string(),
        "Laravel User".to_string(),
    );

    let manager = mail_manager().await;
    let manager = manager.read().await;
    manager.send(&mail).await?;

    println!("✅ Mail sent via mail manager");
    Ok(())
}

async fn simulate_event_broadcasting() -> Result<()> {
    println!("📡 Broadcasting event (Laravel-like: broadcast(new UserRegistered()))");

    let event = UserRegisteredEvent::new(
        "broadcast_user".to_string(),
        "broadcast@example.com".to_string(),
        "Broadcast User".to_string(),
    );

    // Laravel-like: broadcast(new UserRegistered($user))
    crate::app::broadcasting::broadcast(&event).await?;

    println!("✅ Event broadcasted to all channels");
    Ok(())
}

async fn test_job_system() -> Result<()> {
    println!("Creating and dispatching email jobs...");

    // Create different types of email jobs
    let welcome_job = SendEmailJob::welcome(
        "demo@example.com".to_string(),
        "Demo User".to_string(),
        Some("https://example.com/activate/demo123".to_string()),
    );

    let order_job = SendEmailJob::order_shipped(
        "customer@example.com".to_string(),
        "Customer".to_string(),
        "ORDER-12345".to_string(),
        Some("TRACK-67890".to_string()),
    );

    let newsletter_job = SendEmailJob::newsletter(
        "subscriber@example.com".to_string(),
        "Subscriber".to_string(),
        "Monthly Newsletter".to_string(),
        "This month's highlights and updates...".to_string(),
    );

    // Dispatch jobs
    println!("📧 Dispatching welcome email job...");
    let job_id1 = dispatch!(welcome_job)?;
    println!("Job dispatched with ID: {}", job_id1);

    println!("📦 Dispatching order shipped email job...");
    let job_id2 = dispatch!(order_job)?;
    println!("Job dispatched with ID: {}", job_id2);

    println!("📰 Dispatching newsletter email job...");
    let job_id3 = dispatch!(newsletter_job)?;
    println!("Job dispatched with ID: {}", job_id3);

    println!("✅ All email jobs dispatched successfully");
    Ok(())
}

async fn test_websocket_broadcasting() -> Result<()> {
    println!("Testing WebSocket broadcasting...");

    let event = UserRegisteredEvent::new(
        "websocket_user".to_string(),
        "websocket@example.com".to_string(),
        "WebSocket User".to_string(),
    );

    // Broadcast using WebSocket driver
    let broadcast_manager = broadcast_manager().await;
    let manager = broadcast_manager.read().await;

    // Simulate WebSocket broadcast
    manager.broadcast_to_channel("user.events", serde_json::json!({
        "event": "UserRegistered",
        "data": event.broadcast_data(),
        "timestamp": chrono::Utc::now()
    })).await?;

    println!("✅ WebSocket broadcast completed");
    Ok(())
}

async fn test_performance() -> Result<()> {
    println!("Running performance test with 100 concurrent operations...");

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
            // Fire event
            let _ = fire_event!(event.clone());

            // Create and dispatch job
            let job = SendEmailJob::welcome(
                format!("perf_{}@example.com", i),
                format!("Performance User {}", i),
                None,
            );
            let _ = dispatch!(job);

            // Broadcast event
            let _ = crate::app::broadcasting::broadcast(&event).await;
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    futures::future::join_all(handles).await;

    let duration = start.elapsed();
    println!("✅ Processed 100 concurrent operations in {:?}", duration);

    // Performance assertion
    if duration.as_secs() < 5 {
        println!("🚀 Excellent performance!");
    } else if duration.as_secs() < 10 {
        println!("👍 Good performance");
    } else {
        println!("⚠️  Performance could be improved");
    }

    Ok(())
}