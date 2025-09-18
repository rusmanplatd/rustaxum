use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

// Import the event this listener handles
// use crate::app::events::UserRegistered;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendWelcomeEmailListener {
    pub id: String,
    pub event_data: String, // Serialized event data
    pub attempts: u32,
    pub max_attempts: u32,
    pub delay: Option<Duration>,
}

impl SendWelcomeEmailListener {
    pub fn new(event: &UserRegistered) -> Result<Self> {
        Ok(Self {
            id: ulid::Ulid::new().to_string(),
            event_data: serde_json::to_string(event)?,
            attempts: 0,
            max_attempts: 3,
            delay: None,
        })
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }

    pub fn max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    // Handle the event asynchronously (queued)
    pub async fn handle(&mut self) -> Result<()> {
        self.attempts += 1;

        // Add delay if specified
        if let Some(delay) = self.delay {
            sleep(delay).await;
        }

        println!("Handling queued event in {} (attempt {})", stringify!(SendWelcomeEmailListener), self.attempts);

        // Deserialize the event
        let event: UserRegistered = serde_json::from_str(&self.event_data)?;

        // Process the event
        self.process_event(&event).await?;

        println!("Queued event handled successfully by {}", stringify!(SendWelcomeEmailListener));
        Ok(())
    }

    async fn process_event(&self, event: &UserRegistered) -> Result<()> {
        // Implement your actual event processing logic here
        // This could be:
        // - Sending emails/notifications (queued)
        // - Processing files
        // - Making API calls
        // - Heavy computations
        // - Database operations
        // - etc.

        println!("Processing queued event: {:?}", event);

        // Example processing logic with simulated work
        match event.event_type() {
            "UserRegistered" => {
                println!("Handling UserRegistered event in queue");
                // Simulate some async work
                sleep(Duration::from_millis(100)).await;
                // Add specific handling for this event type
            },
            _ => {
                println!("Unknown event type: {}", event.event_type());
            }
        }

        Ok(())
    }

    pub fn should_retry(&self) -> bool {
        self.attempts < self.max_attempts
    }

    pub async fn failed(&self, error: &anyhow::Error) {
        println!("Queued listener {} failed: {}", self.id, error);

        if !self.should_retry() {
            println!("Listener {} exceeded max attempts, moving to failed queue", self.id);
            // Here you could store the failed job in a dead letter queue
        }
    }
}

// Queueable trait for async job processing
#[async_trait::async_trait]
pub trait QueueableListener {
    async fn dispatch(self) -> Result<()>;
}

#[async_trait::async_trait]
impl QueueableListener for SendWelcomeEmailListener {
    async fn dispatch(mut self) -> Result<()> {
        // This would typically add the listener job to a queue (Redis, database, etc.)
        // For now, we'll just execute it directly
        println!("Dispatching queued listener: {}", self.id);

        loop {
            match self.handle().await {
                Ok(()) => break,
                Err(e) => {
                    self.failed(&e).await;
                    if !self.should_retry() {
                        return Err(e);
                    }
                    // In a real implementation, you'd re-queue the listener with a delay
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }

        Ok(())
    }
}

// Synchronous listener trait for immediate processing
#[async_trait::async_trait]
pub trait EventListener<E> {
    async fn handle(&self, event: &E) -> Result<()>;
}

// Helper function to create and queue the listener
pub async fn dispatch_listener(event: &UserRegistered) -> Result<()> {
    let listener = SendWelcomeEmailListener::new(event)?;
    listener.dispatch().await
}
