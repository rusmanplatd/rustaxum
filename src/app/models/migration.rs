use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use crate::app::models::{HasModelType, activity_log::HasId};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, QueryableByName)]
#[diesel(table_name = crate::schema::migrations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
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

impl HasModelType for Migration {
    fn model_type() -> &'static str {
        "Migration"
    }
}

impl HasId for Migration {
    fn id(&self) -> String {
        self.id.to_string()
    }
}