use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use crate::app::events::{Event, EventListener, ShouldQueueListener};
use chrono::Duration;

#[derive(Debug, Clone)]
pub struct SendWelcomeEmailListener {
    pub id: String,
    pub connection: Option<String>,
    pub queue: Option<String>,
    pub delay: Option<Duration>,
}

impl SendWelcomeEmailListener {
    pub fn new() -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            connection: None,
            queue: Some("emails".to_string()),
            delay: None,
        }
    }

    /// Create a new listener with custom queue settings
    pub fn on_queue(queue: &str) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            connection: None,
            queue: Some(queue.to_string()),
            delay: None,
        }
    }

    /// Create a new listener with custom connection
    pub fn on_connection(connection: &str) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            connection: Some(connection.to_string()),
            queue: Some("emails".to_string()),
            delay: None,
        }
    }
}

impl Default for SendWelcomeEmailListener {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventListener for SendWelcomeEmailListener {
    async fn handle(&self, event: Arc<dyn Event>) -> Result<()> {
        tracing::info!("SendWelcomeEmailListener handling event: {}", event.event_name());

        if event.event_name() == "UserRegistered" {
            let event_data = event.to_json();
            let user_id = event_data.get("user_id").and_then(|v| v.as_str()).unwrap_or("unknown");
            let email = event_data.get("email").and_then(|v| v.as_str()).unwrap_or("unknown");
            let name = event_data.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");

            tracing::info!("Sending welcome email to {} ({}) for user: {}", name, email, user_id);

            // Simulate email sending with potential failure
            if email == "fail@example.com" {
                return Err(anyhow::anyhow!("Failed to send email to test failure"));
            }

            // Simulate email sending delay
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            tracing::info!("Welcome email sent successfully to {}", email);
        }

        Ok(())
    }

    async fn failed(&self, event: Arc<dyn Event>, exception: &anyhow::Error) -> Result<()> {
        tracing::error!(
            "SendWelcomeEmailListener failed to handle event {}: {}",
            event.event_name(),
            exception
        );

        // Could send notification to admin, log to external service, etc.
        if let Some(email) = event.to_json().get("email").and_then(|v| v.as_str()) {
            tracing::error!("Failed to send welcome email to: {}", email);
        }

        Ok(())
    }

    fn should_queue(&self) -> bool {
        true
    }

    fn queue_name(&self) -> Option<&str> {
        self.queue.as_deref()
    }

    fn tags(&self) -> Vec<String> {
        vec!["email".to_string(), "welcome".to_string()]
    }

    fn can_retry(&self) -> bool {
        true
    }

    fn max_exceptions(&self) -> Option<u32> {
        Some(3)
    }

    fn backoff(&self) -> Vec<chrono::Duration> {
        vec![
            Duration::seconds(5),
            Duration::seconds(10),
            Duration::seconds(30),
        ]
    }
}

#[async_trait]
impl ShouldQueueListener for SendWelcomeEmailListener {
    fn queue_connection(&self) -> Option<&str> {
        self.connection.as_deref()
    }

    fn queue(&self) -> Option<&str> {
        self.queue.as_deref()
    }

    fn delay(&self) -> Option<chrono::Duration> {
        self.delay
    }

    fn tries(&self) -> Option<u32> {
        Some(3)
    }

    fn timeout(&self) -> Option<chrono::Duration> {
        Some(Duration::minutes(5))
    }

    fn middleware(&self) -> Vec<String> {
        vec!["throttle:10,1".to_string()]
    }

    fn after_commit(&self) -> bool {
        true
    }

    fn via_connection(mut self, connection: &str) -> Self {
        self.connection = Some(connection.to_string());
        self
    }

    fn via_queue(mut self, queue: &str) -> Self {
        self.queue = Some(queue.to_string());
        self
    }

    fn with_delay(mut self, delay: chrono::Duration) -> Self {
        self.delay = Some(delay);
        self
    }
}
