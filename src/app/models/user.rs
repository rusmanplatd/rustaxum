use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::{HasModelType, HasRoles, DieselUlid};
use crate::app::models::activity_log::HasId;

/// User model representing a registered user
/// Contains authentication, profile, and security information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, Insertable)]
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
    pub created_by_id: DieselUlid,
    /// User who last updated this record
    pub updated_by_id: DieselUlid,
    /// User who deleted this record
    pub deleted_by_id: Option<DieselUlid>,
    /// Identity public key for encryption
    pub identity_public_key: Option<String>,
    /// Identity key creation timestamp
    pub identity_key_created_at: Option<DateTime<Utc>>,
    /// MFA enabled flag
    pub mfa_enabled: bool,
    /// MFA secret for TOTP
    pub mfa_secret: Option<String>,
    /// MFA backup codes (hashed)
    pub mfa_backup_codes: Option<serde_json::Value>,
    /// MFA required flag
    pub mfa_required: bool,
    /// Email notifications preference
    pub email_notifications: Option<bool>,
    /// Database notifications preference
    pub database_notifications: Option<bool>,
    /// Broadcast notifications preference
    pub broadcast_notifications: Option<bool>,
    /// Web push notifications preference
    pub web_push_notifications: Option<bool>,
    /// SMS notifications preference
    pub sms_notifications: Option<bool>,
    /// Slack notifications preference
    pub slack_notifications: Option<bool>,
    /// Marketing emails preference
    pub marketing_emails: Option<bool>,
    /// Security alerts preference
    pub security_alerts: Option<bool>,
    /// Order updates preference
    pub order_updates: Option<bool>,
    /// Newsletter preference
    pub newsletter: Option<bool>,
    /// Promotional emails preference
    pub promotional_emails: Option<bool>,
    /// Account notifications preference
    pub account_notifications: Option<bool>,
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
    pub fn new(name: String, email: String, password: String, created_by: &str) -> Self {
        let now = Utc::now();
        let creator_id = DieselUlid::from_string(created_by.trim())
            .expect("Invalid created_by ULID provided to User::new()");
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
            created_by_id: creator_id.clone(),
            updated_by_id: creator_id,
            deleted_by_id: None,
            identity_public_key: None,
            identity_key_created_at: None,
            mfa_enabled: false,
            mfa_secret: None,
            mfa_backup_codes: None,
            mfa_required: false,
            email_notifications: Some(true),
            database_notifications: Some(true),
            broadcast_notifications: Some(true),
            web_push_notifications: Some(true),
            sms_notifications: Some(false),
            slack_notifications: Some(false),
            marketing_emails: Some(true),
            security_alerts: Some(true),
            order_updates: Some(true),
            newsletter: Some(false),
            promotional_emails: Some(false),
            account_notifications: Some(true),
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
        self.deleted_by_id = deleted_by;
    }

    pub fn restore(&mut self) {
        self.deleted_at = None;
        self.deleted_by_id = None;
    }

    pub fn to_new_user(name: String, email: String, password: String, created_by: Option<DieselUlid>) -> User {
        let now = Utc::now();
        let created_by_ulid = created_by.unwrap_or_else(|| DieselUlid::from_string("01SYSTEM0SEEDER00000000000").unwrap());
        User {
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
            created_by_id: created_by_ulid,
            updated_by_id: created_by_ulid,
            deleted_by_id: None,
            identity_public_key: None,
            identity_key_created_at: None,
            mfa_enabled: false,
            mfa_secret: None,
            mfa_backup_codes: None,
            mfa_required: false,
            email_notifications: Some(true),
            database_notifications: Some(true),
            broadcast_notifications: Some(true),
            web_push_notifications: Some(true),
            sms_notifications: Some(false),
            slack_notifications: Some(false),
            marketing_emails: Some(true),
            security_alerts: Some(true),
            order_updates: Some(true),
            newsletter: Some(false),
            promotional_emails: Some(false),
            account_notifications: Some(true),
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

impl HasId for User {
    fn id(&self) -> String {
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
            "roles.permissions",
            "permissions.roles",
            "roles.organization",
            "permissions.organization",
            "authorizationContext",
            "scopedRoles",
            "scopedPermissions",
            "organizations",
            "organizations.position",
            "organizations.position.level",
            "createdBy",
            "updatedBy",
            "deletedBy",
            "createdBy.organizations",
            "updatedBy.organizations",
            "deletedBy.organizations",
            "createdBy.organizations.position",
            "updatedBy.organizations.position",
            "deletedBy.organizations.position",
            "createdBy.organizations.position.level",
            "updatedBy.organizations.position.level",
            "deletedBy.organizations.position.level",
        ]
    }
}

// Implement the enhanced filtering trait
impl crate::app::query_builder::Filterable for User {
    fn apply_basic_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
        match operator {
            "=" => format!("{} = {}", column, Self::format_filter_value(value)),
            "!=" => format!("{} != {}", column, Self::format_filter_value(value)),
            _ => format!("{} {} {}", column, operator, Self::format_filter_value(value))
        }
    }
}

// Implement the enhanced sorting trait
impl crate::app::query_builder::Sortable for User {
    fn apply_basic_sort(column: &str, direction: &str) -> String {
        format!("{} {}", column, direction)
    }
}

// Implement the relationship inclusion trait
impl crate::app::query_builder::Includable for User {
    fn load_relationships(ids: &[String], includes: &[String], _conn: &mut diesel::pg::PgConnection) -> anyhow::Result<()> {

        for include in includes {
            match include.as_str() {
                "roles" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_roles("sys_users", ids, _conn)?;
                },
                "permissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_permissions("sys_users", ids, _conn)?;
                },
                "roles.permissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_roles_with_permissions("sys_users", ids, _conn)?;
                },
                "permissions.roles" => {
                    crate::app::query_builder::RolePermissionLoader::load_model_permissions_with_roles("sys_users", ids, _conn)?;
                },
                "roles.organization" => {
                    crate::app::query_builder::RolePermissionLoader::load_roles_with_organization("sys_users", ids, _conn)?;
                },
                "permissions.organization" => {
                    crate::app::query_builder::RolePermissionLoader::load_permissions_with_organization("sys_users", ids, _conn)?;
                },
                "authorizationContext" => {
                    crate::app::query_builder::RolePermissionLoader::load_complete_authorization_context("sys_users", ids, _conn)?;
                },
                "scopedRoles" => {
                    crate::app::query_builder::RolePermissionLoader::load_scoped_roles("sys_users", ids, _conn)?;
                },
                "scopedPermissions" => {
                    crate::app::query_builder::RolePermissionLoader::load_scoped_permissions("sys_users", ids, _conn)?;
                },
                "organizations" => {
                    tracing::debug!("Loading organizations for users: {:?}", ids);
                },
                "organizations.position" => {
                    tracing::debug!("Loading organizations.position for users: {:?}", ids);
                },
                "organizations.position.level" => {
                    tracing::debug!("Loading organizations.position.level for users: {:?}", ids);
                },
                _ => {
                    tracing::warn!("Unknown relationship: {}", include);
                }
            }
        }
        Ok(())
    }

    fn get_foreign_key(relationship: &str) -> Option<String> {
        match relationship {
            "roles" => Some("user_id".to_string()),
            "permissions" => Some("user_id".to_string()),
            "organizations" => Some("user_organizations.user_id".to_string()), // Load from pivot table
            _ => None
        }
    }

    fn build_join_clause(relationship: &str, main_table: &str) -> Option<String> {
        match relationship {
            "organizations" => {
                // Many-to-many relationship through user_organizations pivot table
                Some(format!(
                    "LEFT JOIN user_organizations ON {}.id = user_organizations.user_id LEFT JOIN organizations ON user_organizations.organization_id = organizations.id",
                    main_table
                ))
            },
            "organizations.position" => {
                // Include organization positions through user_organizations
                Some(format!(
                    "LEFT JOIN user_organizations ON {}.id = user_organizations.user_id LEFT JOIN organizations ON user_organizations.organization_id = organizations.id LEFT JOIN organization_positions ON user_organizations.organization_position_id = organization_positions.id",
                    main_table
                ))
            },
            "organizations.position.level" => {
                // Include organization positions and their levels
                Some(format!(
                    "LEFT JOIN user_organizations ON {}.id = user_organizations.user_id LEFT JOIN organizations ON user_organizations.organization_id = organizations.id LEFT JOIN organization_positions ON user_organizations.organization_position_id = organization_positions.id LEFT JOIN organization_position_levels ON organization_positions.organization_position_level_id = organization_position_levels.id",
                    main_table
                ))
            },
            "roles" => {
                // Many-to-many relationship through sys_model_has_roles pivot table
                Some(format!(
                    "LEFT JOIN sys_model_has_roles ON {}.id = sys_model_has_roles.model_id AND sys_model_has_roles.model_type = 'User' LEFT JOIN sys_roles ON sys_model_has_roles.role_id = sys_roles.id",
                    main_table
                ))
            },
            "permissions" => {
                // Many-to-many relationship through sys_model_has_permissions pivot table
                Some(format!(
                    "LEFT JOIN sys_model_has_permissions ON {}.id = sys_model_has_permissions.model_id AND sys_model_has_permissions.model_type = 'User' LEFT JOIN sys_permissions ON sys_model_has_permissions.permission_id = sys_permissions.id",
                    main_table
                ))
            },
            _ => None
        }
    }

    fn should_eager_load(relationship: &str) -> bool {
        // Organizations and roles are commonly accessed together
        matches!(relationship, "organizations" | "roles")
    }
}

// Implement the query builder service for User
crate::impl_query_builder_service!(User);