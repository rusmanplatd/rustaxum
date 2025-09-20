use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ulid::Ulid;
use crate::app::models::role::{Role, CreateRole, UpdateRole};
use crate::app::services::role_service::RoleService;

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

pub async fn index(
    State(pool): State<PgPool>,
    Query(params): Query<ListRolesQuery>
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    match RoleService::list(&pool, limit, offset).await {
        Ok(roles) => {
            let total = RoleService::count(&pool).await.unwrap_or(0);
            let role_data: Vec<RoleData> = roles.into_iter().map(RoleData::from).collect();
            let response = RoleListResponse {
                data: role_data,
                meta: Meta {
                    total,
                    limit,
                    offset,
                },
            };
            (StatusCode::OK, Json(response)).into_response()
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
    State(pool): State<PgPool>,
    Json(payload): Json<CreateRoleRequest>
) -> impl IntoResponse {
    let create_role = CreateRole {
        name: payload.name,
        description: payload.description,
        guard_name: payload.guard_name,
    };

    match RoleService::create(&pool, create_role).await {
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
    State(pool): State<PgPool>,
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

    match RoleService::find_by_id(&pool, role_id).await {
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
    State(pool): State<PgPool>,
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

    match RoleService::update(&pool, role_id, update_role).await {
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
    State(pool): State<PgPool>,
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

    match RoleService::delete(&pool, role_id).await {
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
    State(pool): State<PgPool>,
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

    match RoleService::assign_to_user(&pool, user_id, role_id).await {
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
    State(pool): State<PgPool>,
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

    match RoleService::remove_from_user(&pool, user_id, role_id).await {
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
    State(pool): State<PgPool>,
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

    match RoleService::get_user_roles(&pool, user_id, None).await {
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