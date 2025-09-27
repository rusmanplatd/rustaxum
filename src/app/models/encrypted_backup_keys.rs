use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::encrypted_backup_keys)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EncryptedBackupKey {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_id: DieselUlid,
    pub encrypted_backup_data: String,
    pub backup_algorithm: String,
    pub backup_type: String,
    pub backup_size_bytes: i64,
    pub backup_hash: String,
    pub is_verified: bool,
    pub expires_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupType {
    FullKeys,
    SessionKeys,
    IdentityKeys,
    PrekeyBundle,
    SignedPrekeys,
    GroupKeys,
    DeviceState,
}

impl From<String> for BackupType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "full_keys" => BackupType::FullKeys,
            "session_keys" => BackupType::SessionKeys,
            "identity_keys" => BackupType::IdentityKeys,
            "prekey_bundle" => BackupType::PrekeyBundle,
            "signed_prekeys" => BackupType::SignedPrekeys,
            "group_keys" => BackupType::GroupKeys,
            "device_state" => BackupType::DeviceState,
            _ => BackupType::FullKeys,
        }
    }
}

impl From<BackupType> for String {
    fn from(backup_type: BackupType) -> Self {
        match backup_type {
            BackupType::FullKeys => "full_keys".to_string(),
            BackupType::SessionKeys => "session_keys".to_string(),
            BackupType::IdentityKeys => "identity_keys".to_string(),
            BackupType::PrekeyBundle => "prekey_bundle".to_string(),
            BackupType::SignedPrekeys => "signed_prekeys".to_string(),
            BackupType::GroupKeys => "group_keys".to_string(),
            BackupType::DeviceState => "device_state".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateEncryptedBackupKey {
    pub device_id: DieselUlid,
    pub encrypted_backup_data: String,
    pub backup_algorithm: String,
    pub backup_type: String,
    pub backup_hash: String,
    pub expires_in_days: Option<i32>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::encrypted_backup_keys)]
pub struct NewEncryptedBackupKey {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_id: DieselUlid,
    pub encrypted_backup_data: String,
    pub backup_algorithm: String,
    pub backup_type: String,
    pub backup_size_bytes: i64,
    pub backup_hash: String,
    pub is_verified: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EncryptedBackupKeyResponse {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_id: DieselUlid,
    pub backup_algorithm: String,
    pub backup_type: String,
    pub backup_size_bytes: i64,
    pub backup_hash: String,
    pub is_verified: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl EncryptedBackupKey {
    pub fn new(
        user_id: DieselUlid,
        device_id: DieselUlid,
        encrypted_backup_data: String,
        backup_algorithm: String,
        backup_type: BackupType,
        backup_hash: String,
        expires_in_days: Option<i32>,
    ) -> NewEncryptedBackupKey {
        let now = Utc::now();
        let expires_in_days = expires_in_days.unwrap_or(365); // Default 1 year
        let expires_at = now + chrono::Duration::days(expires_in_days as i64);
        let backup_size_bytes = encrypted_backup_data.len() as i64;

        NewEncryptedBackupKey {
            id: DieselUlid::new(),
            user_id,
            device_id,
            encrypted_backup_data,
            backup_algorithm,
            backup_type: backup_type.into(),
            backup_size_bytes,
            backup_hash,
            is_verified: false,
            expires_at,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> EncryptedBackupKeyResponse {
        EncryptedBackupKeyResponse {
            id: self.id,
            user_id: self.user_id,
            device_id: self.device_id,
            backup_algorithm: self.backup_algorithm.clone(),
            backup_type: self.backup_type.clone(),
            backup_size_bytes: self.backup_size_bytes,
            backup_hash: self.backup_hash.clone(),
            is_verified: self.is_verified,
            expires_at: self.expires_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn backup_type_enum(&self) -> BackupType {
        self.backup_type.clone().into()
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        self.is_verified && !self.is_expired()
    }

    pub fn verify(&mut self) -> bool {
        use sha2::{Sha256, Digest};

        // Verify backup hash against encrypted backup data
        let mut hasher = Sha256::new();
        hasher.update(&self.encrypted_backup_data);
        hasher.update(&self.created_at.timestamp().to_string());
        hasher.update(self.user_id.to_string().as_bytes());
        let computed_hash = format!("{:x}", hasher.finalize());

        if computed_hash == self.backup_hash {
            self.is_verified = true;
            self.updated_at = Utc::now();
            true
        } else {
            tracing::error!("Backup hash verification failed for user {}", self.user_id);
            false
        }
    }

    pub fn verify_hash(&self, expected_hash: &str) -> bool {
        self.backup_hash == expected_hash
    }

    pub fn extend_expiry(&mut self, additional_days: i32) {
        self.expires_at = self.expires_at + chrono::Duration::days(additional_days as i64);
        self.updated_at = Utc::now();
    }

    pub fn is_full_backup(&self) -> bool {
        matches!(self.backup_type_enum(), BackupType::FullKeys)
    }

    pub fn is_session_backup(&self) -> bool {
        matches!(self.backup_type_enum(), BackupType::SessionKeys)
    }

    pub fn get_size_mb(&self) -> f64 {
        self.backup_size_bytes as f64 / 1_048_576.0 // Convert bytes to MB
    }

    pub fn get_algorithm_family(&self) -> &'static str {
        match self.backup_algorithm.as_str() {
            algo if algo.starts_with("aes") => "aes",
            algo if algo.starts_with("chacha") => "chacha",
            algo if algo.starts_with("xchacha") => "xchacha",
            _ => "unknown",
        }
    }
}

impl HasId for EncryptedBackupKey {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for EncryptedBackupKey {
    fn table_name() -> &'static str {
        "encrypted_backup_keys"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "device_id",
            "backup_algorithm",
            "backup_type",
            "backup_size_bytes",
            "is_verified",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "device_id",
            "backup_type",
            "backup_size_bytes",
            "is_verified",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "device_id",
            "backup_algorithm",
            "backup_type",
            "backup_size_bytes",
            "backup_hash",
            "is_verified",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "user",
            "device",
        ]
    }
}

crate::impl_query_builder_service!(EncryptedBackupKey);