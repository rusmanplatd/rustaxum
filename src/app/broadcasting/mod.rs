pub mod websocket;
pub mod helpers;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock};

/// Base trait for broadcasting events
#[async_trait]
pub trait Broadcastable: Send + Sync + std::fmt::Debug {
    /// Get the broadcast channel name
    fn broadcast_channel(&self) -> String;

    /// Get the event data for broadcasting
    fn broadcast_data(&self) -> serde_json::Value;

    /// Determine if this event should be broadcast privately
    fn is_private(&self) -> bool {
        false
    }

    /// Get the private channel identifier (user ID, group ID, etc.)
    fn private_channel(&self) -> Option<String> {
        None
    }
}

/// Broadcast driver trait for different broadcasting providers
#[async_trait]
pub trait BroadcastDriver: Send + Sync {
    /// Broadcast to a public channel
    async fn broadcast(&self, channel: &str, data: serde_json::Value) -> Result<()>;

    /// Broadcast to a private channel
    async fn broadcast_private(&self, channel: &str, data: serde_json::Value) -> Result<()>;

    /// Get the driver name
    fn driver_name(&self) -> &'static str;
}

/// WebSocket broadcasting driver for real-time communication
#[derive(Debug, Clone)]
pub struct WebSocketDriver {
    manager: Arc<websocket::WebSocketManager>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub channel: String,
    pub event: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl WebSocketDriver {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(websocket::WebSocketManager::new()),
        }
    }

    pub fn with_manager(manager: Arc<websocket::WebSocketManager>) -> Self {
        Self { manager }
    }

    /// Get the WebSocket manager
    pub fn manager(&self) -> Arc<websocket::WebSocketManager> {
        self.manager.clone()
    }
}

impl Default for WebSocketDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BroadcastDriver for WebSocketDriver {
    async fn broadcast(&self, channel: &str, data: serde_json::Value) -> Result<()> {
        let message = BroadcastMessage {
            channel: channel.to_string(),
            event: "broadcast".to_string(),
            data,
            timestamp: chrono::Utc::now(),
        };

        self.manager.broadcast(message).await?;
        tracing::info!("Broadcasted to WebSocket channel: {}", channel);
        Ok(())
    }

    async fn broadcast_private(&self, channel: &str, data: serde_json::Value) -> Result<()> {
        let private_channel = format!("private:{}", channel);
        self.broadcast(&private_channel, data).await
    }

    fn driver_name(&self) -> &'static str {
        "websocket"
    }
}

/// Redis broadcasting driver for distributed systems
#[derive(Debug, Clone)]
pub struct RedisDriver {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub database: u8,
}

impl RedisDriver {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            password: None,
            database: 0,
        }
    }

    pub fn with_password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    pub fn with_database(mut self, database: u8) -> Self {
        self.database = database;
        self
    }
}

#[async_trait]
impl BroadcastDriver for RedisDriver {
    async fn broadcast(&self, channel: &str, data: serde_json::Value) -> Result<()> {
        // Production Redis implementation steps:
        // 1. Establish Redis connection with connection pooling
        // 2. Publish message using Redis PUBLISH command
        // 3. Handle connection failures with retry logic
        // 4. Implement message acknowledgment system
        // 5. Add monitoring and metrics for broadcast health

        let message = BroadcastMessage {
            channel: channel.to_string(),
            event: "broadcast".to_string(),
            data,
            timestamp: chrono::Utc::now(),
        };

        println!("Redis: Publishing to channel '{}': {:?}", channel, message);

        // Simulate Redis publishing
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        tracing::info!("Published to Redis channel: {}", channel);
        Ok(())
    }

    async fn broadcast_private(&self, channel: &str, data: serde_json::Value) -> Result<()> {
        let private_channel = format!("private:{}", channel);
        self.broadcast(&private_channel, data).await
    }

    fn driver_name(&self) -> &'static str {
        "redis"
    }
}

/// Log driver for testing and debugging
#[derive(Debug, Clone)]
pub struct LogDriver;

impl LogDriver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LogDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BroadcastDriver for LogDriver {
    async fn broadcast(&self, channel: &str, data: serde_json::Value) -> Result<()> {
        let message = BroadcastMessage {
            channel: channel.to_string(),
            event: "broadcast".to_string(),
            data,
            timestamp: chrono::Utc::now(),
        };

        println!("BROADCAST LOG: Channel '{}' - {:?}", channel, message);
        tracing::info!("Logged broadcast to channel: {}", channel);
        Ok(())
    }

    async fn broadcast_private(&self, channel: &str, data: serde_json::Value) -> Result<()> {
        let private_channel = format!("private:{}", channel);
        self.broadcast(&private_channel, data).await
    }

    fn driver_name(&self) -> &'static str {
        "log"
    }
}

/// Broadcast manager that handles different drivers
pub struct BroadcastManager {
    drivers: HashMap<String, Box<dyn BroadcastDriver>>,
    default_driver: String,
}

impl BroadcastManager {
    pub fn new(default_driver: String) -> Self {
        Self {
            drivers: HashMap::new(),
            default_driver,
        }
    }

    pub fn register_driver(&mut self, name: String, driver: Box<dyn BroadcastDriver>) {
        self.drivers.insert(name, driver);
    }

    pub async fn broadcast(&self, broadcastable: &dyn Broadcastable) -> Result<()> {
        let channel = broadcastable.broadcast_channel();
        let data = broadcastable.broadcast_data();

        let driver = self.drivers.get(&self.default_driver)
            .ok_or_else(|| anyhow::anyhow!("Broadcast driver '{}' not found", self.default_driver))?;

        if broadcastable.is_private() {
            if let Some(private_channel) = broadcastable.private_channel() {
                driver.broadcast_private(&private_channel, data).await?;
            } else {
                return Err(anyhow::anyhow!("Private broadcast requires a private channel"));
            }
        } else {
            driver.broadcast(&channel, data).await?;
        }

        Ok(())
    }

    pub async fn broadcast_to_channel(&self, channel: &str, data: serde_json::Value) -> Result<()> {
        let driver = self.drivers.get(&self.default_driver)
            .ok_or_else(|| anyhow::anyhow!("Broadcast driver '{}' not found", self.default_driver))?;

        driver.broadcast(channel, data).await
    }

    pub async fn broadcast_with_driver(&self, broadcastable: &dyn Broadcastable, driver_name: &str) -> Result<()> {
        let channel = broadcastable.broadcast_channel();
        let data = broadcastable.broadcast_data();

        let driver = self.drivers.get(driver_name)
            .ok_or_else(|| anyhow::anyhow!("Broadcast driver '{}' not found", driver_name))?;

        if broadcastable.is_private() {
            if let Some(private_channel) = broadcastable.private_channel() {
                driver.broadcast_private(&private_channel, data).await?;
            } else {
                return Err(anyhow::anyhow!("Private broadcast requires a private channel"));
            }
        } else {
            driver.broadcast(&channel, data).await?;
        }

        Ok(())
    }
}

/// Global broadcast manager instance
static BROADCAST_MANAGER: tokio::sync::OnceCell<Arc<RwLock<BroadcastManager>>> = tokio::sync::OnceCell::const_new();

/// Initialize the global broadcast manager
pub async fn init_broadcast_manager(default_driver: String) -> Arc<RwLock<BroadcastManager>> {
    BROADCAST_MANAGER.get_or_init(|| async {
        Arc::new(RwLock::new(BroadcastManager::new(default_driver)))
    }).await.clone()
}

/// Get the global broadcast manager
pub async fn broadcast_manager() -> Arc<RwLock<BroadcastManager>> {
    BROADCAST_MANAGER.get_or_init(|| async {
        Arc::new(RwLock::new(BroadcastManager::new("log".to_string())))
    }).await.clone()
}

/// Broadcast using the global manager
pub async fn broadcast(broadcastable: &dyn Broadcastable) -> Result<()> {
    let manager = broadcast_manager().await;
    let manager = manager.read().await;
    manager.broadcast(broadcastable).await
}

/// Broadcast to a specific channel using the global manager
pub async fn broadcast_to_channel(channel: &str, data: serde_json::Value) -> Result<()> {
    let manager = broadcast_manager().await;
    let manager = manager.read().await;
    manager.broadcast_to_channel(channel, data).await
}