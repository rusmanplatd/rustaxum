use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use totp_rs::{Algorithm, Secret, TOTP};
use qrcode::QrCode;
use base64::{Engine as _, engine::general_purpose};
use image::{ImageFormat, DynamicImage};
use rand::{Rng, thread_rng};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc, Duration};
use ulid::Ulid;
use crate::database::DbPool;
use crate::app::services::user_service::UserService;

const TOTP_ALGORITHM: Algorithm = Algorithm::SHA1;
const TOTP_DIGITS: usize = 6;
const TOTP_STEP: u64 = 30;
const BACKUP_CODES_COUNT: usize = 8;
const BACKUP_CODE_LENGTH: usize = 8;
const MAX_MFA_ATTEMPTS_PER_HOUR: i32 = 10;

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaVerifyRequest {
    pub user_id: String,
    pub code: String,
    pub method_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaMethodInfo {
    pub method_type: String,
    pub is_enabled: bool,
    pub is_verified: bool,
    pub last_used_at: Option<DateTime<Utc>>,
}

pub struct MfaService;

impl MfaService {
    /// Setup TOTP for a user
    pub async fn setup_totp(pool: &DbPool, user_id: String, app_name: &str) -> Result<MfaSetupResponse> {
        // Check if user exists
        let user = UserService::find_by_id(pool, user_id.clone())?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        // Generate TOTP secret
        let secret_bytes: Vec<u8> = (0..20).map(|_| rand::thread_rng().gen()).collect();
        let secret = Secret::Raw(secret_bytes);
        let secret_encoded = secret.to_encoded().to_string();
        let totp = TOTP::new(
            TOTP_ALGORITHM,
            TOTP_DIGITS,
            1,
            TOTP_STEP,
            secret.to_bytes().unwrap(),
        )?;

        // Generate QR code URL
        let qr_code_url = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}",
            app_name, user.email, secret_encoded, app_name
        );

        // Generate QR code image as base64
        let qr_code = QrCode::new(&qr_code_url)?;

        // Generate SVG format QR code
        use qrcode::render::svg;
        let svg_string = qr_code.render::<svg::Color>()
            .min_dimensions(200, 200)
            .dark_color(svg::Color("#000000"))
            .light_color(svg::Color("#FFFFFF"))
            .build();

        let qr_code_data_url = format!("data:image/svg+xml;base64,{}",
            general_purpose::STANDARD.encode(svg_string.as_bytes()));

        // Generate backup codes
        let backup_codes = Self::generate_backup_codes();
        let hashed_backup_codes = Self::hash_backup_codes(&backup_codes)?;

        // TODO: Store MFA method in database (this would use the MFA model once schema is ready)
        let mut conn = pool.get()?;

        use diesel::prelude::*;
        use crate::schema::sys_users;

        let backup_codes_json = serde_json::to_value(&hashed_backup_codes)?;

        diesel::update(sys_users::table.find(&user.id))
            .set((
                sys_users::mfa_secret.eq(Some(secret_encoded.clone())),
                sys_users::mfa_backup_codes.eq(Some(backup_codes_json)),
                sys_users::mfa_enabled.eq(false), // Not enabled until verified
            ))
            .execute(&mut conn)?;

        Ok(MfaSetupResponse {
            secret: secret_encoded.clone(),
            qr_code_url: qr_code_data_url,
            backup_codes,
        })
    }

    /// Verify TOTP code and enable MFA
    pub async fn verify_totp(pool: &DbPool, user_id: String, code: &str) -> Result<bool> {
        let user = UserService::find_by_id(pool, user_id.clone())?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let secret_encoded = user.mfa_secret
            .ok_or_else(|| anyhow::anyhow!("MFA not set up for this user"))?;

        // Check rate limiting
        Self::check_rate_limit(pool, &user_id, "totp").await?;

        let secret = Secret::Encoded(secret_encoded);
        let totp = TOTP::new(
            TOTP_ALGORITHM,
            TOTP_DIGITS,
            1,
            TOTP_STEP,
            secret.to_bytes().unwrap(),
        )?;

        let is_valid = totp.check_current(&code)?;

        // Log the attempt
        Self::log_mfa_attempt(pool, &user_id, "totp", is_valid, None, None).await?;

        if is_valid {
            // Enable MFA
            let mut conn = pool.get()?;
            use diesel::prelude::*;
            use crate::schema::sys_users;

            diesel::update(sys_users::table.find(&user.id))
                .set((
                    sys_users::mfa_enabled.eq(true),
                    sys_users::mfa_required.eq(false),
                ))
                .execute(&mut conn)?;
        }

        Ok(is_valid)
    }

    /// Verify MFA code during login
    pub async fn verify_mfa_code(pool: &DbPool, user_id: String, code: &str) -> Result<bool> {
        let user = UserService::find_by_id(pool, user_id.clone())?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        if !user.mfa_enabled {
            bail!("MFA is not enabled for this user");
        }

        // Check rate limiting
        Self::check_rate_limit(pool, &user_id, "login").await?;

        // Try TOTP first
        if let Some(secret_encoded) = &user.mfa_secret {
            let secret = Secret::Encoded(secret_encoded.clone());
            let totp = TOTP::new(
                TOTP_ALGORITHM,
                TOTP_DIGITS,
                1,
                TOTP_STEP,
                secret.to_bytes().unwrap(),
            )?;

            if totp.check_current(&code)? {
                Self::log_mfa_attempt(pool, &user_id, "totp", true, None, None).await?;
                return Ok(true);
            }
        }

        // Try backup codes
        if let Some(backup_codes_json) = &user.mfa_backup_codes {
            let backup_codes: Vec<String> = serde_json::from_value(backup_codes_json.clone())?;
            let code_hash = Self::hash_backup_code(code);

            if backup_codes.contains(&code_hash) {
                // Remove used backup code
                let mut updated_codes = backup_codes;
                updated_codes.retain(|c| c != &code_hash);

                let mut conn = pool.get()?;
                use diesel::prelude::*;
                use crate::schema::sys_users;

                diesel::update(sys_users::table.find(&user.id))
                    .set(sys_users::mfa_backup_codes.eq(Some(serde_json::to_value(&updated_codes)?)))
                    .execute(&mut conn)?;

                Self::log_mfa_attempt(pool, &user_id, "backup_code", true, None, None).await?;
                return Ok(true);
            }
        }

        Self::log_mfa_attempt(pool, &user_id, "login", false, None, None).await?;
        Ok(false)
    }

    /// Disable MFA for a user
    pub async fn disable_mfa(pool: &DbPool, user_id: String, current_password: &str) -> Result<()> {
        let user = UserService::find_by_id(pool, user_id.clone())?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        // Verify current password
        use crate::app::services::auth_service::AuthService;
        if !AuthService::verify_password(current_password, &user.password)? {
            bail!("Invalid password");
        }

        let mut conn = pool.get()?;
        use diesel::prelude::*;
        use crate::schema::sys_users;

        diesel::update(sys_users::table.find(&user.id))
            .set((
                sys_users::mfa_enabled.eq(false),
                sys_users::mfa_secret.eq(None::<String>),
                sys_users::mfa_backup_codes.eq(None::<serde_json::Value>),
                sys_users::mfa_required.eq(false),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    /// Generate new backup codes
    pub async fn regenerate_backup_codes(pool: &DbPool, user_id: String, current_password: &str) -> Result<Vec<String>> {
        let user = UserService::find_by_id(pool, user_id.clone())?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        if !user.mfa_enabled {
            bail!("MFA is not enabled for this user");
        }

        // Verify current password
        use crate::app::services::auth_service::AuthService;
        if !AuthService::verify_password(current_password, &user.password)? {
            bail!("Invalid password");
        }

        let backup_codes = Self::generate_backup_codes();
        let hashed_backup_codes = Self::hash_backup_codes(&backup_codes)?;

        let mut conn = pool.get()?;
        use diesel::prelude::*;
        use crate::schema::sys_users;

        diesel::update(sys_users::table.find(&user.id))
            .set(sys_users::mfa_backup_codes.eq(Some(serde_json::to_value(&hashed_backup_codes)?)))
            .execute(&mut conn)?;

        Ok(backup_codes)
    }

    /// Check if user has MFA enabled
    pub fn is_mfa_enabled(pool: &DbPool, user_id: String) -> Result<bool> {
        let user = UserService::find_by_id(pool, user_id)?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        Ok(user.mfa_enabled)
    }

    /// Check if MFA is required for the user
    pub fn is_mfa_required(pool: &DbPool, user_id: String) -> Result<bool> {
        let user = UserService::find_by_id(pool, user_id)?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        Ok(user.mfa_required)
    }

    /// Get MFA methods for a user
    pub fn get_mfa_methods(pool: &DbPool, user_id: String) -> Result<Vec<MfaMethodInfo>> {
        let user = UserService::find_by_id(pool, user_id)?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let mut methods = Vec::new();

        if user.mfa_secret.is_some() {
            methods.push(MfaMethodInfo {
                method_type: "totp".to_string(),
                is_enabled: user.mfa_enabled,
                is_verified: user.mfa_enabled,
                last_used_at: user.last_login_at,
            });
        }

        Ok(methods)
    }

    /// Generate random backup codes
    fn generate_backup_codes() -> Vec<String> {
        let mut rng = thread_rng();
        let mut codes = Vec::new();

        for _ in 0..BACKUP_CODES_COUNT {
            let code: String = (0..BACKUP_CODE_LENGTH)
                .map(|_| rng.gen_range(0..10).to_string())
                .collect();
            codes.push(code);
        }

        codes
    }

    /// Hash backup codes for storage
    fn hash_backup_codes(codes: &[String]) -> Result<Vec<String>> {
        codes.iter()
            .map(|code| Ok(Self::hash_backup_code(code)))
            .collect()
    }

    /// Hash a single backup code
    fn hash_backup_code(code: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Check rate limiting for MFA attempts
    async fn check_rate_limit(pool: &DbPool, user_id: &str, method_type: &str) -> Result<()> {
        // This would use the MFA attempts table once schema is ready
        // For now, implement a simple in-memory rate limiting

        // In a real implementation, you would:
        // 1. Query mfa_attempts table for recent attempts
        // 2. Check if attempts exceed the limit
        // 3. Return error if rate limited

        // Placeholder implementation
        Ok(())
    }

    /// Log MFA attempt
    async fn log_mfa_attempt(
        pool: &DbPool,
        user_id: &str,
        method_type: &str,
        success: bool,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<()> {
        // This would insert into mfa_attempts table once schema is ready
        // For now, we'll just log to tracing

        tracing::info!(
            user_id = user_id,
            method_type = method_type,
            success = success,
            ip_address = ip_address,
            user_agent = user_agent,
            "MFA attempt logged"
        );

        Ok(())
    }
}