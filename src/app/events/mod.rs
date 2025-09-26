pub mod user_registered_event;

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Trait for event dispatching (Laravel's Dispatchable)
pub trait Dispatchable {
    /// Dispatch the event
    fn dispatch(self) -> impl std::future::Future<Output = Result<()>> + Send
    where
        Self: Event + Send + Sync + Sized + 'static,
    {
        async move {
            event_dispatcher().await.dispatch(Arc::new(self)).await
        }
    }

    /// Dispatch the event if the condition is true
    fn dispatch_if(self, condition: bool) -> impl std::future::Future<Output = Result<()>> + Send
    where
        Self: Event + Send + Sync + Sized + 'static,
    {
        async move {
            if condition {
                self.dispatch().await
            } else {
                Ok(())
            }
        }
    }

    /// Dispatch the event unless the condition is true
    fn dispatch_unless(self, condition: bool) -> impl std::future::Future<Output = Result<()>> + Send
    where
        Self: Event + Send + Sync + Sized + 'static,
    {
        async move {
            if !condition {
                self.dispatch().await
            } else {
                Ok(())
            }
        }
    }
}

/// Trait for events that interact with sockets (Laravel's InteractsWithSockets)
pub trait InteractsWithSockets {
    /// Exclude specific socket IDs from broadcasting
    fn broadcast_to_everyone_except(&self, socket_ids: Vec<String>) -> Vec<String> {
        socket_ids
    }

    /// Get the socket ID that triggered this event
    fn socket_id(&self) -> Option<String> {
        None
    }
}

/// Trait for events that should serialize models (Laravel's SerializesModels)
pub trait SerializesModels {
    /// Prepare the event for serialization
    fn prepare_for_serialization(&self) {}

    /// Restore the event after serialization
    fn restore_after_serialization(&self) {}
}

/// Trait for events that should be queued
pub trait ShouldQueue: Event {
    /// Get the queue connection name
    fn queue_connection(&self) -> Option<&str> {
        None
    }

    /// Get the queue name
    fn queue(&self) -> Option<&str> {
        None
    }

    /// Get the delay for the event
    fn delay(&self) -> Option<chrono::Duration> {
        None
    }

    /// Get the number of tries for the event
    fn tries(&self) -> Option<u32> {
        None
    }

    /// Get the timeout for the event
    fn timeout(&self) -> Option<chrono::Duration> {
        None
    }

    /// Get the middleware for the event
    fn middleware(&self) -> Vec<String> {
        vec![]
    }

    /// Determine if the event should be dispatched after all database transactions have committed
    fn after_commit(&self) -> bool {
        false
    }
}

/// Trait for events that should be dispatched after database transactions commit
pub trait ShouldDispatchAfterCommit: Event {
    /// Determine if the event should be dispatched after commit
    fn should_dispatch_after_commit(&self) -> bool {
        true
    }
}

/// Trait for events that can be broadcast
pub trait ShouldBroadcast: Event {
    /// Get the channel(s) the event should broadcast on
    fn broadcast_on(&self) -> Vec<String>;

    /// Get the event name for broadcasting
    fn broadcast_as(&self) -> Option<String> {
        None
    }

    /// Get the broadcast data
    fn broadcast_with(&self) -> serde_json::Value {
        self.to_json()
    }

    /// Determine if the broadcast should be queued
    fn should_queue_broadcast(&self) -> bool {
        true
    }
}

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

    /// Get the tags for the event
    fn tags(&self) -> Vec<String> {
        vec![]
    }

    /// Determine if the event should be handled synchronously
    fn is_synchronous(&self) -> bool {
        !self.should_queue()
    }

    /// Get event priority (higher number = higher priority)
    fn priority(&self) -> i32 {
        0
    }

    /// Determine if the event can be unqueued
    fn can_unqueue(&self) -> bool {
        true
    }
}

/// Auto-implement Dispatchable for all Events
impl<T: Event + Send + Sync + 'static> Dispatchable for T {}

/// Trait for listeners that should be queued (Laravel's ShouldQueue)
pub trait ShouldQueueListener: EventListener {
    /// Get the queue connection name
    fn queue_connection(&self) -> Option<&str> {
        None
    }

    /// Get the queue name
    fn queue(&self) -> Option<&str> {
        None
    }

    /// Get the delay for the listener
    fn delay(&self) -> Option<chrono::Duration> {
        None
    }

    /// Get the number of tries for the listener
    fn tries(&self) -> Option<u32> {
        None
    }

    /// Get the timeout for the listener
    fn timeout(&self) -> Option<chrono::Duration> {
        None
    }

    /// Get the middleware for the listener
    fn middleware(&self) -> Vec<String> {
        vec![]
    }

    /// Determine if the listener should be handled after all database transactions have committed
    fn after_commit(&self) -> bool {
        false
    }

    /// Get the retry delay strategy
    fn retry_after(&self, attempt: u32) -> Option<chrono::Duration> {
        // Exponential backoff by default
        Some(chrono::Duration::seconds((2_i64).pow(attempt) * 5))
    }

    /// Set the queue connection
    fn via_connection(self, connection: &str) -> Self
    where
        Self: Sized;

    /// Set the queue name
    fn via_queue(self, queue: &str) -> Self
    where
        Self: Sized;

    /// Set the delay
    fn with_delay(self, delay: chrono::Duration) -> Self
    where
        Self: Sized;
}

/// Trait for listeners that should be queued after database transactions commit
pub trait ShouldQueueAfterCommit: ShouldQueueListener {
    /// Determine if the listener should be queued after commit
    fn should_queue_after_commit(&self) -> bool {
        true
    }
}

/// Base trait for event listeners
#[async_trait]
pub trait EventListener: Send + Sync {
    /// Handle the event
    async fn handle(&self, event: Arc<dyn Event>) -> Result<()>;

    /// Handle the event failure (Laravel's failed method)
    async fn failed(&self, _event: Arc<dyn Event>, _exception: &anyhow::Error) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }

    /// Determine if this listener should be queued
    fn should_queue(&self) -> bool {
        false
    }

    /// Get the queue name for this listener (if queued)
    fn queue_name(&self) -> Option<&str> {
        None
    }

    /// Get the tags for the listener
    fn tags(&self) -> Vec<String> {
        vec![]
    }

    /// Determine if the listener should fail the entire job queue if it fails
    fn fail_on_timeout(&self) -> bool {
        false
    }

    /// Determine if the listener should be retried if it fails
    fn can_retry(&self) -> bool {
        true
    }

    /// Get listener priority (higher number = higher priority)
    fn priority(&self) -> i32 {
        0
    }

    /// Determine if processing should stop after this listener
    fn halt_on_failure(&self) -> bool {
        false
    }

    /// Get the maximum number of times the listener should be retried
    fn max_exceptions(&self) -> Option<u32> {
        None
    }

    /// Get the backoff strategy for retries
    fn backoff(&self) -> Vec<chrono::Duration> {
        vec![
            chrono::Duration::seconds(1),
            chrono::Duration::seconds(5),
            chrono::Duration::seconds(10),
        ]
    }

    /// Determine if the listener should release back to the queue on failure
    fn should_release_on_failure(&self) -> bool {
        true
    }
}

/// Event dispatcher that manages event firing and listener registration
pub struct EventDispatcher {
    listeners: RwLock<HashMap<String, Vec<Arc<dyn EventListener>>>>,
    wildcard_listeners: RwLock<Vec<Arc<dyn EventListener>>>,
    fake_events: RwLock<bool>,
    faked_events: RwLock<Vec<(String, serde_json::Value)>>,
    queueable_handler: RwLock<Option<Arc<dyn QueueableHandler>>>,
}

/// Trait for handling queueable events and listeners
#[async_trait]
pub trait QueueableHandler: Send + Sync {
    /// Queue an event for processing
    async fn queue_event(&self, event: Arc<dyn Event>) -> Result<()>;

    /// Queue a listener for processing
    async fn queue_listener(&self, listener: Arc<dyn EventListener>, event: Arc<dyn Event>) -> Result<()>;
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            listeners: RwLock::new(HashMap::new()),
            wildcard_listeners: RwLock::new(Vec::new()),
            fake_events: RwLock::new(false),
            faked_events: RwLock::new(Vec::new()),
            queueable_handler: RwLock::new(None),
        }
    }

    /// Set the queueable handler
    pub async fn set_queueable_handler(&self, handler: Arc<dyn QueueableHandler>) {
        let mut queueable_handler = self.queueable_handler.write().await;
        *queueable_handler = Some(handler);
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

    /// Register a wildcard listener that receives all events
    pub async fn listen_wildcard(&self, listener: Arc<dyn EventListener>) {
        let mut wildcard_listeners = self.wildcard_listeners.write().await;
        wildcard_listeners.push(listener);
    }

    /// Fire an event and notify all registered listeners
    pub async fn dispatch(&self, event: Arc<dyn Event>) -> Result<()> {
        // Check if events are being faked
        if *self.fake_events.read().await {
            let mut faked_events = self.faked_events.write().await;
            faked_events.push((event.event_name().to_string(), event.to_json()));
            tracing::info!("Event {} was faked", event.event_name());
            return Ok(());
        }

        // Handle broadcasting if the event implements ShouldBroadcast
        if let Err(e) = self.handle_broadcasting(event.clone()).await {
            tracing::error!("Failed to broadcast event: {}", e);
        }

        let event_type = std::any::type_name_of_val(event.as_ref()).to_string();
        let listeners = self.listeners.read().await;
        let wildcard_listeners = self.wildcard_listeners.read().await;

        // Handle specific event listeners
        if let Some(event_listeners) = listeners.get(&event_type) {
            for listener in event_listeners {
                if let Err(e) = self.handle_listener(listener.clone(), event.clone()).await {
                    // Call the failed method on the listener
                    if let Err(failed_error) = listener.failed(event.clone(), &e).await {
                        tracing::error!("Listener failed method also failed: {}", failed_error);
                    }

                    if listener.halt_on_failure() {
                        return Err(e);
                    }
                    tracing::error!("Event listener failed: {}", e);
                }
            }
        }

        // Handle wildcard listeners
        for listener in wildcard_listeners.iter() {
            if let Err(e) = self.handle_listener(listener.clone(), event.clone()).await {
                // Call the failed method on the listener
                if let Err(failed_error) = listener.failed(event.clone(), &e).await {
                    tracing::error!("Wildcard listener failed method also failed: {}", failed_error);
                }

                if listener.halt_on_failure() {
                    return Err(e);
                }
                tracing::error!("Wildcard event listener failed: {}", e);
            }
        }

        Ok(())
    }

    /// Handle broadcasting for events that implement ShouldBroadcast
    async fn handle_broadcasting(&self, event: Arc<dyn Event>) -> Result<()> {
        // For now, we'll use a trait-based approach for events that implement Broadcastable
        // This is a simplified approach - in production, you might want to use a more
        // sophisticated registry or reflection system

        // Check if this is a UserRegisteredEvent
        let event_name = event.event_name();
        if event_name == "UserRegistered" {
            // Create a broadcastable version of the event
            let event_data = event.to_json();
            if let (Some(user_id), Some(email), Some(name)) = (
                event_data.get("user_id").and_then(|v| v.as_str()),
                event_data.get("email").and_then(|v| v.as_str()),
                event_data.get("name").and_then(|v| v.as_str()),
            ) {
                let user_event = crate::app::events::user_registered_event::UserRegisteredEvent::new(
                    user_id.to_string(),
                    email.to_string(),
                    name.to_string(),
                );
                return self.broadcast_event(&user_event).await;
            }
        }

        Ok(())
    }

    /// Broadcast an event using the broadcasting system
    async fn broadcast_event(&self, broadcastable: &dyn crate::app::broadcasting::Broadcastable) -> Result<()> {
        crate::app::broadcasting::broadcast(broadcastable).await
    }

    /// Fire an event and notify all registered listeners (alias for dispatch)
    pub async fn fire(&self, event: Arc<dyn Event>) -> Result<()> {
        self.dispatch(event).await
    }

    /// Handle a single listener
    async fn handle_listener(&self, listener: Arc<dyn EventListener>, event: Arc<dyn Event>) -> Result<()> {
        if listener.should_queue() {
            // Use the queueable handler if available
            if let Some(handler) = self.queueable_handler.read().await.as_ref() {
                return handler.queue_listener(listener, event).await;
            } else {
                tracing::warn!("Listener should be queued but no queueable handler is set, handling synchronously");
            }
        }

        listener.handle(event).await
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

    /// Fire an event and get responses from all listeners
    pub async fn dispatch_until<T>(&self, event: Arc<dyn Event>, mut collector: impl FnMut(&Result<()>) -> Option<T>) -> Vec<T> {
        let event_type = std::any::type_name_of_val(event.as_ref()).to_string();
        let listeners = self.listeners.read().await;
        let mut results = Vec::new();

        if let Some(event_listeners) = listeners.get(&event_type) {
            for listener in event_listeners {
                let result = listener.handle(event.clone()).await;
                if let Some(value) = collector(&result) {
                    results.push(value);
                }
            }
        }

        results
    }

    /// Remove all listeners for a specific event
    pub async fn forget(&self, event_name: &str) {
        let mut listeners = self.listeners.write().await;
        listeners.remove(event_name);
    }

    /// Remove all listeners
    pub async fn flush(&self) {
        let mut listeners = self.listeners.write().await;
        let mut wildcard_listeners = self.wildcard_listeners.write().await;
        listeners.clear();
        wildcard_listeners.clear();
    }

    /// Enable event faking for testing
    pub async fn fake(&self) {
        let mut fake_events = self.fake_events.write().await;
        let mut faked_events = self.faked_events.write().await;
        *fake_events = true;
        faked_events.clear();
    }

    /// Disable event faking
    pub async fn restore(&self) {
        let mut fake_events = self.fake_events.write().await;
        let mut faked_events = self.faked_events.write().await;
        *fake_events = false;
        faked_events.clear();
    }

    /// Get faked events for testing assertions
    pub async fn get_faked_events(&self) -> Vec<(String, serde_json::Value)> {
        self.faked_events.read().await.clone()
    }

    /// Assert that an event was dispatched (Laravel-style testing)
    pub async fn assert_dispatched(&self, event_name: &str) -> bool {
        let faked_events = self.faked_events.read().await;
        faked_events.iter().any(|(name, _)| name == event_name)
    }

    /// Assert that an event was not dispatched
    pub async fn assert_not_dispatched(&self, event_name: &str) -> bool {
        !self.assert_dispatched(event_name).await
    }

    /// Assert that no events were dispatched
    pub async fn assert_nothing_dispatched(&self) -> bool {
        self.faked_events.read().await.is_empty()
    }

    /// Assert that events were dispatched in a specific order
    pub async fn assert_dispatched_times(&self, event_name: &str, times: usize) -> bool {
        let faked_events = self.faked_events.read().await;
        faked_events.iter().filter(|(name, _)| name == event_name).count() == times
    }

    /// Check if events are being faked
    pub async fn is_faking(&self) -> bool {
        *self.fake_events.read().await
    }

    /// Get all registered listeners for debugging
    pub async fn get_listeners(&self) -> HashMap<String, usize> {
        let listeners = self.listeners.read().await;
        listeners.iter().map(|(k, v)| (k.clone(), v.len())).collect()
    }

    /// Check if there are listeners for an event
    pub async fn has_listeners(&self, event_name: &str) -> bool {
        let listeners = self.listeners.read().await;
        let wildcard_listeners = self.wildcard_listeners.read().await;

        listeners.contains_key(event_name) || !wildcard_listeners.is_empty()
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
    event_dispatcher().await.dispatch(event).await
}

/// Fire an event using the global dispatcher (alias)
pub async fn dispatch(event: Arc<dyn Event>) -> Result<()> {
    event_dispatcher().await.dispatch(event).await
}

/// Fire an event until one listener returns a result
pub async fn dispatch_until<T>(event: Arc<dyn Event>, collector: impl FnMut(&Result<()>) -> Option<T>) -> Vec<T> {
    event_dispatcher().await.dispatch_until(event, collector).await
}

/// Register a listener using the global dispatcher
pub async fn listen<E: Event + 'static>(listener: Arc<dyn EventListener>) {
    event_dispatcher().await.listen::<E>(listener).await;
}

/// Register a wildcard listener using the global dispatcher
pub async fn listen_wildcard(listener: Arc<dyn EventListener>) {
    event_dispatcher().await.listen_wildcard(listener).await;
}

/// Enable event faking for testing
pub async fn fake() {
    event_dispatcher().await.fake().await;
}

/// Disable event faking
pub async fn restore() {
    event_dispatcher().await.restore().await;
}

/// Check if events are being faked
pub async fn is_faking() -> bool {
    event_dispatcher().await.is_faking().await
}

/// Laravel-style Event facade
pub struct EventFacade;

impl EventFacade {
    /// Enable event faking (Event::fake())
    pub async fn fake() {
        fake().await;
    }

    /// Disable event faking (Event::assertDispatched)
    pub async fn assert_dispatched(event_name: &str) -> bool {
        event_dispatcher().await.assert_dispatched(event_name).await
    }

    /// Assert event was not dispatched
    pub async fn assert_not_dispatched(event_name: &str) -> bool {
        event_dispatcher().await.assert_not_dispatched(event_name).await
    }

    /// Assert no events were dispatched
    pub async fn assert_nothing_dispatched() -> bool {
        event_dispatcher().await.assert_nothing_dispatched().await
    }

    /// Assert events were dispatched specific number of times
    pub async fn assert_dispatched_times(event_name: &str, times: usize) -> bool {
        event_dispatcher().await.assert_dispatched_times(event_name, times).await
    }

    /// Register an event listener (Event::listen)
    pub async fn listen<E: Event + 'static>(listener: Arc<dyn EventListener>) {
        listen::<E>(listener).await;
    }

    /// Dispatch an event (Event::dispatch)
    pub async fn dispatch_event(event: Arc<dyn Event>) -> Result<()> {
        dispatch(event).await
    }

    /// Fire an event until one listener returns a result
    pub async fn until<T>(event: Arc<dyn Event>, validator: impl FnMut(&Result<()>) -> Option<T>) -> Option<T> {
        event_dispatcher().await.until(event, validator).await
    }

    /// Get all registered listeners
    pub async fn get_listeners() -> HashMap<String, usize> {
        event_dispatcher().await.get_listeners().await
    }

    /// Check if there are listeners for an event
    pub async fn has_listeners(event_name: &str) -> bool {
        event_dispatcher().await.has_listeners(event_name).await
    }

    /// Forget all listeners for an event
    pub async fn forget(event_name: &str) {
        event_dispatcher().await.forget(event_name).await;
    }

    /// Flush all listeners
    pub async fn flush() {
        event_dispatcher().await.flush().await;
    }
}

/// Helper macro to create and fire events
#[macro_export]
macro_rules! fire_event {
    ($event:expr_2021) => {
        $crate::app::events::dispatch(std::sync::Arc::new($event)).await
    };
}

/// Helper macro to register event listeners
#[macro_export]
macro_rules! register_listener {
    ($event_type:ty, $listener:expr_2021) => {
        $crate::app::events::listen::<$event_type>(std::sync::Arc::new($listener)).await;
    };
}

/// Helper macro to dispatch events with automatic queue handling
#[macro_export]
macro_rules! dispatch_event {
    ($event:expr_2021) => {
        {
            let event = std::sync::Arc::new($event);
            if event.should_queue() {
                // Queue the event if it should be queued
                if let Some(handler) = $crate::app::events::event_dispatcher().await.queueable_handler.read().await.as_ref() {
                    handler.queue_event(event).await
                } else {
                    $crate::app::events::dispatch(event).await
                }
            } else {
                $crate::app::events::dispatch(event).await
            }
        }
    };
}

/// Helper macro to create event listeners with automatic registration
#[macro_export]
macro_rules! event_listener {
    ($event_type:ty, $handler:expr_2021) => {
        {
            struct GeneratedListener {
                handler: Box<dyn Fn(std::sync::Arc<dyn $crate::app::events::Event>) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>> + Send + Sync>,
            }

            #[async_trait::async_trait]
            impl $crate::app::events::EventListener for GeneratedListener {
                async fn handle(&self, event: std::sync::Arc<dyn $crate::app::events::Event>) -> anyhow::Result<()> {
                    (self.handler)(event).await
                }
            }

            let listener = GeneratedListener {
                handler: Box::new(move |event| {
                    let handler = $handler;
                    Box::pin(async move {
                        handler(event).await
                    })
                }),
            };

            $crate::app::events::listen::<$event_type>(std::sync::Arc::new(listener)).await;
        }
    };
}