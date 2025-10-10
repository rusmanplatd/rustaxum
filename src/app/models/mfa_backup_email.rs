use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use super::DieselUlid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::mfa_backup_emails)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaBackupEmail {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub backup_email: String,
    pub is_verified: bool,
    pub verification_token: Option<String>,
    pub verification_sent_at: Option<DateTime<Utc>>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::mfa_backup_email_codes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaBackupEmailCode {
    pub id: DieselUlid,
    pub backup_email_id: DieselUlid,
    pub user_id: DieselUlid,
    pub code: String,
    pub code_hash: String,
    pub expires_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub is_used: bool,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AddBackupEmailRequest {
    pub backup_email: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VerifyBackupEmailRequest {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SendBackupEmailCodeRequest {
    pub backup_email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VerifyBackupEmailCodeRequest {
    pub backup_email: String,
    pub code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BackupEmailResponse {
    pub id: String,
    pub backup_email: String,
    pub is_verified: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl MfaBackupEmail {
    pub fn new(
        user_id: DieselUlid,
        backup_email: String,
        verification_token: String,
    ) -> Self {
        let now = Utc::now();
        MfaBackupEmail {
            id: DieselUlid::new(),
            user_id,
            backup_email,
            is_verified: false,
            verified_at: None,
            verification_token: Some(verification_token),
            verification_sent_at: Some(now),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    pub fn to_response(&self) -> BackupEmailResponse {
        BackupEmailResponse {
            id: self.id.to_string(),
            backup_email: self.backup_email.clone(),
            is_verified: self.is_verified,
            verified_at: self.verified_at,
            created_at: self.created_at,
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

impl MfaBackupEmailCode {
    pub fn new(
        backup_email_id: DieselUlid,
        user_id: DieselUlid,
        code: String,
        code_hash: String,
        expires_in_minutes: i64,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let now = Utc::now();
        MfaBackupEmailCode {
            id: DieselUlid::new(),
            backup_email_id,
            user_id,
            code,
            code_hash,
            expires_at: now + chrono::Duration::minutes(expires_in_minutes),
            verified_at: None,
            is_used: false,
            ip_address,
            user_agent,
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
