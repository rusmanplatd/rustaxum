use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use crate::database::DbPool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ulid::Ulid;
use crate::app::models::permission::{Permission, CreatePermission, UpdatePermission};
use crate::app::services::permission_service::PermissionService;
use crate::app::services::sys_model_has_permission_service::SysModelHasPermissionService;
use crate::app::models::user::User;
use crate::app::models::HasModelType;
use crate::app::models::DieselUlid;
use crate::app::query_builder::{QueryParams, QueryBuilderService};

#[derive(Deserialize)]
pub struct CreatePermissionRequest {
    pub guard_name: Option<String>,
    pub resource: Option<String>,
    pub action: String,
}

#[derive(Deserialize)]
pub struct UpdatePermissionRequest {
    pub guard_name: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
}

#[derive(Deserialize)]
pub struct ListPermissionsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub resource: Option<String>,
    pub action: Option<String>,
}

#[derive(Deserialize)]
pub struct AssignPermissionRequest {
    pub role_id: String,
}

#[derive(Serialize)]
pub struct PermissionListResponse {
    pub data: Vec<PermissionData>,
    pub meta: Meta,
}

#[derive(Serialize)]
pub struct PermissionData {
    pub id: String,
    pub guard_name: String,
    pub resource: Option<String>,
    pub action: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct Meta {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

impl From<Permission> for PermissionData {
    fn from(permission: Permission) -> Self {
        Self {
            id: permission.id.to_string(),
            guard_name: permission.guard_name,
            resource: permission.resource,
            action: permission.action,
            created_at: permission.created_at.to_rfc3339(),
            updated_at: permission.updated_at.to_rfc3339(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/permissions",
    tag = "Permissions",
    summary = "List all permissions with advanced resource and action filtering",
    description = "Retrieve permissions with Laravel-style advanced querying, resource-action filtering, multi-column sorting, and role relationship loading. Critical for fine-grained access control and security auditing.",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Items per page (default: 15, max: 100). Use 100 for admin audits, 25-50 for permission management interfaces"),
        ("sort" = Option<String>, Query, description = "Multi-column sorting with action/resource grouping support. Available fields: id, guard_name, resource, action, created_at, updated_at. Syntax: 'field1,-field2,field3:desc'. Examples: 'resource,action', 'guard_name,resource:asc,action', '-created_at,action'"),
        ("include" = Option<String>, Query, description = "Eager load relationships with comprehensive audit user support and security context. Available: roles, organization, createdBy, updatedBy, deletedBy, createdBy.organizations, updatedBy.organizations, deletedBy.organizations, createdBy.organizations.position, updatedBy.organizations.position, deletedBy.organizations.position, createdBy.organizations.position.level, updatedBy.organizations.position.level, deletedBy.organizations.position.level. Supports deep nested loading for complete permission analysis and audit tracking. Examples: 'roles,createdBy.organizations.position.level', 'organization,updatedBy.organizations'"),
        ("filter" = Option<serde_json::Value>, Query, description = "Advanced filtering with 15+ operators for permission management. Available filters: id, guard_name, resource, action, created_at, updated_at. Operators: eq, ne, gt, gte, lt, lte, like, ilike, contains, starts_with, ends_with, in, not_in, is_null, is_not_null, between. Examples: filter[action][contains]=create, filter[resource][eq]=users, filter[guard_name][in]=api,web, filter[action][starts_with]=manage"),
        ("fields" = Option<String>, Query, description = "Field selection for optimized permission queries. Available: id, guard_name, resource, action, created_at, updated_at. Supports relationship field selection. Examples: fields[permissions]=id,resource,action, fields[roles]=id,name"),
        ("cursor" = Option<String>, Query, description = "Cursor for high-performance pagination with permission indexing"),
        ("pagination_type" = Option<String>, Query, description = "Pagination strategy: 'offset' (traditional) or 'cursor' (high-performance for large permission datasets, recommended default)"),
    ),
    responses(
        (status = 200, description = "List of permissions with security metadata", body = Vec<crate::app::models::permission::PermissionResponse>),
        (status = 400, description = "Invalid query parameters", body = crate::app::docs::ErrorResponse),
        (status = 401, description = "Unauthorized access", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>
) -> impl IntoResponse {
    match <Permission as QueryBuilderService<Permission>>::index(Query(params), &pool) {
        Ok(result) => {
            (StatusCode::OK, Json(serde_json::json!(result))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Failed to fetch permissions",
                "message": e.to_string()
            }))).into_response()
        }
    }
}

pub async fn store(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<crate::app::http::middleware::auth_guard::AuthUser>,
    Json(payload): Json<CreatePermissionRequest>
) -> impl IntoResponse {
    let create_permission = CreatePermission {
        guard_name: payload.guard_name,
        resource: payload.resource,
        action: payload.action,
    };

    match PermissionService::create(&pool, create_permission, &auth_user.user_id).await {
        Ok(permission) => {
            let permission_data = PermissionData::from(permission);
            (StatusCode::CREATED, Json(json!({
                "data": permission_data,
                "message": "Permission created successfully"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Failed to create permission",
                "message": e.to_string()
            }))).into_response()
        }
    }
}

pub async fn show(
    State(pool): State<DbPool>,
    Path(id): Path<String>
) -> impl IntoResponse {
    match PermissionService::find_by_id(&pool, id) {
        Ok(Some(permission)) => {
            let permission_data = PermissionData::from(permission);
            (StatusCode::OK, Json(json!({
                "data": permission_data,
                "message": "Permission retrieved successfully"
            }))).into_response()
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(json!({
                "error": "Permission not found"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Failed to fetch permission",
                "message": e.to_string()
            }))).into_response()
        }
    }
}

pub async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdatePermissionRequest>
) -> impl IntoResponse {
    let permission_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Invalid permission ID format"
            }))).into_response();
        }
    };

    let update_permission = UpdatePermission {
        guard_name: payload.guard_name,
        resource: payload.resource,
        action: payload.action,
    };

    match PermissionService::update(&pool, permission_id.to_string(), update_permission) {
        Ok(permission) => {
            let permission_data = PermissionData::from(permission);
            (StatusCode::OK, Json(json!({
                "data": permission_data,
                "message": "Permission updated successfully"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Failed to update permission",
                "message": e.to_string()
            }))).into_response()
        }
    }
}

pub async fn destroy(
    State(pool): State<DbPool>,
    Path(id): Path<String>
) -> impl IntoResponse {
    let permission_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Invalid permission ID format"
            }))).into_response();
        }
    };

    match PermissionService::delete(&pool, permission_id.to_string()) {
        Ok(_) => {
            (StatusCode::OK, Json(json!({
                "message": "Permission deleted successfully"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Failed to delete permission",
                "message": e.to_string()
            }))).into_response()
        }
    }
}

pub async fn assign_to_role(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<AssignPermissionRequest>
) -> impl IntoResponse {
    let permission_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Invalid permission ID format"
            }))).into_response();
        }
    };

    let role_id = match Ulid::from_string(&payload.role_id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Invalid role ID format"
            }))).into_response();
        }
    };

    match PermissionService::assign_to_role(&pool, role_id.to_string(), permission_id.to_string()) {
        Ok(_) => {
            (StatusCode::OK, Json(json!({
                "message": "Permission assigned to role successfully"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Failed to assign permission to role",
                "message": e.to_string()
            }))).into_response()
        }
    }
}

pub async fn remove_from_role(
    State(pool): State<DbPool>,
    Path((id, role_id)): Path<(String, String)>
) -> impl IntoResponse {
    let permission_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Invalid permission ID format"
            }))).into_response();
        }
    };

    let role_id = match Ulid::from_string(&role_id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Invalid role ID format"
            }))).into_response();
        }
    };

    match PermissionService::remove_from_role(&pool, role_id.to_string(), permission_id.to_string()) {
        Ok(_) => {
            (StatusCode::OK, Json(json!({
                "message": "Permission removed from role successfully"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Failed to remove permission from role",
                "message": e.to_string()
            }))).into_response()
        }
    }
}

pub async fn get_role_permissions(
    State(pool): State<DbPool>,
    Path(role_id): Path<String>
) -> impl IntoResponse {
    let role_id = match Ulid::from_string(&role_id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Invalid role ID format"
            }))).into_response();
        }
    };

    match PermissionService::get_role_permissions(&pool, role_id.to_string(), None) {
        Ok(permissions) => {
            let permission_data: Vec<PermissionData> = permissions.into_iter().map(PermissionData::from).collect();
            (StatusCode::OK, Json(json!({
                "data": permission_data,
                "message": "Role permissions retrieved successfully"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Failed to fetch role permissions",
                "message": e.to_string()
            }))).into_response()
        }
    }
}

pub async fn get_user_permissions(
    State(pool): State<DbPool>,
    Path(user_id): Path<String>
) -> impl IntoResponse {
    let user_id = match Ulid::from_string(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Invalid user ID format"
            }))).into_response();
        }
    };

    let diesel_user_id = match DieselUlid::from_string(&user_id.to_string()) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Invalid user ID format"
            }))).into_response();
        }
    };

    match SysModelHasPermissionService::get_model_permissions(&pool, User::model_type(), diesel_user_id, None) {
        Ok(permissions) => {
            let permission_data: Vec<PermissionData> = permissions.into_iter().map(PermissionData::from).collect();
            (StatusCode::OK, Json(json!({
                "data": permission_data,
                "message": "User permissions retrieved successfully"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Failed to fetch user permissions",
                "message": e.to_string()
            }))).into_response()
        }
    }
}