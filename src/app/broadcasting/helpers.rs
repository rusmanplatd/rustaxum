use anyhow::Result;
use crate::app::events::Event;
use super::{Broadcastable, BroadcastMessage, broadcast_manager};

/// Helper function to broadcast any event that implements both Event and Broadcastable
pub async fn broadcast_event<T>(event: &T) -> Result<()>
where
    T: Event + Broadcastable + Send + Sync,
{
    let manager = broadcast_manager().await;
    let manager = manager.read().await;
    manager.broadcast(event).await
}

/// Helper function to broadcast to a specific channel
pub async fn broadcast_to_channel(channel: &str, event: &str, data: serde_json::Value) -> Result<()> {
    let message = BroadcastMessage {
        channel: channel.to_string(),
        event: event.to_string(),
        data,
        timestamp: chrono::Utc::now(),
    };

    let ws_manager = super::websocket::websocket_manager().await;
    ws_manager.broadcast(message).await
}

/// Helper function to broadcast to a user's private channel
pub async fn broadcast_to_user(user_id: &str, event: &str, data: serde_json::Value) -> Result<()> {
    let channel = format!("user.{}", user_id);
    broadcast_to_channel(&channel, event, data).await
}

/// Helper function to broadcast to an organization channel
pub async fn broadcast_to_organization(org_id: &str, event: &str, data: serde_json::Value) -> Result<()> {
    let channel = format!("org.{}", org_id);
    broadcast_to_channel(&channel, event, data).await
}

/// Helper function to broadcast to a team channel
pub async fn broadcast_to_team(team_id: &str, event: &str, data: serde_json::Value) -> Result<()> {
    let channel = format!("team.{}", team_id);
    broadcast_to_channel(&channel, event, data).await
}

/// Helper function to broadcast notifications
pub async fn broadcast_notification(user_id: &str, title: &str, message: &str, action_url: Option<&str>) -> Result<()> {
    let data = serde_json::json!({
        "title": title,
        "message": message,
        "action_url": action_url,
        "timestamp": chrono::Utc::now()
    });

    broadcast_to_user(user_id, "notification", data).await
}

/// Helper function to broadcast system alerts to all users
pub async fn broadcast_system_alert(level: &str, message: &str, action_required: bool) -> Result<()> {
    let data = serde_json::json!({
        "level": level,
        "message": message,
        "action_required": action_required,
        "timestamp": chrono::Utc::now()
    });

    broadcast_to_channel("system", "alert", data).await
}

/// Laravel-style helper function to broadcast events
#[macro_export]
macro_rules! broadcast {
    // Broadcast an event that implements Broadcastable
    ($event:expr) => {
        $crate::app::broadcasting::helpers::broadcast_event(&$event).await
    };

    // Broadcast to a specific channel
    ($channel:expr, $event:expr, $data:expr) => {
        $crate::app::broadcasting::helpers::broadcast_to_channel($channel, $event, $data).await
    };
}

/// Laravel-style helper function to broadcast to users
#[macro_export]
macro_rules! broadcast_to_user {
    ($user_id:expr, $event:expr, $data:expr) => {
        $crate::app::broadcasting::helpers::broadcast_to_user($user_id, $event, $data).await
    };
}

/// Laravel-style helper function to broadcast notifications
#[macro_export]
macro_rules! notify_user {
    ($user_id:expr, $title:expr, $message:expr) => {
        $crate::app::broadcasting::helpers::broadcast_notification($user_id, $title, $message, None).await
    };
    ($user_id:expr, $title:expr, $message:expr, $action_url:expr) => {
        $crate::app::broadcasting::helpers::broadcast_notification($user_id, $title, $message, Some($action_url)).await
    };
}

/// Helper struct for fluent broadcasting interface (Laravel-style)
pub struct BroadcastBuilder {
    channels: Vec<String>,
    event: Option<String>,
    data: Option<serde_json::Value>,
}

impl BroadcastBuilder {
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            event: None,
            data: None,
        }
    }

    /// Add a channel to broadcast to
    pub fn channel(mut self, channel: &str) -> Self {
        self.channels.push(channel.to_string());
        self
    }

    /// Add multiple channels to broadcast to
    pub fn channels(mut self, channels: Vec<String>) -> Self {
        self.channels.extend(channels);
        self
    }

    /// Broadcast to a user's private channel
    pub fn to_user(mut self, user_id: &str) -> Self {
        self.channels.push(format!("user.{}", user_id));
        self
    }

    /// Broadcast to an organization channel
    pub fn to_organization(mut self, org_id: &str) -> Self {
        self.channels.push(format!("org.{}", org_id));
        self
    }

    /// Broadcast to a team channel
    pub fn to_team(mut self, team_id: &str) -> Self {
        self.channels.push(format!("team.{}", team_id));
        self
    }

    /// Set the event name
    pub fn event(mut self, event: &str) -> Self {
        self.event = Some(event.to_string());
        self
    }

    /// Set the data to broadcast
    pub fn with(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Send the broadcast
    pub async fn send(self) -> Result<()> {
        let event = self.event.unwrap_or_else(|| "broadcast".to_string());
        let data = self.data.unwrap_or_else(|| serde_json::json!({}));

        for channel in self.channels {
            broadcast_to_channel(&channel, &event, data.clone()).await?;
        }

        Ok(())
    }
}

/// Create a new broadcast builder (Laravel-style)
pub fn broadcast() -> BroadcastBuilder {
    BroadcastBuilder::new()
}

/// Laravel Broadcast facade-style helper
pub struct BroadcastFacade;

impl BroadcastFacade {
    /// Create a new broadcast builder
    pub fn channel(channel: &str) -> BroadcastBuilder {
        BroadcastBuilder::new().channel(channel)
    }

    /// Broadcast to user
    pub fn to_user(user_id: &str) -> BroadcastBuilder {
        BroadcastBuilder::new().to_user(user_id)
    }

    /// Broadcast to organization
    pub fn to_organization(org_id: &str) -> BroadcastBuilder {
        BroadcastBuilder::new().to_organization(org_id)
    }

    /// Broadcast to team
    pub fn to_team(team_id: &str) -> BroadcastBuilder {
        BroadcastBuilder::new().to_team(team_id)
    }

    /// Send a notification to a user
    pub async fn notify_user(user_id: &str, title: &str, message: &str, action_url: Option<&str>) -> Result<()> {
        broadcast_notification(user_id, title, message, action_url).await
    }

    /// Send a system alert
    pub async fn system_alert(level: &str, message: &str, action_required: bool) -> Result<()> {
        broadcast_system_alert(level, message, action_required).await
    }
}