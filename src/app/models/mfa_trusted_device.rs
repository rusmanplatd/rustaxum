use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use super::DieselUlid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::mfa_trusted_devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaTrustedDevice {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_fingerprint: String,
    pub device_name: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub trust_token: String,
    pub expires_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::mfa_trusted_devices)]
pub struct NewMfaTrustedDevice {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_fingerprint: String,
    pub device_name: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub trust_token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TrustDeviceRequest {
    pub device_fingerprint: String,
    pub device_name: Option<String>,
    pub trust_duration_days: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TrustedDeviceResponse {
    pub id: String,
    pub device_name: Option<String>,
    pub ip_address: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl MfaTrustedDevice {
    pub fn new(
        user_id: DieselUlid,
        device_fingerprint: String,
        device_name: Option<String>,
        trust_token: String,
        expires_in_days: i64,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> NewMfaTrustedDevice {
        let now = Utc::now();
        NewMfaTrustedDevice {
            id: DieselUlid::new(),
            user_id,
            device_fingerprint,
            device_name,
            ip_address,
            user_agent,
            trust_token,
            expires_at: now + chrono::Duration::days(expires_in_days),
            created_at: now,
        }
    }

    pub fn to_response(&self) -> TrustedDeviceResponse {
        TrustedDeviceResponse {
            id: self.id.to_string(),
            device_name: self.device_name.clone(),
            ip_address: self.ip_address.clone(),
            last_used_at: self.last_used_at,
            expires_at: self.expires_at,
            created_at: self.created_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_revoked()
    }
}
