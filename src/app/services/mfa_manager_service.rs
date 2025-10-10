use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::database::DbPool;
use crate::app::services::user_service::UserService;
use crate::app::services::mfa_service::MfaService;
use crate::app::services::mfa_email_service::MfaEmailService;
use crate::app::services::mfa_sms_service::MfaSmsService;
use crate::app::models::DieselUlid;
use diesel::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaMethodStatus {
    pub method_type: String,
    pub is_enabled: bool,
    pub is_verified: bool,
    pub is_primary: bool,
    pub is_backup: bool,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaUserPreferences {
    pub primary_method: Option<String>,
    pub backup_method: Option<String>,
    pub trust_device_enabled: bool,
    pub trust_device_duration_days: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaChallengeRequest {
    pub user_id: String,
    pub method_type: String,
    pub action_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaVerificationRequest {
    pub user_id: String,
    pub method_type: String,
    pub code_or_token: String,
    pub device_fingerprint: Option<String>,
    pub trust_device: Option<bool>,
}

pub struct MfaManagerService;

impl MfaManagerService {
    /// Get all MFA methods available for a user
    pub async fn get_all_methods(pool: &DbPool, user_id_param: String) -> Result<Vec<MfaMethodStatus>> {
        use crate::schema::mfa_methods::dsl::*;
        use crate::schema::sys_users;

        let user_id_ulid = DieselUlid::from_string(&user_id_param)?;
        let mut conn = pool.get()?;

        // Get user preferences
        let user = sys_users::table
            .find(&user_id_ulid)
            .select((
                sys_users::mfa_primary_method,
                sys_users::mfa_backup_method,
            ))
            .first::<(Option<String>, Option<String>)>(&mut conn)
            .optional()?;

        let (primary_method, backup_method) = user.unwrap_or((None, None));

        // Get all enabled methods
        let methods = mfa_methods
            .filter(user_id.eq(&user_id_ulid))
            .filter(deleted_at.is_null())
            .select((method_type, is_enabled, is_verified, last_used_at, metadata))
            .load::<(String, bool, bool, Option<chrono::DateTime<chrono::Utc>>, Option<serde_json::Value>)>(&mut conn)?;

        let mut result = Vec::new();
        for (m_type, enabled, verified, last_used, meta) in methods {
            result.push(MfaMethodStatus {
                method_type: m_type.clone(),
                is_enabled: enabled,
                is_verified: verified,
                is_primary: primary_method.as_ref() == Some(&m_type),
                is_backup: backup_method.as_ref() == Some(&m_type),
                last_used_at: last_used,
                metadata: meta,
            });
        }

        Ok(result)
    }

    /// Get user's MFA preferences
    pub async fn get_preferences(pool: &DbPool, user_id: String) -> Result<MfaUserPreferences> {
        use crate::schema::sys_users;

        let user_id_ulid = DieselUlid::from_string(&user_id)?;
        let mut conn = pool.get()?;

        let prefs = sys_users::table
            .find(&user_id_ulid)
            .select((
                sys_users::mfa_primary_method,
                sys_users::mfa_backup_method,
                sys_users::mfa_trust_device_enabled,
                sys_users::mfa_trust_device_duration_days,
            ))
            .first::<(Option<String>, Option<String>, bool, Option<i32>)>(&mut conn)?;

        Ok(MfaUserPreferences {
            primary_method: prefs.0,
            backup_method: prefs.1,
            trust_device_enabled: prefs.2,
            trust_device_duration_days: prefs.3.unwrap_or(30),
        })
    }

    /// Update user's MFA preferences
    pub async fn update_preferences(
        pool: &DbPool,
        user_id: String,
        preferences: MfaUserPreferences,
    ) -> Result<()> {
        use crate::schema::sys_users;

        let user_id_ulid = DieselUlid::from_string(&user_id)?;
        let mut conn = pool.get()?;

        // Validate that the methods exist and are enabled
        if let Some(ref primary) = preferences.primary_method {
            Self::validate_method_enabled(pool, &user_id, primary).await?;
        }

        if let Some(ref backup) = preferences.backup_method {
            Self::validate_method_enabled(pool, &user_id, backup).await?;
        }

        diesel::update(sys_users::table.find(&user_id_ulid))
            .set((
                sys_users::mfa_primary_method.eq(preferences.primary_method),
                sys_users::mfa_backup_method.eq(preferences.backup_method),
                sys_users::mfa_trust_device_enabled.eq(preferences.trust_device_enabled),
                sys_users::mfa_trust_device_duration_days.eq(preferences.trust_device_duration_days),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    /// Send MFA challenge based on method type
    pub async fn send_challenge(
        pool: &DbPool,
        request: MfaChallengeRequest,
    ) -> Result<serde_json::Value> {
        match request.method_type.as_str() {
            "email" => {
                MfaEmailService::send_code(pool, request.user_id.clone(), None, None).await?;
                Ok(json!({
                    "method": "email",
                    "message": "Code sent to your email"
                }))
            }
            "sms" => {
                // Get user's phone number
                let user = UserService::find_by_id(pool, request.user_id.clone())?
                    .ok_or_else(|| anyhow::anyhow!("User not found"))?;

                let phone = user.phone_number
                    .ok_or_else(|| anyhow::anyhow!("No phone number on file"))?;

                MfaSmsService::send_code(pool, request.user_id.clone(), phone, None, None).await?;
                Ok(json!({
                    "method": "sms",
                    "message": "Code sent to your phone"
                }))
            }
            "totp" => {
                Ok(json!({
                    "method": "totp",
                    "message": "Enter code from your authenticator app"
                }))
            }
            "webauthn" | "biometric" => {
                // WebAuthn and biometric require client-side ceremony
                Ok(json!({
                    "method": request.method_type,
                    "message": "Use your security key or biometric",
                    "requires_client_ceremony": true
                }))
            }
            "push" => {
                Ok(json!({
                    "method": "push",
                    "message": "Check your mobile device for approval",
                    "requires_mobile_app": true
                }))
            }
            _ => bail!("Unsupported MFA method: {}", request.method_type),
        }
    }

    /// Verify MFA code/token for any method
    pub async fn verify_challenge(
        pool: &DbPool,
        request: MfaVerificationRequest,
    ) -> Result<bool> {
        let is_valid = match request.method_type.as_str() {
            "totp" => {
                MfaService::verify_mfa_code(pool, request.user_id.clone(), &request.code_or_token).await?
            }
            "email" => {
                MfaEmailService::verify_code(pool, request.user_id.clone(), &request.code_or_token).await?
            }
            "sms" => {
                let user = UserService::find_by_id(pool, request.user_id.clone())?
                    .ok_or_else(|| anyhow::anyhow!("User not found"))?;
                let phone = user.phone_number.ok_or_else(|| anyhow::anyhow!("No phone number"))?;

                MfaSmsService::verify_code(pool, request.user_id.clone(), phone, &request.code_or_token).await?
            }
            _ => bail!("Unsupported verification method: {}", request.method_type),
        };

        // If verification successful and user wants to trust device
        if is_valid && request.trust_device.unwrap_or(false) {
            if let Some(fingerprint) = request.device_fingerprint {
                Self::trust_device(pool, request.user_id.clone(), fingerprint, None, None, None).await?;
            }
        }

        Ok(is_valid)
    }

    /// Check if device is trusted (skip MFA)
    pub async fn is_device_trusted(
        pool: &DbPool,
        user_id_param: String,
        device_fingerprint_param: String,
    ) -> Result<bool> {
        use crate::schema::mfa_trusted_devices::dsl::*;

        let user_id_ulid = DieselUlid::from_string(&user_id_param)?;
        let mut conn = pool.get()?;
        let now = chrono::Utc::now();

        let count = mfa_trusted_devices
            .filter(user_id.eq(&user_id_ulid))
            .filter(device_fingerprint.eq(&device_fingerprint_param))
            .filter(expires_at.gt(now))
            .filter(revoked_at.is_null())
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(count > 0)
    }

    /// Trust a device
    async fn trust_device(
        pool: &DbPool,
        user_id_param: String,
        fingerprint: String,
        device_name: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<()> {
        use crate::schema::mfa_trusted_devices;
        use crate::app::models::mfa_trusted_device::MfaTrustedDevice;

        let user_id_ulid = DieselUlid::from_string(&user_id_param)?;

        // Get trust duration from user preferences
        let prefs = Self::get_preferences(pool, user_id_param.clone()).await?;

        // Generate trust token
        let trust_token = uuid::Uuid::new_v4().to_string();

        let now = chrono::Utc::now();
        let new_device = MfaTrustedDevice {
            id: DieselUlid::new(),
            user_id: user_id_ulid,
            device_fingerprint: fingerprint,
            device_name,
            ip_address,
            user_agent,
            trust_token,
            expires_at: now + chrono::Duration::days(prefs.trust_device_duration_days as i64),
            last_used_at: None,
            created_at: now,
            revoked_at: None,
        };

        let mut conn = pool.get()?;
        diesel::insert_into(mfa_trusted_devices::table)
            .values(&new_device)
            .execute(&mut conn)?;

        Ok(())
    }

    /// Revoke trusted device
    pub async fn revoke_trusted_device(pool: &DbPool, user_id_param: String, device_id_param: String) -> Result<()> {
        use crate::schema::mfa_trusted_devices::dsl::*;

        let user_id_ulid = DieselUlid::from_string(&user_id_param)?;
        let device_ulid = DieselUlid::from_string(&device_id_param)?;
        let mut conn = pool.get()?;

        diesel::update(mfa_trusted_devices)
            .filter(id.eq(&device_ulid))
            .filter(user_id.eq(&user_id_ulid))
            .set(revoked_at.eq(Some(chrono::Utc::now())))
            .execute(&mut conn)?;

        Ok(())
    }

    /// Get recommended MFA method for user
    pub async fn get_recommended_method(pool: &DbPool, user_id: String) -> Result<Option<String>> {
        let methods = Self::get_all_methods(pool, user_id.clone()).await?;
        let prefs = Self::get_preferences(pool, user_id).await?;

        // Return primary method if set and enabled
        if let Some(primary) = prefs.primary_method {
            if methods.iter().any(|m| m.method_type == primary && m.is_enabled) {
                return Ok(Some(primary));
            }
        }

        // Otherwise return first enabled method in order of preference
        let preference_order = vec!["webauthn", "biometric", "push", "totp", "email", "sms"];

        for method_type in preference_order {
            if methods.iter().any(|m| m.method_type == method_type && m.is_enabled) {
                return Ok(Some(method_type.to_string()));
            }
        }

        Ok(None)
    }

    /// Validate that a method is enabled for user
    async fn validate_method_enabled(pool: &DbPool, user_id_param: &str, method_type_param: &str) -> Result<()> {
        use crate::schema::mfa_methods::dsl::*;

        let user_id_ulid = DieselUlid::from_string(user_id_param)?;
        let mut conn = pool.get()?;

        let count = mfa_methods
            .filter(user_id.eq(&user_id_ulid))
            .filter(method_type.eq(method_type_param))
            .filter(is_enabled.eq(true))
            .filter(is_verified.eq(true))
            .count()
            .get_result::<i64>(&mut conn)?;

        if count == 0 {
            bail!("MFA method '{}' is not enabled for this user", method_type_param);
        }

        Ok(())
    }

    /// Audit log for MFA actions
    pub async fn log_mfa_action(
        pool: &DbPool,
        user_id: String,
        method_type: String,
        action: String,
        status: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        use crate::schema::mfa_audit_log;

        let user_id_ulid = DieselUlid::from_string(&user_id)?;
        let mut conn = pool.get()?;

        diesel::insert_into(mfa_audit_log::table)
            .values((
                mfa_audit_log::id.eq(DieselUlid::new()),
                mfa_audit_log::user_id.eq(&user_id_ulid),
                mfa_audit_log::method_type.eq(&method_type),
                mfa_audit_log::action.eq(&action),
                mfa_audit_log::status.eq(&status),
                mfa_audit_log::ip_address.eq(ip_address),
                mfa_audit_log::user_agent.eq(user_agent),
                mfa_audit_log::metadata.eq(metadata),
                mfa_audit_log::created_at.eq(chrono::Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }
}
