use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegisteredEvent {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: EventData,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    // Add event-specific data fields here
    pub message: String,
    pub user_id: Option<String>,
}

impl UserRegisteredEvent {
    pub fn new(data: EventData) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            timestamp: chrono::Utc::now(),
            data,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn user_id(mut self, user_id: String) -> Self {
        self.data.user_id = Some(user_id);
        self
    }

    pub async fn dispatch(self) -> Result<()> {
        // Dispatch the event to all registered listeners
        println!("Dispatching event: {} at {}", self.id, self.timestamp);

        // In a real implementation, this would:
        // 1. Get all listeners for this event type
        // 2. Execute each listener (either sync or async)
        // 3. Handle any listener failures gracefully

        self.broadcast().await?;
        self.store().await?;

        Ok(())
    }

    async fn broadcast(&self) -> Result<()> {
        // Broadcast the event using tokio's broadcast channel
        // This allows multiple listeners to receive the same event
        println!("Broadcasting event {} to listeners", self.id);

        // Example: Send to broadcast channel
        // In a real app, you'd have a global event broadcaster
        // EVENT_BROADCASTER.send(self.clone())?;

        Ok(())
    }

    async fn store(&self) -> Result<()> {
        // Store the event for audit/replay purposes
        println!("Storing event {} for audit trail", self.id);

        // In a real implementation, this would store to database:
        // sqlx::query!(
        //     "INSERT INTO events (id, event_type, data, timestamp) VALUES ($1, $2, $3, $4)",
        //     self.id,
        //     stringify!(UserRegisteredEvent),
        //     serde_json::to_string(&self.data)?,
        //     self.timestamp
        // ).execute(pool).await?;

        Ok(())
    }

    pub fn event_type() -> &'static str {
        stringify!(UserRegisteredEvent)
    }
}

// Event trait for type-safe event handling
#[async_trait::async_trait]
pub trait Event: Send + Sync + Clone {
    async fn dispatch(self) -> Result<()>;
    fn event_type() -> &'static str;
}

#[async_trait::async_trait]
impl Event for UserRegisteredEvent {
    async fn dispatch(self) -> Result<()> {
        Self::dispatch(self).await
    }

    fn event_type() -> &'static str {
        Self::event_type()
    }
}

// Event dispatcher for handling multiple event types
pub struct EventDispatcher {
    // In a real implementation, this would hold:
    // - Registered listeners for each event type
    // - Broadcasting channels
    // - Error handling strategies
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn dispatch<E: Event>(&self, event: E) -> Result<()> {
        event.dispatch().await
    }

    // Register a listener for this event type
    pub fn listen<F, Fut>(&self, _handler: F) -> Result<()>
    where
        F: Fn(UserRegisteredEvent) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        // Register the handler for this event type
        println!("Registered listener for {}", Self::event_type());
        Ok(())
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
