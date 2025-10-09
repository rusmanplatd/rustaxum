use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use super::DieselUlid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::mfa_webauthn_credentials)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaWebAuthnCredential {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub credential_id: String,
    pub public_key: String,
    pub counter: i64,
    pub device_name: Option<String>,
    pub aaguid: Option<String>,
    pub transports: Option<Vec<Option<String>>>,
    pub attestation_format: Option<String>,
    pub is_backup_eligible: bool,
    pub is_backup_state: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::mfa_webauthn_credentials)]
pub struct NewMfaWebAuthnCredential {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub credential_id: String,
    pub public_key: String,
    pub counter: i64,
    pub device_name: Option<String>,
    pub aaguid: Option<String>,
    pub transports: Option<Vec<Option<String>>>,
    pub attestation_format: Option<String>,
    pub is_backup_eligible: bool,
    pub is_backup_state: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::mfa_webauthn_challenges)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaWebAuthnChallenge {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub challenge: String,
    pub challenge_type: String,
    pub expires_at: DateTime<Utc>,
    pub is_used: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::mfa_webauthn_challenges)]
pub struct NewMfaWebAuthnChallenge {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub challenge: String,
    pub challenge_type: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WebAuthnRegistrationStartRequest {
    pub device_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WebAuthnRegistrationFinishRequest {
    pub credential_id: String,
    pub public_key: String,
    pub attestation_object: String,
    pub client_data_json: String,
    pub device_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WebAuthnAuthenticationStartRequest {
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WebAuthnAuthenticationFinishRequest {
    pub user_id: String,
    pub credential_id: String,
    pub authenticator_data: String,
    pub client_data_json: String,
    pub signature: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebAuthnCredentialResponse {
    pub id: String,
    pub device_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub transports: Option<Vec<String>>,
}

impl MfaWebAuthnCredential {
    pub fn new(
        user_id: DieselUlid,
        credential_id: String,
        public_key: String,
        device_name: Option<String>,
        aaguid: Option<String>,
        transports: Option<Vec<Option<String>>>,
        attestation_format: Option<String>,
        is_backup_eligible: bool,
        is_backup_state: bool,
    ) -> NewMfaWebAuthnCredential {
        let now = Utc::now();
        NewMfaWebAuthnCredential {
            id: DieselUlid::new(),
            user_id,
            credential_id,
            public_key,
            counter: 0,
            device_name,
            aaguid,
            transports,
            attestation_format,
            is_backup_eligible,
            is_backup_state,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> WebAuthnCredentialResponse {
        WebAuthnCredentialResponse {
            id: self.id.to_string(),
            device_name: self.device_name.clone(),
            created_at: self.created_at,
            last_used_at: self.last_used_at,
            transports: self.transports.as_ref().map(|t| {
                t.iter().filter_map(|s| s.clone()).collect()
            }),
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

impl MfaWebAuthnChallenge {
    pub fn new(
        user_id: DieselUlid,
        challenge: String,
        challenge_type: String,
        expires_in_minutes: i64,
    ) -> NewMfaWebAuthnChallenge {
        let now = Utc::now();
        NewMfaWebAuthnChallenge {
            id: DieselUlid::new(),
            user_id,
            challenge,
            challenge_type,
            expires_at: now + chrono::Duration::minutes(expires_in_minutes),
            created_at: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.is_used && !self.is_expired()
    }
}
