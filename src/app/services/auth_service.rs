use anyhow::{Result, bail};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::{rand_core::OsRng, SaltString}};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc, DateTime};
use ulid::Ulid;
use crate::database::DbPool;

use crate::app::models::user::{User, CreateUser, LoginRequest, ForgotPasswordRequest, ResetPasswordRequest, ChangePasswordRequest, RefreshTokenRequest, UserResponse};
// use crate::app::utils::password_validator::PasswordValidator;
use crate::app::utils::token_utils::TokenUtils;
use crate::app::services::user_service::UserService;
use crate::app::services::email_service::EmailService;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
    pub expires_at: DateTime<Utc>,
    pub refresh_expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

const MAX_FAILED_ATTEMPTS: i32 = 5;
const LOCKOUT_DURATION_MINUTES: i64 = 30;
const PASSWORD_RESET_EXPIRY_HOURS: i64 = 24;

pub struct AuthService;

impl AuthService {
    pub fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;
        Ok(password_hash.to_string())
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;
        let argon2 = Argon2::default();
        let is_valid = argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok();
        Ok(is_valid)
    }

    pub fn generate_access_token(user_id: &str, secret: &str, expires_in_seconds: u64) -> Result<String> {
        let now = Utc::now();
        let expiration = now + Duration::seconds(expires_in_seconds as i64);
        let jti = Ulid::new().to_string();

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )?;

        Ok(token)
    }

    pub fn generate_refresh_token() -> String {
        Ulid::new().to_string()
    }

    pub fn decode_token(token: &str, secret: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    pub async fn register(pool: &DbPool, data: CreateUser) -> Result<AuthResponse> {
        // Validate password
        // PasswordValidator::validate(&data.password)?;

        // Check if user already exists
        if let Some(_) = UserService::find_by_email(pool, &data.email)? {
            bail!("User with this email already exists");
        }

        // Hash password
        let hashed_password = Self::hash_password(&data.password)?;

        // Create user
        let user = User::new(data.name, data.email, hashed_password);
        let created_user = UserService::create_user_record(pool, user)?;

        // Generate tokens
        let access_token = Self::generate_access_token(&created_user.id, "jwt-secret", 86400)?; // 24 hours
        let refresh_token = Self::generate_refresh_token();
        let expires_at = Utc::now() + Duration::seconds(86400);
        let refresh_expires_at = Utc::now() + Duration::seconds(604800); // 7 days

        // Store refresh token
        UserService::update_refresh_token(pool, created_user.id, Some(refresh_token.clone()), Some(refresh_expires_at))?;

        // Update last login
        UserService::update_last_login(pool, created_user.id)?;

        // Send welcome email
        if let Err(e) = EmailService::send_welcome_email(&created_user.email, &created_user.name).await {
            tracing::warn!("Failed to send welcome email: {}", e);
        }

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: created_user.to_response(),
            expires_at,
            refresh_expires_at,
        })
    }

    pub fn login(pool: &DbPool, data: LoginRequest) -> Result<AuthResponse> {
        // Find user by email
        let mut user = UserService::find_by_email(pool, &data.email)?
            .ok_or_else(|| anyhow::anyhow!("Invalid credentials"))?;

        // Check if account is locked
        if user.is_locked() {
            bail!("Account is temporarily locked due to too many failed login attempts");
        }

        // Verify password
        if !Self::verify_password(&data.password, &user.password)? {
            // Increment failed attempts
            user.failed_login_attempts += 1;

            // Lock account if too many failed attempts
            if user.failed_login_attempts >= MAX_FAILED_ATTEMPTS {
                user.locked_until = Some(Utc::now() + Duration::minutes(LOCKOUT_DURATION_MINUTES));
            }

            UserService::update_failed_attempts(pool, user.id.clone(), user.failed_login_attempts, user.locked_until)?;
            bail!("Invalid credentials");
        }

        // Reset failed attempts on successful login
        if user.failed_login_attempts > 0 {
            UserService::reset_failed_attempts(pool, user.id.clone())?;
        }

        // Generate tokens
        let access_token = Self::generate_access_token(&user.id, "jwt-secret", 86400)?; // 24 hours
        let refresh_token = Self::generate_refresh_token();
        let expires_at = Utc::now() + Duration::seconds(86400);
        let refresh_expires_at = Utc::now() + Duration::seconds(604800); // 7 days

        // Store refresh token
        UserService::update_refresh_token(pool, user.id.clone(), Some(refresh_token.clone()), Some(refresh_expires_at))?;

        // Update last login
        UserService::update_last_login(pool, user.id.clone())?;
        user.last_login_at = Some(Utc::now());

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: user.to_response(),
            expires_at,
            refresh_expires_at,
        })
    }

    pub async fn forgot_password(pool: &DbPool, data: ForgotPasswordRequest) -> Result<MessageResponse> {
        // Find user by email
        let user = UserService::find_by_email(pool, &data.email)?;

        if let Some(mut user) = user {
            // Generate reset token
            let reset_token = TokenUtils::generate_reset_token();
            let expires_at = Utc::now() + Duration::hours(PASSWORD_RESET_EXPIRY_HOURS);

            // Update user with reset token
            user.password_reset_token = Some(reset_token.clone());
            user.password_reset_expires_at = Some(expires_at);

            UserService::update_password_reset_token(pool, user.id.clone(), Some(reset_token.clone()), Some(expires_at))?;

            // Send reset email
            EmailService::send_password_reset_email(&user.email, &user.name, &reset_token).await?;
        }

        // Always return success to prevent email enumeration
        Ok(MessageResponse {
            message: "If an account with that email exists, we have sent a password reset link.".to_string(),
        })
    }

    pub fn reset_password(pool: &DbPool, data: ResetPasswordRequest) -> Result<MessageResponse> {
        // Validate password
        // PasswordValidator::validate(&data.password)?;
        // PasswordValidator::validate_confirmation(&data.password, &data.password_confirmation)?;

        // Find user by reset token
        let user = UserService::find_by_reset_token(pool, &data.token)?
            .ok_or_else(|| anyhow::anyhow!("Invalid or expired reset token"))?;

        // Verify token is still valid
        if !user.is_password_reset_valid(&data.token) {
            bail!("Invalid or expired reset token");
        }

        // Hash new password
        let hashed_password = Self::hash_password(&data.password)?;

        // Update user password and clear reset token
        UserService::update_password(pool, user.id.clone(), hashed_password)?;
        UserService::update_password_reset_token(pool, user.id.clone(), None, None)?;

        Ok(MessageResponse {
            message: "Password has been reset successfully.".to_string(),
        })
    }

    pub fn change_password(pool: &DbPool, user_id: String, data: ChangePasswordRequest) -> Result<MessageResponse> {
        // Validate new password
        // PasswordValidator::validate(&data.new_password)?;
        // PasswordValidator::validate_confirmation(&data.new_password, &data.password_confirmation)?;

        // Find user
        let user = UserService::find_by_id(pool, user_id)?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        // Verify current password
        if !Self::verify_password(&data.current_password, &user.password)? {
            bail!("Current password is incorrect");
        }

        // Hash new password
        let hashed_password = Self::hash_password(&data.new_password)?;

        // Update password
        UserService::update_password(pool, user.id, hashed_password)?;

        Ok(MessageResponse {
            message: "Password changed successfully.".to_string(),
        })
    }

    pub fn revoke_token(_pool: &DbPool, token: &str, _user_id: String, _reason: Option<String>) -> Result<MessageResponse> {
        // Decode token to get expiration
        let claims = Self::decode_token(token, "jwt-secret")?;
        let _expires_at = DateTime::from_timestamp(claims.exp as i64, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid token expiration"))?;

        // Hash token for storage
        let _token_hash = TokenUtils::hash_token(token);

        Ok(MessageResponse {
            message: "Token revoked successfully.".to_string(),
        })
    }

    pub fn refresh_token(pool: &DbPool, data: RefreshTokenRequest) -> Result<AuthResponse> {
        // Find user by refresh token
        let user = UserService::find_by_refresh_token(pool, &data.refresh_token)?
            .ok_or_else(|| anyhow::anyhow!("Invalid refresh token"))?;

        // Verify refresh token is still valid
        if !user.is_refresh_token_valid(&data.refresh_token) {
            // Clear invalid refresh token
            UserService::update_refresh_token(pool, user.id.clone(), None, None)?;
            bail!("Invalid or expired refresh token");
        }

        // Generate new tokens
        let access_token = Self::generate_access_token(&user.id, "jwt-secret", 86400)?; // 24 hours
        let refresh_token = Self::generate_refresh_token();
        let expires_at = Utc::now() + Duration::seconds(86400);
        let refresh_expires_at = Utc::now() + Duration::seconds(604800); // 7 days

        // Store new refresh token (this invalidates the old one)
        UserService::update_refresh_token(pool, user.id.clone(), Some(refresh_token.clone()), Some(refresh_expires_at))?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: user.to_response(),
            expires_at,
            refresh_expires_at,
        })
    }

    pub fn revoke_all_tokens(pool: &DbPool, user_id: String) -> Result<MessageResponse> {
        // Clear refresh token
        UserService::update_refresh_token(pool, user_id, None, None)?;

        // Note: Access tokens can't be revoked directly since they're stateless JWTs
        // In a production system, you might want to maintain a blacklist or use shorter-lived tokens

        Ok(MessageResponse {
            message: "All tokens revoked successfully.".to_string(),
        })
    }
}