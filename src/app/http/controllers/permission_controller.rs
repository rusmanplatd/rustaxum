use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
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
    pub name: String,
    pub guard_name: Option<String>,
    pub resource: Option<String>,
    pub action: String,
}

#[derive(Deserialize)]
pub struct UpdatePermissionRequest {
    pub name: Option<String>,
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
    pub name: String,
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
            name: permission.name,
            guard_name: permission.guard_name,
            resource: permission.resource,
            action: permission.action,
            created_at: permission.created_at.to_rfc3339(),
            updated_at: permission.updated_at.to_rfc3339(),
        }
    }
}

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
    Json(payload): Json<CreatePermissionRequest>
) -> impl IntoResponse {
    let create_permission = CreatePermission {
        name: payload.name,
        guard_name: payload.guard_name,
        resource: payload.resource,
        action: payload.action,
    };

    match PermissionService::create(&pool, create_permission) {
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
        name: payload.name,
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