use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use crate::database::DbPool;
use ulid::Ulid;
use std::collections::HashSet;
use crate::app::services::sys_model_has_role_service::SysModelHasRoleService;
use crate::app::services::sys_model_has_permission_service::SysModelHasPermissionService;
use crate::app::models::user::User;
use crate::app::models::HasModelType;
use crate::app::services::auth_service::AuthService;
use crate::app::utils::token_utils::TokenUtils;

#[derive(Clone)]
pub struct RoleRequirement {
    pub roles: Vec<String>,
    pub guard_name: Option<String>,
    pub require_all: bool, // If true, user must have ALL roles, if false, user needs ANY role
}

#[derive(Clone)]
pub struct PermissionRequirement {
    pub permissions: Vec<String>,
    pub guard_name: Option<String>,
    pub require_all: bool,
}

impl RoleRequirement {
    pub fn new(roles: Vec<&str>) -> Self {
        Self {
            roles: roles.into_iter().map(|s| s.to_string()).collect(),
            guard_name: None,
            require_all: false,
        }
    }

    pub fn any_of(roles: Vec<&str>) -> Self {
        Self {
            roles: roles.into_iter().map(|s| s.to_string()).collect(),
            guard_name: None,
            require_all: false,
        }
    }

    pub fn all_of(roles: Vec<&str>) -> Self {
        Self {
            roles: roles.into_iter().map(|s| s.to_string()).collect(),
            guard_name: None,
            require_all: true,
        }
    }

    pub fn guard(mut self, guard_name: &str) -> Self {
        self.guard_name = Some(guard_name.to_string());
        self
    }
}

impl PermissionRequirement {
    pub fn new(permissions: Vec<&str>) -> Self {
        Self {
            permissions: permissions.into_iter().map(|s| s.to_string()).collect(),
            guard_name: None,
            require_all: false,
        }
    }

    pub fn any_of(permissions: Vec<&str>) -> Self {
        Self {
            permissions: permissions.into_iter().map(|s| s.to_string()).collect(),
            guard_name: None,
            require_all: false,
        }
    }

    pub fn all_of(permissions: Vec<&str>) -> Self {
        Self {
            permissions: permissions.into_iter().map(|s| s.to_string()).collect(),
            guard_name: None,
            require_all: true,
        }
    }

    pub fn guard(mut self, guard_name: &str) -> Self {
        self.guard_name = Some(guard_name.to_string());
        self
    }
}

// Extract user ID from JWT token in Authorization header
fn extract_user_id_from_token(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get("authorization")?;
    let auth_str = auth_header.to_str().ok()?;

    // Extract the token from the header
    let token = TokenUtils::extract_token_from_header(Some(auth_str)).ok()?;

    // Decode the JWT token to get claims
    let claims = AuthService::decode_token(token, "jwt-secret").ok()?;

    // Extract the user ID from the subject (sub) field
    let user_id = Ulid::from_string(&claims.sub).ok()?;

    Some(user_id)
}

pub async fn require_role(
    requirement: RoleRequirement,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
where
{
    move |req: Request, next: Next| {
        let requirement = requirement.clone();
        Box::pin(async move {
            let (parts, body) = req.into_parts();

            // Extract database pool from state
            let pool = match parts.extensions.get::<State<DbPool>>() {
                Some(State(pool)) => pool,
                None => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                        "error": "Database connection not available"
                    }))).into_response();
                }
            };

            // Extract user ID from token
            let user_id = match extract_user_id_from_token(&parts.headers) {
                Some(id) => id,
                None => {
                    return (StatusCode::UNAUTHORIZED, Json(json!({
                        "error": "Authentication required"
                    }))).into_response();
                }
            };

            // Get user roles
            let user_roles = match SysModelHasRoleService::get_model_roles(
                pool,
                User::model_type(),
                user_id,
                requirement.guard_name.as_deref()
            ) {
                Ok(roles) => roles,
                Err(_) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                        "error": "Failed to fetch user roles"
                    }))).into_response();
                }
            };

            let user_role_names: HashSet<String> = user_roles
                .into_iter()
                .map(|role| role.name)
                .collect();

            let required_roles: HashSet<String> = requirement.roles.into_iter().collect();

            let has_access = if requirement.require_all {
                // User must have ALL required roles
                required_roles.is_subset(&user_role_names)
            } else {
                // User must have ANY of the required roles
                !required_roles.is_disjoint(&user_role_names)
            };

            if !has_access {
                return (StatusCode::FORBIDDEN, Json(json!({
                    "error": "Insufficient role permissions"
                }))).into_response();
            }

            // Reconstruct request and continue
            let req = Request::from_parts(parts, body);
            next.run(req).await
        })
    }
}

pub async fn require_permission(
    requirement: PermissionRequirement,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
where
{
    move |req: Request, next: Next| {
        let requirement = requirement.clone();
        Box::pin(async move {
            let (parts, body) = req.into_parts();

            // Extract database pool from state
            let pool = match parts.extensions.get::<State<DbPool>>() {
                Some(State(pool)) => pool,
                None => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                        "error": "Database connection not available"
                    }))).into_response();
                }
            };

            // Extract user ID from token
            let user_id = match extract_user_id_from_token(&parts.headers) {
                Some(id) => id,
                None => {
                    return (StatusCode::UNAUTHORIZED, Json(json!({
                        "error": "Authentication required"
                    }))).into_response();
                }
            };

            // Check user permissions
            let has_permission = match check_user_permissions(
                pool,
                user_id,
                &requirement.permissions,
                requirement.guard_name.as_deref(),
                requirement.require_all,
            ).await {
                Ok(has_perm) => has_perm,
                Err(_) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                        "error": "Failed to check user permissions"
                    }))).into_response();
                }
            };

            if !has_permission {
                return (StatusCode::FORBIDDEN, Json(json!({
                    "error": "Insufficient permissions"
                }))).into_response();
            }

            // Reconstruct request and continue
            let req = Request::from_parts(parts, body);
            next.run(req).await
        })
    }
}

async fn check_user_permissions(
    pool: &DbPool,
    user_id: String,
    required_permissions: &[String],
    guard_name: Option<&str>,
    require_all: bool,
) -> Result<bool, anyhow::Error> {
    let user_permissions = SysModelHasPermissionService::get_model_permissions(pool, User::model_type(), user_id, guard_name)?;

    let user_permission_names: HashSet<String> = user_permissions
        .into_iter()
        .map(|permission| permission.name)
        .collect();

    let required_permission_set: HashSet<String> = required_permissions.iter().cloned().collect();

    let has_access = if require_all {
        // User must have ALL required permissions
        required_permission_set.is_subset(&user_permission_names)
    } else {
        // User must have ANY of the required permissions
        !required_permission_set.is_disjoint(&user_permission_names)
    };

    Ok(has_access)
}

// Convenience macros for easier usage
#[macro_export]
macro_rules! require_role {
    ($($role:expr_2021),+) => {
        $crate::app::http::middleware::role_middleware::require_role(
            $crate::app::http::middleware::role_middleware::RoleRequirement::any_of(vec![$($role),+])
        )
    };
}

#[macro_export]
macro_rules! require_all_roles {
    ($($role:expr_2021),+) => {
        $crate::app::http::middleware::role_middleware::require_role(
            $crate::app::http::middleware::role_middleware::RoleRequirement::all_of(vec![$($role),+])
        )
    };
}

#[macro_export]
macro_rules! require_permission {
    ($($permission:expr_2021),+) => {
        $crate::app::http::middleware::role_middleware::require_permission(
            $crate::app::http::middleware::role_middleware::PermissionRequirement::any_of(vec![$($permission),+])
        )
    };
}

#[macro_export]
macro_rules! require_all_permissions {
    ($($permission:expr_2021),+) => {
        $crate::app::http::middleware::role_middleware::require_permission(
            $crate::app::http::middleware::role_middleware::PermissionRequirement::all_of(vec![$($permission),+])
        )
    };
}