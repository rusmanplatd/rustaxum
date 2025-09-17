use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub id: i32,
    pub migration: String,
    pub batch: i32,
    pub executed_at: DateTime<Utc>,
}

impl Migration {
    pub fn new(migration: String, batch: i32) -> Self {
        Self {
            id: 0, // Will be set by database
            migration,
            batch,
            executed_at: Utc::now(),
        }
    }
}