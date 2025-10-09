use anyhow::{Result, bail};
use rand::{Rng, thread_rng};
use sha2::{Sha256, Digest};
use crate::database::DbPool;
use crate::app::services::user_service::UserService;
use crate::app::models::mfa_sms::{MfaSmsCode, NewMfaSmsCode};
use crate::app::models::DieselUlid;
use diesel::prelude::*;

const SMS_CODE_LENGTH: usize = 6;
const SMS_CODE_EXPIRY_MINUTES: i64 = 5;
const MAX_SMS_CODES_PER_HOUR: i64 = 3;

pub struct MfaSmsService;

impl MfaSmsService {
    /// Generate and send OTP code via SMS
    pub async fn send_code(
        pool: &DbPool,
        user_id: String,
        phone_number: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<()> {
        // Check user exists
        let user = UserService::find_by_id(pool, user_id.clone())?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        // Validate phone number format (basic validation)
        if !Self::is_valid_phone_number(&phone_number) {
            bail!("Invalid phone number format");
        }

        // Rate limiting
        Self::check_rate_limit(pool, &user_id, &phone_number).await?;

        // Generate 6-digit code
        let code = Self::generate_code();
        let code_hash = Self::hash_code(&code);

        let user_id_ulid = DieselUlid::from_string(&user_id)?;

        // Invalidate existing unused codes
        Self::invalidate_existing_codes(pool, &user_id).await?;

        // Store the code
        let new_code = MfaSmsCode::new(
            user_id_ulid,
            phone_number.clone(),
            code.clone(),
            code_hash,
            SMS_CODE_EXPIRY_MINUTES,
            ip_address,
            user_agent,
        );

        let mut conn = pool.get()?;
        diesel::insert_into(crate::schema::mfa_sms_codes::table)
            .values(&new_code)
            .execute(&mut conn)?;

        // Send SMS (integration with SMS provider)
        Self::send_sms(&phone_number, &code).await?;

        tracing::info!(
            user_id = user_id,
            phone_number = phone_number,
            "SMS MFA code sent successfully"
        );

        Ok(())
    }

    /// Verify SMS OTP code
    pub async fn verify_code(
        pool: &DbPool,
        user_id: String,
        phone_number: String,
        code: &str,
    ) -> Result<bool> {
        use crate::schema::mfa_sms_codes::dsl::*;

        let user_id_ulid = DieselUlid::from_string(&user_id)?;
        let code_hash = Self::hash_code(code);

        let mut conn = pool.get()?;

        // Find the most recent unused code
        let sms_code = mfa_sms_codes
            .filter(user_id.eq(&user_id_ulid))
            .filter(phone_number.eq(&phone_number))
            .filter(is_used.eq(false))
            .filter(code_hash.eq(&code_hash))
            .order(created_at.desc())
            .select(MfaSmsCode::as_select())
            .first::<MfaSmsCode>(&mut conn)
            .optional()?;

        match sms_code {
            Some(sms_code) => {
                if sms_code.is_expired() {
                    bail!("SMS code has expired");
                }

                // Mark as used
                diesel::update(crate::schema::mfa_sms_codes::table.find(&sms_code.id))
                    .set((
                        is_used.eq(true),
                        verified_at.eq(Some(chrono::Utc::now())),
                    ))
                    .execute(&mut conn)?;

                tracing::info!(
                    user_id = user_id_ulid.to_string(),
                    "SMS MFA code verified successfully"
                );

                Ok(true)
            }
            None => {
                tracing::warn!(
                    user_id = user_id_ulid.to_string(),
                    "Invalid SMS MFA code attempted"
                );
                Ok(false)
            }
        }
    }

    /// Generate random numeric code
    fn generate_code() -> String {
        let mut rng = thread_rng();
        (0..SMS_CODE_LENGTH)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect()
    }

    /// Hash code for storage
    fn hash_code(code: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Send SMS via provider (Twilio, AWS SNS, etc.)
    async fn send_sms(phone_number: &str, code: &str) -> Result<()> {
        // TODO: Integrate with actual SMS provider
        // For now, just log it
        tracing::info!(
            phone_number = phone_number,
            "SMS would be sent: Your verification code is: {}. Valid for {} minutes.",
            code,
            SMS_CODE_EXPIRY_MINUTES
        );

        // Example Twilio integration:
        /*
        use reqwest::Client;
        let client = Client::new();
        let twilio_account_sid = std::env::var("TWILIO_ACCOUNT_SID")?;
        let twilio_auth_token = std::env::var("TWILIO_AUTH_TOKEN")?;
        let twilio_phone_number = std::env::var("TWILIO_PHONE_NUMBER")?;

        client
            .post(&format!(
                "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
                twilio_account_sid
            ))
            .basic_auth(&twilio_account_sid, Some(&twilio_auth_token))
            .form(&[
                ("To", phone_number),
                ("From", &twilio_phone_number),
                ("Body", &format!("Your verification code is: {}. Valid for {} minutes.", code, SMS_CODE_EXPIRY_MINUTES)),
            ])
            .send()
            .await?;
        */

        Ok(())
    }

    /// Validate phone number format
    fn is_valid_phone_number(phone: &str) -> bool {
        // Basic E.164 format validation
        let cleaned = phone.chars().filter(|c| c.is_numeric() || *c == '+').collect::<String>();
        cleaned.starts_with('+') && cleaned.len() >= 10 && cleaned.len() <= 15
    }

    /// Check rate limiting
    async fn check_rate_limit(pool: &DbPool, user_id_param: &str, phone: &str) -> Result<()> {
        use crate::schema::mfa_sms_codes::dsl::*;

        let user_id_ulid = DieselUlid::from_string(user_id_param)?;
        let one_hour_ago = chrono::Utc::now() - chrono::Duration::hours(1);

        let mut conn = pool.get()?;

        let count = mfa_sms_codes
            .filter(user_id.eq(&user_id_ulid))
            .filter(phone_number.eq(phone))
            .filter(created_at.gt(one_hour_ago))
            .count()
            .get_result::<i64>(&mut conn)?;

        if count >= MAX_SMS_CODES_PER_HOUR {
            bail!("Too many SMS codes requested. Please try again later.");
        }

        Ok(())
    }

    /// Invalidate existing unused codes
    async fn invalidate_existing_codes(pool: &DbPool, user_id_param: &str) -> Result<()> {
        use crate::schema::mfa_sms_codes::dsl::*;

        let user_id_ulid = DieselUlid::from_string(user_id_param)?;
        let mut conn = pool.get()?;

        diesel::update(mfa_sms_codes)
            .filter(user_id.eq(&user_id_ulid))
            .filter(is_used.eq(false))
            .set(is_used.eq(true))
            .execute(&mut conn)?;

        Ok(())
    }

    /// Clean up expired codes
    pub async fn cleanup_expired_codes(pool: &DbPool) -> Result<usize> {
        use crate::schema::mfa_sms_codes::dsl::*;

        let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
        let mut conn = pool.get()?;

        let deleted = diesel::delete(mfa_sms_codes)
            .filter(expires_at.lt(cutoff))
            .execute(&mut conn)?;

        tracing::info!(
            deleted_count = deleted,
            "Cleaned up expired SMS MFA codes"
        );

        Ok(deleted)
    }
}
