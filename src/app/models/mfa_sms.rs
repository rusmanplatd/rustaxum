use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use super::DieselUlid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::mfa_sms_codes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaSmsCode {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub phone_number: String,
    pub code: String,
    pub code_hash: String,
    pub expires_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub is_used: bool,
    pub send_attempts: i32,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::mfa_sms_codes)]
pub struct NewMfaSmsCode {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub phone_number: String,
    pub code: String,
    pub code_hash: String,
    pub expires_at: DateTime<Utc>,
    pub send_attempts: i32,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SendSmsCodeRequest {
    pub phone_number: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VerifySmsCodeRequest {
    pub phone_number: String,
    pub code: String,
}

impl MfaSmsCode {
    pub fn new(
        user_id: DieselUlid,
        phone_number: String,
        code: String,
        code_hash: String,
        expires_in_minutes: i64,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> NewMfaSmsCode {
        let now = Utc::now();
        NewMfaSmsCode {
            id: DieselUlid::new(),
            user_id,
            phone_number,
            code,
            code_hash,
            expires_at: now + chrono::Duration::minutes(expires_in_minutes),
            send_attempts: 1,
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
