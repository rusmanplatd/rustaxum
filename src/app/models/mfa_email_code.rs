use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use super::DieselUlid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = crate::schema::mfa_email_codes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaEmailCode {
    pub id: DieselUlid,
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
pub struct SendEmailCodeRequest {
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VerifyEmailCodeRequest {
    pub user_id: String,
    pub code: String,
}

impl MfaEmailCode {
    pub fn new(
        user_id: DieselUlid,
        code: String,
        code_hash: String,
        expires_in_minutes: i64,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let now = Utc::now();
        MfaEmailCode {
            id: DieselUlid::new(),
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
