use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::models::user::User;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserResource {
    /// User unique identifier
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// User's full name
    #[schema(example = "John Doe")]
    pub name: String,
    /// User's email address
    #[schema(example = "john.doe@example.com")]
    pub email: String,
    /// Email verification timestamp
    pub email_verified_at: Option<DateTime<Utc>>,
    /// Username (optional)
    pub username: Option<String>,
    /// User avatar URL
    pub avatar: Option<String>,
    /// User's birthdate
    pub birthdate: Option<chrono::NaiveDate>,
    /// Last successful login timestamp
    pub last_login_at: Option<DateTime<Utc>>,
    /// Last seen timestamp
    pub last_seen_at: DateTime<Utc>,
    /// User's locale preference
    pub locale: Option<String>,
    /// User's phone number
    pub phone_number: Option<String>,
    /// Phone verification timestamp
    pub phone_verified_at: Option<DateTime<Utc>>,
    /// User's timezone
    pub zoneinfo: Option<String>,
    /// User creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserWithAuditResource {
    /// User unique identifier
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// User's full name
    #[schema(example = "John Doe")]
    pub name: String,
    /// User's email address
    #[schema(example = "john.doe@example.com")]
    pub email: String,
    /// Email verification timestamp
    pub email_verified_at: Option<DateTime<Utc>>,
    /// Username (optional)
    pub username: Option<String>,
    /// User avatar URL
    pub avatar: Option<String>,
    /// User's birthdate
    pub birthdate: Option<chrono::NaiveDate>,
    /// Last successful login timestamp
    pub last_login_at: Option<DateTime<Utc>>,
    /// Last seen timestamp
    pub last_seen_at: DateTime<Utc>,
    /// User's locale preference
    pub locale: Option<String>,
    /// User's phone number
    pub phone_number: Option<String>,
    /// Phone verification timestamp
    pub phone_verified_at: Option<DateTime<Utc>>,
    /// User's timezone
    pub zoneinfo: Option<String>,
    /// User creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Soft delete timestamp
    pub deleted_at: Option<DateTime<Utc>>,
    /// User who created this record
    pub created_by: Option<String>,
    /// User who last updated this record
    pub updated_by: Option<String>,
    /// User who deleted this record
    pub deleted_by: Option<String>,
}

impl UserResource {
    pub fn from_model(model: User) -> Self {
        Self {
            id: model.id.to_string(),
            name: model.name,
            email: model.email,
            email_verified_at: model.email_verified_at,
            username: model.username,
            avatar: model.avatar,
            birthdate: model.birthdate,
            last_login_at: model.last_login_at,
            last_seen_at: model.last_seen_at,
            locale: model.locale,
            phone_number: model.phone_number,
            phone_verified_at: model.phone_verified_at,
            zoneinfo: model.zoneinfo,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }

    pub fn collection(models: Vec<User>) -> Vec<Self> {
        models.into_iter().map(Self::from_model).collect()
    }
}

impl UserWithAuditResource {
    pub fn from_model(model: User) -> Self {
        Self {
            id: model.id.to_string(),
            name: model.name,
            email: model.email,
            email_verified_at: model.email_verified_at,
            username: model.username,
            avatar: model.avatar,
            birthdate: model.birthdate,
            last_login_at: model.last_login_at,
            last_seen_at: model.last_seen_at,
            locale: model.locale,
            phone_number: model.phone_number,
            phone_verified_at: model.phone_verified_at,
            zoneinfo: model.zoneinfo,
            created_at: model.created_at,
            updated_at: model.updated_at,
            deleted_at: model.deleted_at,
            created_by: model.created_by.map(|id| id.to_string()),
            updated_by: model.updated_by.map(|id| id.to_string()),
            deleted_by: model.deleted_by.map(|id| id.to_string()),
        }
    }

    pub fn collection(models: Vec<User>) -> Vec<Self> {
        models.into_iter().map(Self::from_model).collect()
    }
}

/// Basic role information for resource responses
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct UserRoleBasicInfo {
    /// Role unique identifier
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// Role name
    #[schema(example = "Administrator")]
    pub name: String,
    /// Role slug/code
    #[schema(example = "admin")]
    pub slug: String,
    /// Role description
    pub description: Option<String>,
    /// Source of assignment (direct or inherited)
    #[schema(example = "direct")]
    pub source: String,
}

/// Basic permission information for resource responses
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct UserPermissionBasicInfo {
    /// Permission unique identifier
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// Permission name
    #[schema(example = "users.create")]
    pub name: String,
    /// Permission slug/code
    #[schema(example = "users_create")]
    pub slug: String,
    /// Permission description
    pub description: Option<String>,
    /// Source of assignment (direct, role, or organization)
    #[schema(example = "role")]
    pub source: String,
    /// Source role ID if inherited from a role
    pub source_role_id: Option<String>,
}

/// Enhanced user resource with roles and permissions
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserResourceWithRolesAndPermissions {
    /// User unique identifier
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// User's full name
    #[schema(example = "John Doe")]
    pub name: String,
    /// User's email address
    #[schema(example = "john.doe@example.com")]
    pub email: String,
    /// Email verification timestamp
    pub email_verified_at: Option<DateTime<Utc>>,
    /// Username (optional)
    pub username: Option<String>,
    /// User avatar URL
    pub avatar: Option<String>,
    /// User's birthdate
    pub birthdate: Option<chrono::NaiveDate>,
    /// Last successful login timestamp
    pub last_login_at: Option<DateTime<Utc>>,
    /// Last seen timestamp
    pub last_seen_at: DateTime<Utc>,
    /// User's locale preference
    pub locale: Option<String>,
    /// User's phone number
    pub phone_number: Option<String>,
    /// Phone verification timestamp
    pub phone_verified_at: Option<DateTime<Utc>>,
    /// User's timezone
    pub zoneinfo: Option<String>,
    /// User creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Roles directly assigned to this user
    pub roles: Vec<UserRoleBasicInfo>,
    /// Permissions directly assigned to this user (direct and from roles)
    pub permissions: Vec<UserPermissionBasicInfo>,
}
