use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use super::DieselUlid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::mfa_push_devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaPushDevice {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_token: String,
    pub device_type: String,
    pub device_name: Option<String>,
    pub device_id: Option<String>,
    pub platform_version: Option<String>,
    pub app_version: Option<String>,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::mfa_push_devices)]
pub struct NewMfaPushDevice {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_token: String,
    pub device_type: String,
    pub device_name: Option<String>,
    pub device_id: Option<String>,
    pub platform_version: Option<String>,
    pub app_version: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::mfa_push_challenges)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaPushChallenge {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_id: Option<DieselUlid>,
    pub challenge: String,
    pub action_type: String,
    pub action_details: Option<serde_json::Value>,
    pub response: Option<String>,
    pub responded_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub location_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::mfa_push_challenges)]
pub struct NewMfaPushChallenge {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_id: Option<DieselUlid>,
    pub challenge: String,
    pub action_type: String,
    pub action_details: Option<serde_json::Value>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub location_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RegisterPushDeviceRequest {
    pub device_token: String,
    pub device_type: String, // ios, android, web
    pub device_name: Option<String>,
    pub device_id: Option<String>,
    pub platform_version: Option<String>,
    pub app_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SendPushChallengeRequest {
    pub action_type: String, // login, transaction, sensitive_action
    pub action_details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RespondPushChallengeRequest {
    pub challenge_id: String,
    pub response: String, // approved, denied
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PushDeviceResponse {
    pub id: String,
    pub device_type: String,
    pub device_name: Option<String>,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PushChallengeResponse {
    pub id: String,
    pub action_type: String,
    pub action_details: Option<serde_json::Value>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl MfaPushDevice {
    pub fn new(
        user_id: DieselUlid,
        device_token: String,
        device_type: String,
        device_name: Option<String>,
        device_id: Option<String>,
        platform_version: Option<String>,
        app_version: Option<String>,
    ) -> NewMfaPushDevice {
        let now = Utc::now();
        NewMfaPushDevice {
            id: DieselUlid::new(),
            user_id,
            device_token,
            device_type,
            device_name,
            device_id,
            platform_version,
            app_version,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> PushDeviceResponse {
        PushDeviceResponse {
            id: self.id.to_string(),
            device_type: self.device_type.clone(),
            device_name: self.device_name.clone(),
            is_active: self.is_active,
            last_used_at: self.last_used_at,
            created_at: self.created_at,
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

impl MfaPushChallenge {
    pub fn new(
        user_id: DieselUlid,
        device_id: Option<DieselUlid>,
        challenge: String,
        action_type: String,
        action_details: Option<serde_json::Value>,
        expires_in_minutes: i64,
        ip_address: Option<String>,
        location_data: Option<serde_json::Value>,
    ) -> NewMfaPushChallenge {
        let now = Utc::now();
        NewMfaPushChallenge {
            id: DieselUlid::new(),
            user_id,
            device_id,
            challenge,
            action_type,
            action_details,
            expires_at: now + chrono::Duration::minutes(expires_in_minutes),
            ip_address,
            location_data,
            created_at: now,
        }
    }

    pub fn to_response(&self) -> PushChallengeResponse {
        PushChallengeResponse {
            id: self.id.to_string(),
            action_type: self.action_type.clone(),
            action_details: self.action_details.clone(),
            expires_at: self.expires_at,
            created_at: self.created_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_pending(&self) -> bool {
        self.response.is_none() && !self.is_expired()
    }

    pub fn is_approved(&self) -> bool {
        self.response.as_ref().map(|r| r == "approved").unwrap_or(false)
    }
}
