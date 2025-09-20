use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expires_in: u64,
    pub refresh_token_expires_in: u64,
    pub password_reset_expiry_hours: u64,
    pub max_failed_attempts: u32,
    pub lockout_duration_minutes: u64,
    pub password_min_length: usize,
    pub require_email_verification: bool,
}

impl AuthConfig {
    pub fn from_env() -> Result<Self> {
        Ok(AuthConfig {
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key-here-change-this-in-production".to_string()),
            jwt_expires_in: env::var("JWT_EXPIRES_IN")
                .unwrap_or_else(|_| "86400".to_string())
                .parse()
                .unwrap_or(86400),
            refresh_token_expires_in: env::var("REFRESH_TOKEN_EXPIRES_IN")
                .unwrap_or_else(|_| "604800".to_string())
                .parse()
                .unwrap_or(604800),
            password_reset_expiry_hours: env::var("PASSWORD_RESET_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            max_failed_attempts: env::var("MAX_FAILED_ATTEMPTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            lockout_duration_minutes: env::var("LOCKOUT_DURATION_MINUTES")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            password_min_length: env::var("PASSWORD_MIN_LENGTH")
                .unwrap_or_else(|_| "8".to_string())
                .parse()
                .unwrap_or(8),
            require_email_verification: env::var("REQUIRE_EMAIL_VERIFICATION")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        })
    }

    pub fn jwt_expires_in_chrono(&self) -> chrono::Duration {
        chrono::Duration::seconds(self.jwt_expires_in as i64)
    }

    pub fn refresh_token_expires_in_chrono(&self) -> chrono::Duration {
        chrono::Duration::seconds(self.refresh_token_expires_in as i64)
    }

    pub fn password_reset_expiry_chrono(&self) -> chrono::Duration {
        chrono::Duration::hours(self.password_reset_expiry_hours as i64)
    }

    pub fn lockout_duration_chrono(&self) -> chrono::Duration {
        chrono::Duration::minutes(self.lockout_duration_minutes as i64)
    }
}