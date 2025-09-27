use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::schema::devices;
use super::DieselUlid;
use chrono::{DateTime, Utc};
use crate::app::models::{HasModelType, activity_log::HasId};

fn default_interval() -> diesel::pg::data_types::PgInterval {
    diesel::pg::data_types::PgInterval::from_days(7)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustLevel {
    Trusted,
    Untrusted,
    Unverified,
}

impl From<String> for TrustLevel {
    fn from(s: String) -> Self {
        match s.as_str() {
            "trusted" => TrustLevel::Trusted,
            "untrusted" => TrustLevel::Untrusted,
            "unverified" => TrustLevel::Unverified,
            _ => TrustLevel::Unverified,
        }
    }
}

impl From<TrustLevel> for String {
    fn from(tl: TrustLevel) -> Self {
        match tl {
            TrustLevel::Trusted => "trusted".to_string(),
            TrustLevel::Untrusted => "untrusted".to_string(),
            TrustLevel::Unverified => "unverified".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = devices)]
#[diesel(primary_key(id))]
pub struct Device {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_name: String,
    pub device_type: String,
    pub identity_public_key: String,
    pub signed_prekey_public: String,
    pub signed_prekey_signature: String,
    pub signed_prekey_id: i32,
    pub supported_algorithms: Vec<Option<String>>,
    pub is_active: bool,
    pub last_seen_at: DateTime<Utc>,
    pub registration_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub signed_prekey_rotation_needed: bool,
    pub last_key_rotation_at: Option<DateTime<Utc>>,
    #[serde(skip, default = "default_interval")]
    pub prekey_rotation_interval: diesel::pg::data_types::PgInterval,
    pub trust_level: String, // Store as String, convert to/from enum as needed
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = devices)]
pub struct NewDevice {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_name: String,
    pub device_type: String,
    pub identity_public_key: String,
    pub signed_prekey_public: String,
    pub signed_prekey_signature: String,
    pub signed_prekey_id: i32,
    pub supported_algorithms: Option<Vec<Option<String>>>,
    pub is_active: Option<bool>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub registration_id: i32,
    pub signed_prekey_rotation_needed: Option<bool>,
    pub last_key_rotation_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub prekey_rotation_interval: Option<diesel::pg::data_types::PgInterval>,
    pub trust_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    Mobile,
    Desktop,
    Web,
    Tablet,
    Other,
}

impl From<String> for DeviceType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "mobile" => DeviceType::Mobile,
            "desktop" => DeviceType::Desktop,
            "web" => DeviceType::Web,
            "tablet" => DeviceType::Tablet,
            _ => DeviceType::Other,
        }
    }
}

impl From<DeviceType> for String {
    fn from(dt: DeviceType) -> Self {
        match dt {
            DeviceType::Mobile => "mobile".to_string(),
            DeviceType::Desktop => "desktop".to_string(),
            DeviceType::Web => "web".to_string(),
            DeviceType::Tablet => "tablet".to_string(),
            DeviceType::Other => "other".to_string(),
        }
    }
}

impl Device {
    pub fn device_type_enum(&self) -> DeviceType {
        self.device_type.clone().into()
    }

    pub fn trust_level_enum(&self) -> TrustLevel {
        self.trust_level.clone().into()
    }

    pub fn is_trusted(&self) -> bool {
        matches!(self.trust_level_enum(), TrustLevel::Trusted)
    }

    pub fn needs_key_rotation(&self) -> bool {
        self.signed_prekey_rotation_needed
    }
}

impl NewDevice {
    pub fn new(
        user_id: DieselUlid,
        device_name: String,
        device_type: DeviceType,
        identity_public_key: String,
        signed_prekey_public: String,
        signed_prekey_signature: String,
        signed_prekey_id: i32,
        registration_id: i32,
    ) -> Self {
        use diesel::pg::data_types::PgInterval;

        // Default 7-day rotation interval
        let interval = PgInterval::from_days(7);

        Self {
            id: DieselUlid::new(),
            user_id,
            device_name,
            device_type: device_type.into(),
            identity_public_key,
            signed_prekey_public,
            signed_prekey_signature,
            signed_prekey_id,
            supported_algorithms: Some(vec![
                Some("aes-256-gcm".to_string()),
                Some("chacha20-poly1305".to_string()),
                Some("curve25519".to_string()),
                Some("hmac-sha256".to_string()),
            ]),
            is_active: Some(true),
            last_seen_at: Some(Utc::now()),
            registration_id,
            signed_prekey_rotation_needed: Some(false),
            last_key_rotation_at: None,
            prekey_rotation_interval: Some(interval),
            trust_level: Some(TrustLevel::Unverified.into()),
        }
    }

    pub fn with_algorithms(mut self, algorithms: Vec<String>) -> Self {
        self.supported_algorithms = Some(algorithms.into_iter().map(Some).collect());
        self
    }

    pub fn with_trust_level(mut self, trust_level: TrustLevel) -> Self {
        self.trust_level = Some(trust_level.into());
        self
    }
}

impl HasModelType for Device {
    fn model_type() -> &'static str {
        "Device"
    }
}

impl HasId for Device {
    fn id(&self) -> String {
        self.id.to_string()
    }
}