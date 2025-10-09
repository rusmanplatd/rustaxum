use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use super::DieselUlid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::mfa_biometric_credentials)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaBiometricCredential {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_id: Option<DieselUlid>,
    pub biometric_type: String,
    pub credential_id: String,
    pub public_key: String,
    pub platform: String,
    pub device_name: Option<String>,
    pub is_platform_authenticator: bool,
    pub counter: i64,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::mfa_biometric_credentials)]
pub struct NewMfaBiometricCredential {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub device_id: Option<DieselUlid>,
    pub biometric_type: String,
    pub credential_id: String,
    pub public_key: String,
    pub platform: String,
    pub device_name: Option<String>,
    pub is_platform_authenticator: bool,
    pub counter: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BiometricRegistrationRequest {
    pub biometric_type: String, // fingerprint, face, iris, voice
    pub platform: String, // ios, android, windows, macos, linux
    pub device_name: Option<String>,
    pub credential_id: String,
    pub public_key: String,
    pub authenticator_data: String,
    pub client_data_json: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BiometricAuthenticationRequest {
    pub user_id: String,
    pub credential_id: String,
    pub authenticator_data: String,
    pub client_data_json: String,
    pub signature: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BiometricCredentialResponse {
    pub id: String,
    pub biometric_type: String,
    pub platform: String,
    pub device_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

impl MfaBiometricCredential {
    pub fn new(
        user_id: DieselUlid,
        device_id: Option<DieselUlid>,
        biometric_type: String,
        credential_id: String,
        public_key: String,
        platform: String,
        device_name: Option<String>,
        is_platform_authenticator: bool,
    ) -> NewMfaBiometricCredential {
        let now = Utc::now();
        NewMfaBiometricCredential {
            id: DieselUlid::new(),
            user_id,
            device_id,
            biometric_type,
            credential_id,
            public_key,
            platform,
            device_name,
            is_platform_authenticator,
            counter: 0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> BiometricCredentialResponse {
        BiometricCredentialResponse {
            id: self.id.to_string(),
            biometric_type: self.biometric_type.clone(),
            platform: self.platform.clone(),
            device_name: self.device_name.clone(),
            created_at: self.created_at,
            last_used_at: self.last_used_at,
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}
