use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::device_session_backups)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeviceSessionBackup {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub device_id: DieselUlid,
    pub user_id: DieselUlid,
    pub backup_name: String,
    pub backup_type: String,
    pub backup_version: i32,
    pub encrypted_sessions_data: String,
    pub backup_algorithm: String,
    pub backup_key_hash: String,
    pub sessions_count: i32,
    pub conversations_count: i32,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub backup_checksum: String,
    pub is_verified: bool,
    pub verification_failed_at: Option<DateTime<Utc>>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionBackupType {
    Full,      // Complete device session backup
    Partial,   // Selected conversations backup
    Metadata,  // Session metadata only
    Emergency, // Emergency backup with minimal data
}

impl From<String> for SessionBackupType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "full" => SessionBackupType::Full,
            "partial" => SessionBackupType::Partial,
            "metadata" => SessionBackupType::Metadata,
            "emergency" => SessionBackupType::Emergency,
            _ => SessionBackupType::Full,
        }
    }
}

impl From<SessionBackupType> for String {
    fn from(backup_type: SessionBackupType) -> Self {
        match backup_type {
            SessionBackupType::Full => "full".to_string(),
            SessionBackupType::Partial => "partial".to_string(),
            SessionBackupType::Metadata => "metadata".to_string(),
            SessionBackupType::Emergency => "emergency".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateDeviceSessionBackup {
    pub device_id: DieselUlid,
    pub backup_name: String,
    pub backup_type: String,
    pub encrypted_sessions_data: String,
    pub backup_algorithm: String,
    pub backup_key_hash: String,
    pub sessions_count: i32,
    pub conversations_count: i32,
    pub expires_in_days: Option<i32>,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceSessionBackupResponse {
    pub id: DieselUlid,
    pub device_id: DieselUlid,
    pub user_id: DieselUlid,
    pub backup_name: String,
    pub backup_type: String,
    pub backup_version: i32,
    pub backup_algorithm: String,
    pub sessions_count: i32,
    pub conversations_count: i32,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub is_verified: bool,
    pub verification_failed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl DeviceSessionBackup {
    pub fn new(
        device_id: DieselUlid,
        user_id: DieselUlid,
        backup_name: String,
        backup_type: SessionBackupType,
        encrypted_sessions_data: String,
        backup_algorithm: String,
        backup_key_hash: String,
        sessions_count: i32,
        conversations_count: i32,
        expires_in_days: Option<i32>,
    ) -> Self {
        let now = Utc::now();
        let expires_in_days = expires_in_days.unwrap_or(90); // Default 3 months
        let expires_at = now + chrono::Duration::days(expires_in_days as i64);
        let backup_checksum = Self::calculate_checksum(&encrypted_sessions_data);

        DeviceSessionBackup {
            id: DieselUlid::new(),
            device_id,
            user_id,
            backup_name,
            backup_type: backup_type.into(),
            backup_version: 1,
            encrypted_sessions_data,
            backup_algorithm,
            backup_key_hash,
            sessions_count,
            conversations_count,
            created_at: now,
            expires_at,
            last_accessed_at: None,
            backup_checksum,
            is_verified: false,
            verification_failed_at: None,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> DeviceSessionBackupResponse {
        DeviceSessionBackupResponse {
            id: self.id,
            device_id: self.device_id,
            user_id: self.user_id,
            backup_name: self.backup_name.clone(),
            backup_type: self.backup_type.clone(),
            backup_version: self.backup_version,
            backup_algorithm: self.backup_algorithm.clone(),
            sessions_count: self.sessions_count,
            conversations_count: self.conversations_count,
            created_at: self.created_at,
            expires_at: self.expires_at,
            last_accessed_at: self.last_accessed_at,
            is_verified: self.is_verified,
            verification_failed_at: self.verification_failed_at,
            updated_at: self.updated_at,
        }
    }

    pub fn backup_type_enum(&self) -> SessionBackupType {
        self.backup_type.clone().into()
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        self.is_verified && !self.is_expired()
    }

    pub fn verify(&mut self) -> bool {
        let calculated_checksum = Self::calculate_checksum(&self.encrypted_sessions_data);
        if calculated_checksum == self.backup_checksum {
            self.is_verified = true;
            self.verification_failed_at = None;
            self.updated_at = Utc::now();
            true
        } else {
            self.verification_failed_at = Some(Utc::now());
            self.updated_at = Utc::now();
            false
        }
    }

    pub fn access(&mut self) {
        self.last_accessed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn extend_expiry(&mut self, additional_days: i32) {
        self.expires_at = self.expires_at + chrono::Duration::days(additional_days as i64);
        self.updated_at = Utc::now();
    }

    pub fn is_full_backup(&self) -> bool {
        matches!(self.backup_type_enum(), SessionBackupType::Full)
    }

    pub fn is_emergency_backup(&self) -> bool {
        matches!(self.backup_type_enum(), SessionBackupType::Emergency)
    }

    pub fn get_backup_size_estimate(&self) -> usize {
        self.encrypted_sessions_data.len()
    }

    pub fn get_backup_efficiency(&self) -> f64 {
        if self.sessions_count > 0 {
            self.conversations_count as f64 / self.sessions_count as f64
        } else {
            0.0
        }
    }

    fn calculate_checksum(data: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl HasId for DeviceSessionBackup {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for DeviceSessionBackup {
    fn table_name() -> &'static str {
        "device_session_backups"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "user_id",
            "backup_name",
            "backup_type",
            "backup_version",
            "sessions_count",
            "conversations_count",
            "is_verified",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "backup_name",
            "backup_type",
            "backup_version",
            "sessions_count",
            "conversations_count",
            "is_verified",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "user_id",
            "backup_name",
            "backup_type",
            "backup_version",
            "backup_algorithm",
            "sessions_count",
            "conversations_count",
            "is_verified",
            "expires_at",
            "last_accessed_at",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "device",
            "user",
        ]
    }
}

crate::impl_query_builder_service!(DeviceSessionBackup);