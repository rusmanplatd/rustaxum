pub mod user_registered_event;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Base trait that all events must implement
#[async_trait]
pub trait Event: Send + Sync + std::fmt::Debug {
    /// Get the event name for identification
    fn event_name(&self) -> &'static str;

    /// Get the event data as JSON value
    fn to_json(&self) -> serde_json::Value;

    /// Determine if this event should be queued for async processing
    fn should_queue(&self) -> bool {
        false
    }

    /// Get the queue name for this event (if queued)
    fn queue_name(&self) -> Option<&str> {
        None
    }
}

/// Base trait for event listeners
#[async_trait]
pub trait EventListener: Send + Sync {
    /// Handle the event
    async fn handle(&self, event: Arc<dyn Event>) -> Result<()>;

    /// Determine if this listener should be queued
    fn should_queue(&self) -> bool {
        false
    }

    /// Get the queue name for this listener (if queued)
    fn queue_name(&self) -> Option<&str> {
        None
    }
}

/// Event dispatcher that manages event firing and listener registration
pub struct EventDispatcher {
    listeners: RwLock<HashMap<String, Vec<Arc<dyn EventListener>>>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            listeners: RwLock::new(HashMap::new()),
        }
    }

    /// Register a listener for a specific event
    pub async fn listen<E: Event + 'static>(&self, listener: Arc<dyn EventListener>) {
        let event_name = std::any::type_name::<E>().to_string();
        let mut listeners = self.listeners.write().await;
        listeners.entry(event_name).or_insert_with(Vec::new).push(listener);
    }

    /// Register a listener for a specific event by name
    pub async fn listen_for(&self, event_name: String, listener: Arc<dyn EventListener>) {
        let mut listeners = self.listeners.write().await;
        listeners.entry(event_name).or_insert_with(Vec::new).push(listener);
    }

    /// Fire an event and notify all registered listeners
    pub async fn fire(&self, event: Arc<dyn Event>) -> Result<()> {
        let event_type = std::any::type_name_of_val(event.as_ref()).to_string();
        let listeners = self.listeners.read().await;

        if let Some(event_listeners) = listeners.get(&event_type) {
            for listener in event_listeners {
                if let Err(e) = listener.handle(event.clone()).await {
                    tracing::error!("Event listener failed: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Fire an event until one listener returns a non-empty result
    pub async fn until<T>(&self, event: Arc<dyn Event>, mut validator: impl FnMut(&Result<()>) -> Option<T>) -> Option<T> {
        let event_type = std::any::type_name_of_val(event.as_ref()).to_string();
        let listeners = self.listeners.read().await;

        if let Some(event_listeners) = listeners.get(&event_type) {
            for listener in event_listeners {
                let result = listener.handle(event.clone()).await;
                if let Some(value) = validator(&result) {
                    return Some(value);
                }
            }
        }

        None
    }

    /// Remove all listeners for a specific event
    pub async fn forget(&self, event_name: &str) {
        let mut listeners = self.listeners.write().await;
        listeners.remove(event_name);
    }

    /// Get all registered listeners for debugging
    pub async fn get_listeners(&self) -> HashMap<String, usize> {
        let listeners = self.listeners.read().await;
        listeners.iter().map(|(k, v)| (k.clone(), v.len())).collect()
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Global event dispatcher instance
static EVENT_DISPATCHER: tokio::sync::OnceCell<Arc<EventDispatcher>> = tokio::sync::OnceCell::const_new();

/// Get the global event dispatcher
pub async fn event_dispatcher() -> Arc<EventDispatcher> {
    EVENT_DISPATCHER.get_or_init(|| async {
        Arc::new(EventDispatcher::new())
    }).await.clone()
}

/// Fire an event using the global dispatcher
pub async fn event(event: Arc<dyn Event>) -> Result<()> {
    event_dispatcher().await.fire(event).await
}

/// Register a listener using the global dispatcher
pub async fn listen<E: Event + 'static>(listener: Arc<dyn EventListener>) {
    event_dispatcher().await.listen::<E>(listener).await;
}

/// Helper macro to create and fire events
#[macro_export]
macro_rules! fire_event {
    ($event:expr) => {
        $crate::app::events::event(std::sync::Arc::new($event)).await
    };
}

/// Helper macro to register event listeners
#[macro_export]
macro_rules! register_listener {
    ($event_type:ty, $listener:expr) => {
        $crate::app::events::listen::<$event_type>(std::sync::Arc::new($listener)).await;
    };
}