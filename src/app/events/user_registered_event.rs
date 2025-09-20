use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::app::events::Event;
use crate::app::broadcasting::Broadcastable;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegisteredEvent {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub metadata: HashMap<String, String>,
}

impl UserRegisteredEvent {
    pub fn new(user_id: String, email: String, name: String) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            timestamp: chrono::Utc::now(),
            user_id,
            email,
            name,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl Event for UserRegisteredEvent {
    fn event_name(&self) -> &'static str {
        "UserRegistered"
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }

    fn should_queue(&self) -> bool {
        true
    }

    fn queue_name(&self) -> Option<&str> {
        Some("events")
    }
}

impl Broadcastable for UserRegisteredEvent {
    fn broadcast_channel(&self) -> String {
        "user.registered".to_string()
    }

    fn broadcast_data(&self) -> serde_json::Value {
        serde_json::json!({
            "event": "UserRegistered",
            "user_id": self.user_id,
            "name": self.name,
            "timestamp": self.timestamp,
            "metadata": self.metadata
        })
    }

    fn is_private(&self) -> bool {
        false
    }
}
