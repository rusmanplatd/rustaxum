use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::{HasModelType, HasRoles, DieselUlid};

/// User model representing a registered user
/// Contains authentication, profile, and security information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::sys_users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    /// Unique user identifier
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    /// User's full name
    #[schema(example = "John Doe")]
    pub name: String,
    /// User's email address
    #[schema(example = "john.doe@example.com")]
    pub email: String,
    /// Email verification timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub email_verified_at: Option<DateTime<Utc>>,
    /// Username (optional)
    pub username: Option<String>,
    /// Hashed password (never exposed in responses)
    #[schema(example = "$2b$12$...")]
    pub password: String,
    /// Remember me token for persistent sessions
    pub remember_token: Option<String>,
    /// Password reset token
    pub password_reset_token: Option<String>,
    /// Password reset token expiration
    pub password_reset_expires_at: Option<DateTime<Utc>>,
    /// JWT refresh token for authentication
    pub refresh_token: Option<String>,
    /// Refresh token expiration timestamp
    pub refresh_token_expires_at: Option<DateTime<Utc>>,
    /// User avatar URL
    pub avatar: Option<String>,
    /// User's birthdate
    pub birthdate: Option<chrono::NaiveDate>,
    /// Number of consecutive failed login attempts
    #[schema(example = 0)]
    pub failed_login_attempts: i32,
    /// Google OAuth ID
    pub google_id: Option<String>,
    /// Last successful login timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub last_login_at: Option<DateTime<Utc>>,
    /// Last seen timestamp
    pub last_seen_at: DateTime<Utc>,
    /// User's locale preference
    pub locale: Option<String>,
    /// Account lock expiration timestamp
    pub locked_until: Option<DateTime<Utc>>,
    /// User's phone number
    pub phone_number: Option<String>,
    /// Phone verification timestamp
    pub phone_verified_at: Option<DateTime<Utc>>,
    /// User's timezone
    pub zoneinfo: Option<String>,
    /// User creation timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
    /// Soft delete timestamp
    pub deleted_at: Option<DateTime<Utc>>,
    /// User who created this record
    pub created_by: Option<DieselUlid>,
    /// User who last updated this record
    pub updated_by: Option<DieselUlid>,
    /// User who deleted this record
    pub deleted_by: Option<DieselUlid>,
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

/// Insertable struct for creating new users in the database
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::sys_users)]
pub struct NewUser {
    pub id: DieselUlid,
    pub name: String,
    pub email: String,
    pub username: Option<String>,
    pub password: String,
    pub last_seen_at: DateTime<Utc>,
    pub failed_login_attempts: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<DieselUlid>,
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
    pub id: DieselUlid,
    pub name: String,
    pub email: String,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Get phone number (convenience method)
    pub fn phone(&self) -> Option<&String> {
        self.phone_number.as_ref()
    }
    pub fn new(name: String, email: String, password: String) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            name,
            email,
            email_verified_at: None,
            username: None,
            password,
            remember_token: None,
            password_reset_token: None,
            password_reset_expires_at: None,
            refresh_token: None,
            refresh_token_expires_at: None,
            avatar: None,
            birthdate: None,
            failed_login_attempts: 0,
            google_id: None,
            last_login_at: None,
            last_seen_at: now,
            locale: None,
            locked_until: None,
            phone_number: None,
            phone_verified_at: None,
            zoneinfo: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        }
    }

    pub fn to_response(&self) -> UserResponse {
        UserResponse {
            id: self.id,
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

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn delete(&mut self, deleted_by: Option<DieselUlid>) {
        self.deleted_at = Some(Utc::now());
        self.deleted_by = deleted_by;
    }

    pub fn restore(&mut self) {
        self.deleted_at = None;
        self.deleted_by = None;
    }

    pub fn to_new_user(name: String, email: String, password: String, created_by: Option<DieselUlid>) -> NewUser {
        let now = Utc::now();
        NewUser {
            id: DieselUlid::new(),
            name,
            email,
            username: None,
            password,
            last_seen_at: now,
            failed_login_attempts: 0,
            created_at: now,
            updated_at: now,
            created_by,
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


impl crate::app::query_builder::Queryable for User {
    fn table_name() -> &'static str {
        "sys_users"
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

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "roles",
            "permissions",
            "organization",
        ]
    }
}

// Implement the query builder service for User
crate::impl_query_builder_service!(User);