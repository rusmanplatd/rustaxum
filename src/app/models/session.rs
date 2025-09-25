use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::sessions;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SessionModel {
    pub id: String,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub payload: String,
    pub last_activity: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = sessions)]
pub struct NewSession {
    pub id: String,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub payload: String,
    pub last_activity: i32,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = sessions)]
pub struct UpdateSession {
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub payload: String,
    pub last_activity: i32,
}

impl SessionModel {
    pub fn new(
        id: String,
        payload: String,
        user_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> NewSession {
        NewSession {
            id,
            user_id,
            ip_address,
            user_agent,
            payload,
            last_activity: chrono::Utc::now().timestamp() as i32,
        }
    }

    pub fn is_expired(&self, lifetime_seconds: i32) -> bool {
        let current_time = chrono::Utc::now().timestamp() as i32;
        current_time - self.last_activity > lifetime_seconds
    }
}

impl UpdateSession {
    pub fn new(
        payload: String,
        user_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            user_id,
            ip_address,
            user_agent,
            payload,
            last_activity: chrono::Utc::now().timestamp() as i32,
        }
    }
}