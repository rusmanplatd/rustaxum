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
use crate::app::models::role::{Role, CreateRole, UpdateRole};
use crate::app::services::role_service::RoleService;
use crate::app::services::sys_model_has_role_service::SysModelHasRoleService;
use crate::app::models::user::User;
use crate::app::models::HasModelType;
use crate::app::models::DieselUlid;
use crate::app::query_builder::{QueryParams, QueryBuilderService};

#[derive(Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub guard_name: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub guard_name: Option<String>,
}

#[derive(Deserialize)]
pub struct ListRolesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct AssignRoleRequest {
    pub user_id: String,
}

#[derive(Serialize)]
pub struct RoleListResponse {
    pub data: Vec<RoleData>,
    pub meta: Meta,
}

#[derive(Serialize)]
pub struct RoleData {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub guard_name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct Meta {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

impl From<Role> for RoleData {
    fn from(role: Role) -> Self {
        Self {
            id: role.id.to_string(),
            name: role.name,
            description: role.description,
            guard_name: role.guard_name,
            created_at: role.created_at.to_rfc3339(),
            updated_at: role.updated_at.to_rfc3339(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/roles",
    tag = "Roles",
    summary = "List all roles with advanced permission filtering and multi-tenant support",
    description = "Retrieve roles with Laravel-style advanced querying, permission-based filtering, multi-column sorting, and organizational scoping. Essential for RBAC (Role-Based Access Control) and security management.",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Items per page (default: 15, max: 100). Use 50-100 for admin dashboards, 10-25 for user interfaces"),
        ("sort" = Option<String>, Query, description = "Multi-column sorting with permission hierarchy support. Available fields: id, name, description, guard_name, created_at, updated_at. Syntax: 'field1,-field2,field3:desc'. Examples: 'name', 'guard_name,name:asc', '-created_at,name'"),
        ("include" = Option<String>, Query, description = "Eager load relationships with comprehensive audit user support and security context. Available: permissions, users, organization, createdBy, updatedBy, deletedBy, createdBy.organizations, updatedBy.organizations, deletedBy.organizations, createdBy.organizations.position, updatedBy.organizations.position, deletedBy.organizations.position, createdBy.organizations.position.level, updatedBy.organizations.position.level, deletedBy.organizations.position.level. Supports deep nested loading for complete RBAC and audit analysis. Examples: 'permissions,createdBy.organizations.position.level', 'users.organizations,organization'"),
        ("filter" = Option<serde_json::Value>, Query, description = "Advanced filtering with 15+ operators for role management. Available filters: id, name, description, guard_name, created_at, updated_at. Operators: eq, ne, gt, gte, lt, lte, like, ilike, contains, starts_with, ends_with, in, not_in, is_null, is_not_null, between. Examples: filter[name][contains]=admin, filter[guard_name][eq]=api, filter[name][starts_with]=Super"),
        ("fields" = Option<String>, Query, description = "Field selection for optimized RBAC queries. Available: id, name, description, guard_name, created_at, updated_at. Supports relationship field selection. Examples: fields[roles]=id,name,guard_name, fields[permissions]=id,action,resource"),
        ("cursor" = Option<String>, Query, description = "Cursor for high-performance pagination with role hierarchy indexing"),
        ("pagination_type" = Option<String>, Query, description = "Pagination strategy: 'offset' (traditional) or 'cursor' (high-performance for large role datasets, recommended default)"),
    ),
    responses(
        (status = 200, description = "List of roles with RBAC metadata", body = Vec<crate::app::models::role::RoleResponse>),
        (status = 400, description = "Invalid query parameters", body = crate::app::docs::ErrorResponse),
        (status = 401, description = "Unauthorized access", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>
) -> impl IntoResponse {
    match <Role as QueryBuilderService<Role>>::index(Query(params), &pool) {
        Ok(result) => {
            (StatusCode::OK, Json(serde_json::json!(result))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Failed to fetch roles",
                "message": e.to_string()
            }))).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/roles",
    tag = "Roles",
    summary = "Create a new role",
    description = "Create a new role with name, description, and guard name. Requires authentication.",
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created successfully", body = crate::app::models::role::RoleResponse),
        (status = 400, description = "Invalid request data", body = crate::app::docs::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn store(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<crate::app::http::middleware::auth_guard::AuthUser>,
    Json(payload): Json<CreateRoleRequest>
) -> impl IntoResponse {
    let create_role = CreateRole {
        name: payload.name,
        description: payload.description,
        guard_name: payload.guard_name,
    };

    match RoleService::create(&pool, create_role, &auth_user.user_id).await {
        Ok(role) => {
            let role_data = RoleData::from(role);
            (StatusCode::CREATED, Json(json!({
                "data": role_data,
                "message": "Role created successfully"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Failed to create role",
                "message": e.to_string()
            }))).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/roles/{id}",
    tag = "Roles",
    summary = "Get role by ID",
    description = "Retrieve a specific role by its ID with full role details.",
    params(
        ("id" = String, Path, description = "Role ID (ULID format)")
    ),
    responses(
        (status = 200, description = "Role retrieved successfully", body = crate::app::models::role::RoleResponse),
        (status = 400, description = "Invalid role ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Role not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn show(
    State(pool): State<DbPool>,
    Path(id): Path<String>
) -> impl IntoResponse{
    let role_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Invalid role ID format"
            }))).into_response();
        }
    };

    match RoleService::find_by_id(&pool, role_id.to_string()) {
        Ok(Some(role)) => {
            let role_data = RoleData::from(role);
            (StatusCode::OK, Json(json!({
                "data": role_data,
                "message": "Role retrieved successfully"
            }))).into_response()
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(json!({
                "error": "Role not found"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Failed to fetch role",
                "message": e.to_string()
            }))).into_response()
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/roles/{id}",
    tag = "Roles",
    summary = "Update role",
    description = "Update an existing role's name, description, or guard name.",
    params(
        ("id" = String, Path, description = "Role ID (ULID format)")
    ),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated successfully", body = crate::app::models::role::RoleResponse),
        (status = 400, description = "Invalid request data or role ID", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Role not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateRoleRequest>
) -> impl IntoResponse {
    let role_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Invalid role ID format"
            }))).into_response();
        }
    };

    let update_role = UpdateRole {
        name: payload.name,
        description: payload.description,
        guard_name: payload.guard_name,
    };

    match RoleService::update(&pool, role_id.to_string(), update_role, None).await {
        Ok(role) => {
            let role_data = RoleData::from(role);
            (StatusCode::OK, Json(json!({
                "data": role_data,
                "message": "Role updated successfully"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Failed to update role",
                "message": e.to_string()
            }))).into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/roles/{id}",
    tag = "Roles",
    summary = "Delete role",
    description = "Soft delete a role by its ID. The role will be marked as deleted but retained in the database.",
    params(
        ("id" = String, Path, description = "Role ID (ULID format)")
    ),
    responses(
        (status = 200, description = "Role deleted successfully", body = crate::app::docs::MessageResponse),
        (status = 400, description = "Invalid role ID format or deletion failed", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Role not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn destroy(
    State(pool): State<DbPool>,
    Path(id): Path<String>
) -> impl IntoResponse {
    let role_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Invalid role ID format"
            }))).into_response();
        }
    };

    match RoleService::delete(&pool, role_id.to_string(), None).await {
        Ok(_) => {
            (StatusCode::OK, Json(json!({
                "message": "Role deleted successfully"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Failed to delete role",
                "message": e.to_string()
            }))).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/roles/{id}/assign",
    tag = "Roles",
    summary = "Assign role to user",
    description = "Assign a role to a user by creating a model-has-role relationship.",
    params(
        ("id" = String, Path, description = "Role ID (ULID format)")
    ),
    request_body = AssignRoleRequest,
    responses(
        (status = 200, description = "Role assigned to user successfully", body = crate::app::docs::MessageResponse),
        (status = 400, description = "Invalid role ID or user ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Role or user not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn assign_to_user(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<AssignRoleRequest>
) -> impl IntoResponse {
    let role_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Invalid role ID format"
            }))).into_response();
        }
    };

    let user_id = match Ulid::from_string(&payload.user_id) {
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

    let diesel_role_id = match DieselUlid::from_string(&role_id.to_string()) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Invalid role ID format"
            }))).into_response();
        }
    };

    match SysModelHasRoleService::assign_role_to_model(&pool, User::model_type(), diesel_user_id, diesel_role_id, None, None).await {
        Ok(_) => {
            (StatusCode::OK, Json(json!({
                "message": "Role assigned to user successfully"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Failed to assign role to user",
                "message": e.to_string()
            }))).into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/roles/{id}/users/{user_id}",
    tag = "Roles",
    summary = "Remove role from user",
    description = "Remove a role assignment from a user by deleting the model-has-role relationship.",
    params(
        ("id" = String, Path, description = "Role ID (ULID format)"),
        ("user_id" = String, Path, description = "User ID (ULID format)")
    ),
    responses(
        (status = 200, description = "Role removed from user successfully", body = crate::app::docs::MessageResponse),
        (status = 400, description = "Invalid role ID or user ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Role or user not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn remove_from_user(
    State(pool): State<DbPool>,
    Path((id, user_id)): Path<(String, String)>
) -> impl IntoResponse {
    let role_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Invalid role ID format"
            }))).into_response();
        }
    };

    let user_id = match Ulid::from_string(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Invalid user ID format"
            }))).into_response();
        }
    };

    match SysModelHasRoleService::remove_role_from_model(&pool, User::model_type(), user_id.to_string(), role_id.to_string()).await {
        Ok(_) => {
            (StatusCode::OK, Json(json!({
                "message": "Role removed from user successfully"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(json!({
                "error": "Failed to remove role from user",
                "message": e.to_string()
            }))).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/users/{user_id}/roles",
    tag = "Roles",
    summary = "Get user roles",
    description = "Retrieve all roles assigned to a specific user.",
    params(
        ("user_id" = String, Path, description = "User ID (ULID format)")
    ),
    responses(
        (status = 200, description = "User roles retrieved successfully", body = Vec<crate::app::models::role::RoleResponse>),
        (status = 400, description = "Invalid user ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "User not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn get_user_roles(
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

    match SysModelHasRoleService::get_model_roles(&pool, User::model_type(), user_id.to_string(), None) {
        Ok(roles) => {
            let role_data: Vec<RoleData> = roles.into_iter().map(RoleData::from).collect();
            (StatusCode::OK, Json(json!({
                "data": role_data,
                "message": "User sys_roles retrieved successfully"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Failed to fetch user roles",
                "message": e.to_string()
            }))).into_response()
        }
    }
}