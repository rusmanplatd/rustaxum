use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::query_builder::{Queryable, SortDirection};
use super::{HasModelType, HasRoles};

/// User model representing a registered user
/// Contains authentication, profile, and security information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    /// Unique user identifier
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: Ulid,
    /// User's full name
    #[schema(example = "John Doe")]
    pub name: String,
    /// User's email address
    #[schema(example = "john.doe@example.com")]
    pub email: String,
    /// Email verification timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub email_verified_at: Option<DateTime<Utc>>,
    /// Hashed password (never exposed in responses)
    #[schema(example = "$2b$12$...")]
    pub password: String,
    /// Remember me token for persistent sessions
    pub remember_token: Option<String>,
    /// JWT refresh token for authentication
    pub refresh_token: Option<String>,
    /// Refresh token expiration timestamp
    pub refresh_token_expires_at: Option<DateTime<Utc>>,
    /// Password reset token
    pub password_reset_token: Option<String>,
    /// Password reset token expiration
    pub password_reset_expires_at: Option<DateTime<Utc>>,
    /// Last successful login timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub last_login_at: Option<DateTime<Utc>>,
    /// Number of consecutive failed login attempts
    #[schema(example = 0)]
    pub failed_login_attempts: i32,
    /// Account lock expiration timestamp
    pub locked_until: Option<DateTime<Utc>>,
    /// User creation timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// Create user payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

/// Update user payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// Login request payload
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    /// User's email address
    #[schema(example = "john.doe@example.com")]
    pub email: String,
    /// User's password
    #[schema(example = "password123")]
    pub password: String,
}

/// Forgot password request payload
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

/// Reset password request payload
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub password: String,
    pub password_confirmation: String,
}

/// Change password request payload
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    pub password_confirmation: String,
}

/// Refresh token request payload
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// User response payload for API endpoints (excludes sensitive fields)
#[derive(Debug, Serialize, ToSchema)]
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
            refresh_token: None,
            refresh_token_expires_at: None,
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

    pub fn is_refresh_token_valid(&self, token: &str) -> bool {
        if let (Some(refresh_token), Some(expires_at)) = (&self.refresh_token, &self.refresh_token_expires_at) {
            refresh_token == token && Utc::now() < *expires_at
        } else {
            false
        }
    }
}

impl HasModelType for User {
    fn model_type() -> &'static str {
        "User"
    }
}

impl HasRoles for User {
    fn model_id(&self) -> String {
        self.id.to_string()
    }
}

impl FromRow<'_, PgRow> for User {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        Ok(User {
            id,
            name: row.try_get("name")?,
            email: row.try_get("email")?,
            email_verified_at: row.try_get("email_verified_at")?,
            password: row.try_get("password")?,
            remember_token: row.try_get("remember_token")?,
            refresh_token: row.try_get("refresh_token")?,
            refresh_token_expires_at: row.try_get("refresh_token_expires_at")?,
            password_reset_token: row.try_get("password_reset_token")?,
            password_reset_expires_at: row.try_get("password_reset_expires_at")?,
            last_login_at: row.try_get("last_login_at")?,
            failed_login_attempts: row.try_get("failed_login_attempts")?,
            locked_until: row.try_get("locked_until")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl Queryable for User {
    fn table_name() -> &'static str {
        "users"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "email",
            "email_verified_at",
            "last_login_at",
            "failed_login_attempts",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "email",
            "email_verified_at",
            "last_login_at",
            "failed_login_attempts",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "email",
            "email_verified_at",
            "last_login_at",
            "failed_login_attempts",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }
}