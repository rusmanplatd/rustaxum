use anyhow::{Result, bail};
use rand::{Rng, thread_rng};
use sha2::{Sha256, Digest};
use crate::database::DbPool;
use crate::app::services::user_service::UserService;
use crate::app::services::email_service::EmailService;
use crate::app::models::mfa_email_code::{MfaEmailCode, NewMfaEmailCode};
use crate::app::models::DieselUlid;
use diesel::prelude::*;

const EMAIL_CODE_LENGTH: usize = 6;
const EMAIL_CODE_EXPIRY_MINUTES: i64 = 10;
const MAX_EMAIL_CODES_PER_HOUR: i64 = 5;

pub struct MfaEmailService;

impl MfaEmailService {
    /// Generate and send OTP code via email
    pub async fn send_code(
        pool: &DbPool,
        user_id: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<()> {
        // Check user exists
        let user = UserService::find_by_id(pool, user_id.clone())?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        // Rate limiting: check how many codes sent in last hour
        Self::check_rate_limit(pool, &user_id).await?;

        // Generate 6-digit code
        let code = Self::generate_code();
        let code_hash = Self::hash_code(&code);

        let user_id_ulid = DieselUlid::from_string(&user_id)?;

        // Invalidate any existing unused codes for this user
        Self::invalidate_existing_codes(pool, &user_id).await?;

        // Store the code
        let new_code = MfaEmailCode::new(
            user_id_ulid,
            code.clone(),
            code_hash,
            EMAIL_CODE_EXPIRY_MINUTES,
            ip_address,
            user_agent,
        );

        let mut conn = pool.get()?;
        diesel::insert_into(crate::schema::mfa_email_codes::table)
            .values(&new_code)
            .execute(&mut conn)?;

        // Send email with code
        Self::send_email(&user.email, &user.name, &code).await?;

        tracing::info!(
            user_id = user_id,
            "Email MFA code sent successfully"
        );

        Ok(())
    }

    /// Verify email OTP code
    pub async fn verify_code(
        pool: &DbPool,
        user_id: String,
        code: &str,
    ) -> Result<bool> {
        use crate::schema::mfa_email_codes::dsl::*;

        let user_id_ulid = DieselUlid::from_string(&user_id)?;
        let code_hash = Self::hash_code(code);

        let mut conn = pool.get()?;

        // Find the most recent unused code for this user
        let email_code = mfa_email_codes
            .filter(user_id.eq(&user_id_ulid))
            .filter(is_used.eq(false))
            .filter(code_hash.eq(&code_hash))
            .order(created_at.desc())
            .select(MfaEmailCode::as_select())
            .first::<MfaEmailCode>(&mut conn)
            .optional()?;

        match email_code {
            Some(mut email_code) => {
                // Check if expired
                if email_code.is_expired() {
                    bail!("Email code has expired");
                }

                // Mark as used
                diesel::update(crate::schema::mfa_email_codes::table.find(&email_code.id))
                    .set((
                        is_used.eq(true),
                        verified_at.eq(Some(chrono::Utc::now())),
                    ))
                    .execute(&mut conn)?;

                tracing::info!(
                    user_id = user_id_ulid.to_string(),
                    "Email MFA code verified successfully"
                );

                Ok(true)
            }
            None => {
                tracing::warn!(
                    user_id = user_id_ulid.to_string(),
                    "Invalid email MFA code attempted"
                );
                Ok(false)
            }
        }
    }

    /// Check if email MFA is enabled for user
    pub fn is_enabled(pool: &DbPool, user_id: String) -> Result<bool> {
        use crate::schema::mfa_methods::dsl::*;

        let user_id_ulid = DieselUlid::from_string(&user_id)?;
        let mut conn = pool.get()?;

        let count = mfa_methods
            .filter(user_id.eq(&user_id_ulid))
            .filter(method_type.eq("email"))
            .filter(is_enabled.eq(true))
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(count > 0)
    }

    /// Generate random numeric code
    fn generate_code() -> String {
        let mut rng = thread_rng();
        (0..EMAIL_CODE_LENGTH)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect()
    }

    /// Hash code for storage
    fn hash_code(code: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Send email with OTP code
    async fn send_email(to_email: &str, to_name: &str, code: &str) -> Result<()> {
        let subject = "Your Multi-Factor Authentication Code";
        let body = format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; padding: 20px;">
                <h2>Your MFA Code</h2>
                <p>Hello {},</p>
                <p>Your multi-factor authentication code is:</p>
                <h1 style="background: #f0f0f0; padding: 20px; text-align: center; font-size: 32px; letter-spacing: 10px;">{}</h1>
                <p>This code will expire in {} minutes.</p>
                <p>If you didn't request this code, please ignore this email.</p>
                <p>For security reasons, never share this code with anyone.</p>
                <br>
                <p>Best regards,<br>The Security Team</p>
            </body>
            </html>
            "#,
            to_name,
            code,
            EMAIL_CODE_EXPIRY_MINUTES
        );

        EmailService::send_html_email(to_email, subject, &body).await?;

        Ok(())
    }

    /// Check rate limiting for sending codes
    async fn check_rate_limit(pool: &DbPool, user_id_param: &str) -> Result<()> {
        use crate::schema::mfa_email_codes::dsl::*;

        let user_id_ulid = DieselUlid::from_string(user_id_param)?;
        let one_hour_ago = chrono::Utc::now() - chrono::Duration::hours(1);

        let mut conn = pool.get()?;

        let count = mfa_email_codes
            .filter(user_id.eq(&user_id_ulid))
            .filter(created_at.gt(one_hour_ago))
            .count()
            .get_result::<i64>(&mut conn)?;

        if count >= MAX_EMAIL_CODES_PER_HOUR {
            bail!("Too many email codes requested. Please try again later.");
        }

        Ok(())
    }

    /// Invalidate existing unused codes for user
    async fn invalidate_existing_codes(pool: &DbPool, user_id_param: &str) -> Result<()> {
        use crate::schema::mfa_email_codes::dsl::*;

        let user_id_ulid = DieselUlid::from_string(user_id_param)?;
        let mut conn = pool.get()?;

        diesel::update(mfa_email_codes)
            .filter(user_id.eq(&user_id_ulid))
            .filter(is_used.eq(false))
            .set(is_used.eq(true))
            .execute(&mut conn)?;

        Ok(())
    }

    /// Clean up expired codes (should be run periodically)
    pub async fn cleanup_expired_codes(pool: &DbPool) -> Result<usize> {
        use crate::schema::mfa_email_codes::dsl::*;

        let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
        let mut conn = pool.get()?;

        let deleted = diesel::delete(mfa_email_codes)
            .filter(expires_at.lt(cutoff))
            .execute(&mut conn)?;

        tracing::info!(
            deleted_count = deleted,
            "Cleaned up expired email MFA codes"
        );

        Ok(deleted)
    }
}
