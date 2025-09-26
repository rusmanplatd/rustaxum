# Event System Documentation

## Overview

This framework provides a comprehensive Laravel-inspired event system built with Rust and async/await patterns. The event system allows you to decouple different parts of your application by providing a way to fire events when certain actions occur and listen for those events to perform additional tasks.

## Table of Contents

1. [Core Concepts](#core-concepts)
2. [Event Traits](#event-traits)
3. [Creating Events](#creating-events)
4. [Creating Listeners](#creating-listeners)
5. [Event Dispatcher](#event-dispatcher)
6. [Broadcasting Events](#broadcasting-events)
7. [Queue Integration](#queue-integration)
8. [Testing Events](#testing-events)
9. [Configuration](#configuration)
10. [Best Practices](#best-practices)
11. [API Reference](#api-reference)

## Core Concepts

### Events

Events are simple data structures that represent something that has happened in your application. They carry information about what occurred and can be dispatched to notify interested listeners.

### Listeners

Listeners are handlers that respond to events. They contain the logic that should be executed when a specific event is fired.

### Event Dispatcher

The event dispatcher is responsible for managing event-listener relationships and firing events to their registered listeners.

## Event Traits

### Base Event Trait

All events must implement the `Event` trait:

```rust
#[async_trait]
pub trait Event: Send + Sync + std::fmt::Debug {
    fn event_name(&self) -> &'static str;
    fn to_json(&self) -> serde_json::Value;
    fn should_queue(&self) -> bool { false }
    fn queue_name(&self) -> Option<&str> { None }
    fn tags(&self) -> Vec<String> { vec![] }
    fn is_synchronous(&self) -> bool { !self.should_queue() }
    fn priority(&self) -> i32 { 0 }
    fn can_unqueue(&self) -> bool { true }
}
```

### Dispatchable Trait

Events automatically implement the `Dispatchable` trait, providing Laravel-style dispatch methods:

```rust
pub trait Dispatchable {
    fn dispatch(self) -> impl std::future::Future<Output = Result<()>> + Send;
    fn dispatch_if(self, condition: bool) -> impl std::future::Future<Output = Result<()>> + Send;
    fn dispatch_unless(self, condition: bool) -> impl std::future::Future<Output = Result<()>> + Send;
}
```

### ShouldQueue Trait

For events that should be processed asynchronously:

```rust
pub trait ShouldQueue: Event {
    fn queue_connection(&self) -> Option<&str> { None }
    fn queue(&self) -> Option<&str> { None }
    fn delay(&self) -> Option<chrono::Duration> { None }
    fn tries(&self) -> Option<u32> { None }
    fn timeout(&self) -> Option<chrono::Duration> { None }
    fn after_commit(&self) -> bool { false }
}
```

### ShouldBroadcast Trait

For events that should be broadcast to WebSocket channels:

```rust
pub trait ShouldBroadcast: Event {
    fn broadcast_on(&self) -> Vec<String>;
    fn broadcast_as(&self) -> Option<String> { None }
    fn broadcast_with(&self) -> serde_json::Value { self.to_json() }
    fn should_queue_broadcast(&self) -> bool { true }
}
```

## Creating Events

### Using the CLI Generator

Generate a new event using the Artisan CLI:

```bash
cargo run --bin artisan -- make:event UserRegistered
cargo run --bin artisan -- make:event OrderProcessed
cargo run --bin artisan -- make:event ProductUpdated
```

### Manual Event Creation

Create an event by implementing the required traits:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::app::events::{Event, Dispatchable};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderProcessedEvent {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub order_id: String,
    pub user_id: String,
    pub amount: f64,
    pub metadata: HashMap<String, String>,
}

impl OrderProcessedEvent {
    pub fn new(order_id: String, user_id: String, amount: f64) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            timestamp: chrono::Utc::now(),
            order_id,
            user_id,
            amount,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl Event for OrderProcessedEvent {
    fn event_name(&self) -> &'static str {
        "OrderProcessed"
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }

    fn should_queue(&self) -> bool {
        true // Process this event asynchronously
    }

    fn queue_name(&self) -> Option<&str> {
        Some("orders")
    }

    fn tags(&self) -> Vec<String> {
        vec!["order".to_string(), "commerce".to_string()]
    }

    fn priority(&self) -> i32 {
        5 // Higher priority than default
    }
}
```

### Broadcasting Events

For events that should be broadcast to WebSocket clients:

```rust
use crate::app::broadcasting::Broadcastable;

impl Broadcastable for OrderProcessedEvent {
    fn broadcast_channel(&self) -> String {
        format!("user.{}", self.user_id)
    }

    fn broadcast_data(&self) -> serde_json::Value {
        serde_json::json!({
            "event": "OrderProcessed",
            "order_id": self.order_id,
            "amount": self.amount,
            "timestamp": self.timestamp
        })
    }

    fn is_private(&self) -> bool {
        true // Only the specific user should receive this
    }
}
```

## Creating Listeners

### Using the CLI Generator

Generate listeners using the Artisan CLI:

```bash
# Basic listener
cargo run --bin artisan -- make:listener SendOrderConfirmation

# Listener for a specific event
cargo run --bin artisan -- make:listener SendOrderConfirmation --event OrderProcessed

# Queued listener
cargo run --bin artisan -- make:listener ProcessPayment --event OrderProcessed --queued
```

### Manual Listener Creation

Create a listener by implementing the `EventListener` trait:

```rust
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use crate::app::events::{Event, EventListener, ShouldQueueListener};
use chrono::Duration;

#[derive(Debug, Clone)]
pub struct SendOrderConfirmationListener {
    pub id: String,
    pub queue: Option<String>,
    pub delay: Option<Duration>,
}

impl SendOrderConfirmationListener {
    pub fn new() -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            queue: Some("emails".to_string()),
            delay: None,
        }
    }
}

#[async_trait]
impl EventListener for SendOrderConfirmationListener {
    async fn handle(&self, event: Arc<dyn Event>) -> Result<()> {
        if event.event_name() == "OrderProcessed" {
            let event_data = event.to_json();
            let order_id = event_data.get("order_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let user_id = event_data.get("user_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            tracing::info!("Sending order confirmation for order {} to user {}", order_id, user_id);

            // Your email sending logic here
            // email_service.send_order_confirmation(user_id, order_id).await?;

            tracing::info!("Order confirmation sent successfully");
        }

        Ok(())
    }

    async fn failed(&self, event: Arc<dyn Event>, exception: &anyhow::Error) -> Result<()> {
        tracing::error!(
            "Failed to send order confirmation for event {}: {}",
            event.event_name(),
            exception
        );

        // Could notify admins, log to external service, etc.
        Ok(())
    }

    fn should_queue(&self) -> bool {
        true
    }

    fn queue_name(&self) -> Option<&str> {
        self.queue.as_deref()
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
            Duration::seconds(15),
            Duration::seconds(30),
        ]
    }
}
```

### Queued Listeners

For listeners that should be processed in the background:

```rust
#[async_trait]
impl ShouldQueueListener for SendOrderConfirmationListener {
    fn queue_connection(&self) -> Option<&str> {
        None // Use default connection
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
        true // Wait for database transactions to commit
    }

    fn via_connection(mut self, connection: &str) -> Self {
        // Implementation for setting connection
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
```

## Event Dispatcher

### Basic Usage

The event dispatcher manages the relationship between events and listeners:

```rust
use crate::app::events::{event_dispatcher, dispatch, listen};
use std::sync::Arc;

// Register a listener
let listener = Arc::new(SendOrderConfirmationListener::new());
listen::<OrderProcessedEvent>(listener).await;

// Dispatch an event
let event = OrderProcessedEvent::new(
    "order_123".to_string(),
    "user_456".to_string(),
    99.99,
);
dispatch(Arc::new(event)).await?;
```

### Wildcard Listeners

Listen to all events with wildcard listeners:

```rust
use crate::app::events::listen_wildcard;

#[derive(Debug)]
struct AuditListener;

#[async_trait]
impl EventListener for AuditListener {
    async fn handle(&self, event: Arc<dyn Event>) -> Result<()> {
        tracing::info!("Audit: Event {} was fired", event.event_name());
        // Log all events for auditing purposes
        Ok(())
    }
}

// Register the wildcard listener
listen_wildcard(Arc::new(AuditListener)).await;
```

### Event Dispatcher Methods

The dispatcher provides several useful methods:

```rust
let dispatcher = event_dispatcher().await;

// Fire event until one listener returns a result
let result = dispatcher.until(event.clone(), |result| {
    result.as_ref().ok().map(|_| "success")
}).await;

// Check if there are listeners for an event
let has_listeners = dispatcher.has_listeners("OrderProcessed").await;

// Get all registered listeners (for debugging)
let listeners = dispatcher.get_listeners().await;

// Remove listeners
dispatcher.forget("OrderProcessed").await;
dispatcher.flush().await; // Remove all listeners
```

## Broadcasting Events

Events can be automatically broadcast to WebSocket channels when they implement the `Broadcastable` trait:

```rust
use crate::app::broadcasting::Broadcastable;

impl Broadcastable for OrderProcessedEvent {
    fn broadcast_channel(&self) -> String {
        format!("orders.{}", self.order_id)
    }

    fn broadcast_data(&self) -> serde_json::Value {
        serde_json::json!({
            "event": "order.processed",
            "data": {
                "order_id": self.order_id,
                "status": "completed",
                "timestamp": self.timestamp
            }
        })
    }

    fn is_private(&self) -> bool {
        false
    }
}
```

The event system automatically handles broadcasting for events that implement `Broadcastable`.

## Queue Integration

### Queueable Events

Events can be queued for background processing:

```rust
impl ShouldQueue for OrderProcessedEvent {
    fn queue_connection(&self) -> Option<&str> {
        Some("redis") // Use Redis queue
    }

    fn queue(&self) -> Option<&str> {
        Some("orders") // Use orders queue
    }

    fn delay(&self) -> Option<chrono::Duration> {
        Some(chrono::Duration::seconds(30)) // Delay 30 seconds
    }

    fn tries(&self) -> Option<u32> {
        Some(5) // Retry up to 5 times
    }

    fn after_commit(&self) -> bool {
        true // Wait for DB transactions to commit
    }
}
```

### Queue Handler

Implement a custom queue handler for processing queued events and listeners:

```rust
use crate::app::events::QueueableHandler;

pub struct RedisQueueHandler {
    // Redis connection, etc.
}

#[async_trait]
impl QueueableHandler for RedisQueueHandler {
    async fn queue_event(&self, event: Arc<dyn Event>) -> Result<()> {
        // Serialize event and add to Redis queue
        Ok(())
    }

    async fn queue_listener(&self, listener: Arc<dyn EventListener>, event: Arc<dyn Event>) -> Result<()> {
        // Serialize listener + event and add to queue
        Ok(())
    }
}

// Set the queue handler
let queue_handler = Arc::new(RedisQueueHandler::new());
event_dispatcher().await.set_queueable_handler(queue_handler).await;
```

## Testing Events

### Event Faking

The event system provides comprehensive testing support through event faking:

```rust
use crate::app::events::{EventFacade, fake, restore};

#[tokio::test]
async fn test_order_processing() {
    // Enable event faking
    EventFacade::fake().await;

    // Your application code that dispatches events
    let order = create_test_order().await;
    process_order(order).await?;

    // Assert events were dispatched
    assert!(EventFacade::assert_dispatched("OrderProcessed").await);
    assert!(EventFacade::assert_dispatched_times("OrderProcessed", 1).await);
    assert!(EventFacade::assert_not_dispatched("OrderCancelled").await);

    // Clean up
    restore().await;
}
```

### Testing Listeners

Test listeners in isolation:

```rust
#[tokio::test]
async fn test_order_confirmation_listener() {
    let listener = SendOrderConfirmationListener::new();
    let event = Arc::new(OrderProcessedEvent::new(
        "order_123".to_string(),
        "user_456".to_string(),
        99.99,
    ));

    let result = listener.handle(event).await;
    assert!(result.is_ok());
}
```

### Testing Event Broadcasting

Test that events are properly broadcast:

```rust
#[tokio::test]
async fn test_event_broadcasting() {
    fake().await;

    let event = OrderProcessedEvent::new(
        "order_123".to_string(),
        "user_456".to_string(),
        99.99,
    );

    event.dispatch().await?;

    // Check broadcasting was triggered
    // This would depend on your broadcasting implementation

    restore().await;
}
```

## Configuration

Configure the event system through environment variables:

```env
# .env
EVENTS_DEFAULT_QUEUE=events
EVENTS_STORE_ENABLED=true
EVENTS_STORE_TABLE=events
EVENTS_RETRY_ATTEMPTS=3
EVENTS_RETRY_DELAY_SECONDS=60
```

The configuration is loaded through the `EventsConfig` struct:

```rust
use crate::config::events::EventsConfig;

let config = EventsConfig::from_env()?;
println!("Default queue: {}", config.default_queue);
```

## Best Practices

### 1. Event Naming

- Use descriptive, past-tense names: `UserRegistered`, `OrderProcessed`, `PaymentCompleted`
- Be specific about what happened: `ProductInventoryUpdated` vs `ProductChanged`

### 2. Event Data

- Include all relevant data in the event to avoid additional database queries in listeners
- Use immutable data structures when possible
- Include timestamps and correlation IDs for tracking

### 3. Listener Design

- Keep listeners focused on a single responsibility
- Make listeners idempotent (safe to run multiple times)
- Handle failures gracefully with proper error logging
- Use queued listeners for time-consuming operations

### 4. Error Handling

- Implement the `failed` method for listeners to handle errors
- Use appropriate retry strategies and backoff policies
- Log errors with sufficient context for debugging
- Consider dead letter queues for failed events

### 5. Testing

- Use event faking for unit tests
- Test listeners in isolation
- Test the integration between events and listeners
- Mock external dependencies in listener tests

### 6. Performance

- Use queued events for non-critical operations
- Implement proper queue monitoring and scaling
- Consider event batching for high-volume scenarios
- Monitor event processing latency

## API Reference

### Core Functions

```rust
// Dispatch events
pub async fn dispatch(event: Arc<dyn Event>) -> Result<()>
pub async fn event(event: Arc<dyn Event>) -> Result<()> // Alias

// Register listeners
pub async fn listen<E: Event + 'static>(listener: Arc<dyn EventListener>)
pub async fn listen_wildcard(listener: Arc<dyn EventListener>)

// Testing
pub async fn fake()
pub async fn restore()
pub async fn is_faking() -> bool
```

### Helper Macros

```rust
// Fire an event
fire_event!(OrderProcessedEvent::new("order_123".to_string(), "user_456".to_string(), 99.99));

// Register a listener
register_listener!(OrderProcessedEvent, Arc::new(SendOrderConfirmationListener::new()));

// Dispatch with automatic queue handling
dispatch_event!(OrderProcessedEvent::new("order_123".to_string(), "user_456".to_string(), 99.99));
```

### EventFacade Methods

```rust
impl EventFacade {
    pub async fn fake()
    pub async fn assert_dispatched(event_name: &str) -> bool
    pub async fn assert_not_dispatched(event_name: &str) -> bool
    pub async fn assert_nothing_dispatched() -> bool
    pub async fn assert_dispatched_times(event_name: &str, times: usize) -> bool
    pub async fn listen<E: Event + 'static>(listener: Arc<dyn EventListener>)
    pub async fn dispatch_event(event: Arc<dyn Event>) -> Result<()>
    pub async fn has_listeners(event_name: &str) -> bool
    pub async fn get_listeners() -> HashMap<String, usize>
    pub async fn forget(event_name: &str)
    pub async fn flush()
}
```

## Example: Complete Order Processing System

Here's a complete example showing how to implement an order processing system with events:

```rust
// 1. Create the event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreatedEvent {
    pub order_id: String,
    pub user_id: String,
    pub items: Vec<OrderItem>,
    pub total: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Event for OrderCreatedEvent {
    fn event_name(&self) -> &'static str { "OrderCreated" }
    fn to_json(&self) -> serde_json::Value { serde_json::to_value(self).unwrap_or_default() }
    fn should_queue(&self) -> bool { true }
}

// 2. Create listeners
pub struct SendOrderConfirmationListener;
pub struct UpdateInventoryListener;
pub struct ProcessPaymentListener;

// 3. Register listeners (typically in your app startup)
pub async fn register_order_listeners() -> Result<()> {
    listen::<OrderCreatedEvent>(Arc::new(SendOrderConfirmationListener)).await;
    listen::<OrderCreatedEvent>(Arc::new(UpdateInventoryListener)).await;
    listen::<OrderCreatedEvent>(Arc::new(ProcessPaymentListener)).await;
    Ok(())
}

// 4. Dispatch the event (in your order controller)
pub async fn create_order(order_data: CreateOrderRequest) -> Result<()> {
    // Create order in database
    let order = create_order_in_db(order_data).await?;

    // Dispatch event
    let event = OrderCreatedEvent {
        order_id: order.id,
        user_id: order.user_id,
        items: order.items,
        total: order.total,
        timestamp: chrono::Utc::now(),
    };

    event.dispatch().await?;
    Ok(())
}
```

This comprehensive event system provides all the tools needed to build a robust, scalable application with proper separation of concerns and excellent testability.
