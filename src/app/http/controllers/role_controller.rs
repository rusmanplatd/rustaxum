use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
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
    summary = "List all roles",
    description = "Get all roles with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Sort field and direction. Available fields: id, name, description, created_at, updated_at (prefix with '-' for descending)"),
        ("include" = Option<String>, Query, description = "Comma-separated list of relationships to include. Available: permissions"),
        ("filter" = Option<serde_json::Value>, Query, description = "Filter parameters. Available filters: name, description (e.g., filter[name]=admin, filter[description]=Administrator)"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to select. Available: id, name, description, created_at, updated_at"),
        ("cursor" = Option<String>, Query, description = "Cursor for cursor-based pagination"),
        ("pagination_type" = Option<String>, Query, description = "Pagination type: 'offset' or 'cursor' (default: cursor)"),
    ),
    responses(
        (status = 200, description = "List of roles", body = Vec<crate::app::models::role::RoleResponse>),
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

pub async fn store(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateRoleRequest>
) -> impl IntoResponse {
    let create_role = CreateRole {
        name: payload.name,
        description: payload.description,
        guard_name: payload.guard_name,
    };

    match RoleService::create(&pool, create_role, None).await {
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

pub async fn show(
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

    match SysModelHasRoleService::assign_role_to_model(&pool, User::model_type(), diesel_user_id, diesel_role_id, None, None, None).await {
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

    match SysModelHasRoleService::remove_role_from_model(&pool, User::model_type(), user_id.to_string(), role_id.to_string(), None).await {
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