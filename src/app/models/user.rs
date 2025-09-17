use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Ulid,
    pub name: String,
    pub email: String,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub password: String,
    pub remember_token: Option<String>,
    pub password_reset_token: Option<String>,
    pub password_reset_expires_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub password: String,
    pub password_confirmation: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    pub password_confirmation: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(name: String, email: String, password: String) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            name,
            email,
            email_verified_at: None,
            password,
            remember_token: None,
            password_reset_token: None,
            password_reset_expires_at: None,
            last_login_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> UserResponse {
        UserResponse {
            id: self.id.to_string(),
            name: self.name.clone(),
            email: self.email.clone(),
            email_verified_at: self.email_verified_at,
            last_login_at: self.last_login_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            Utc::now() < locked_until
        } else {
            false
        }
    }

    pub fn is_password_reset_valid(&self, token: &str) -> bool {
        if let (Some(reset_token), Some(expires_at)) = (&self.password_reset_token, &self.password_reset_expires_at) {
            reset_token == token && Utc::now() < *expires_at
        } else {
            false
        }
    }
}