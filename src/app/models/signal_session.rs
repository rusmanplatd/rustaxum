use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::schema::signal_sessions;
use super::DieselUlid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = signal_sessions)]
#[diesel(primary_key(id))]
pub struct SignalSession {
    pub id: DieselUlid,
    pub local_device_id: DieselUlid,
    pub remote_device_id: DieselUlid,
    pub conversation_id: DieselUlid,
    pub encrypted_session_state: String,
    pub session_algorithm: String,
    pub session_version: i32,
    pub is_active: bool,
    pub established_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub encrypted_send_counter: Option<String>,
    pub encrypted_receive_counter: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub backup_encrypted_state: Option<String>,
    pub recovery_key_hash: Option<String>,
    pub backup_created_at: Option<DateTime<Utc>>,
    pub backup_device_id: Option<DieselUlid>,
    pub is_recoverable: bool,
}
impl SignalSession {
    pub fn needs_backup(&self) -> bool {
        self.backup_encrypted_state.is_none() && self.is_active
    }

    pub fn can_recover(&self) -> bool {
        self.is_recoverable && self.recovery_key_hash.is_some()
    }
}

impl SignalSession {
    pub fn new(
        local_device_id: DieselUlid,
        remote_device_id: DieselUlid,
        conversation_id: DieselUlid,
        encrypted_session_state: String,
        session_algorithm: String,
    ) -> Self {
        let now = Utc::now();
        SignalSession {
            id: DieselUlid::new(),
            local_device_id,
            remote_device_id,
            conversation_id,
            encrypted_session_state,
            session_algorithm,
            session_version: 1,
            is_active: true,
            established_at: now,
            last_used_at: now,
            encrypted_send_counter: None,
            encrypted_receive_counter: None,
            created_at: now,
            updated_at: now,
            backup_encrypted_state: None,
            recovery_key_hash: None,
            backup_created_at: None,
            backup_device_id: None,
            is_recoverable: false,
        }
    }

    pub fn with_backup(
        mut self,
        backup_state: String,
        recovery_key_hash: String,
        backup_device_id: DieselUlid,
    ) -> Self {
        self.backup_encrypted_state = Some(backup_state);
        self.recovery_key_hash = Some(recovery_key_hash);
        self.backup_device_id = Some(backup_device_id);
        self.backup_created_at = Some(Utc::now());
        self.is_recoverable = true;
        self
    }
}