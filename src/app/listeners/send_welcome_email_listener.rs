use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use crate::app::events::{Event, EventListener};
use crate::app::events::user_registered_event::UserRegisteredEvent;

#[derive(Debug)]
pub struct SendWelcomeEmailListener {
    pub id: String,
}

impl SendWelcomeEmailListener {
    pub fn new() -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
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
        println!("SendWelcomeEmailListener handling event: {}", event.event_name());

        if event.event_name() == "UserRegistered" {
            let event_data = event.to_json();
            let user_id = event_data.get("user_id").and_then(|v| v.as_str()).unwrap_or("unknown");
            let email = event_data.get("email").and_then(|v| v.as_str()).unwrap_or("unknown");
            let name = event_data.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");

            println!("Sending welcome email to {} ({}) for user: {}", name, email, user_id);

            // Simulate email sending
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            println!("Welcome email sent successfully!");
        }

        Ok(())
    }

    fn should_queue(&self) -> bool {
        true
    }

    fn queue_name(&self) -> Option<&str> {
        Some("emails")
    }
}
