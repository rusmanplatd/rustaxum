use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use ulid::Ulid;
use crate::database::DbPool;
use std::collections::HashMap;

use crate::app::models::sys_model_has_permission::{CreateSysModelHasPermission, UpdateSysModelHasPermission, SysModelHasPermissionResponse};
use crate::app::services::sys_model_has_permission_service::SysModelHasPermissionService;
use crate::app::http::requests::{CreateSysModelHasPermissionRequest, UpdateSysModelHasPermissionRequest};

use crate::app::docs::{ErrorResponse, MessageResponse};

#[utoipa::path(
    get,
    path = "/api/sys-model-has-permissions",
    tag = "Model Permissions",
    summary = "List all model permissions",
    description = "Get all model permission assignments with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination"),
        ("limit" = Option<u32>, Query, description = "Number of items per page (max 100)"),
        ("sort" = Option<String>, Query, description = "Sort field"),
        ("direction" = Option<String>, Query, description = "Sort direction (asc/desc)"),
    ),
    responses(
        (status = 200, description = "List of model permissions", body = Vec<SysModelHasPermissionResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    match SysModelHasPermissionService::list(&pool, params) {
        Ok(permissions) => {
            let responses: Vec<_> = permissions.into_iter().map(|p| p.to_response()).collect();
            (StatusCode::OK, ResponseJson(responses)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/sys-model-has-permissions/{id}",
    tag = "Model Permissions",
    summary = "Get model permission by ID",
    description = "Retrieve a specific model permission assignment by its unique identifier",
    params(
        ("id" = String, Path, description = "Model permission unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Model permission details", body = SysModelHasPermissionResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Model permission not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn show(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    let permission_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match SysModelHasPermissionService::find_by_id(&pool, permission_id.to_string()) {
        Ok(Some(permission)) => (StatusCode::OK, ResponseJson(permission.to_response())).into_response(),
        Ok(None) => {
            let error = ErrorResponse {
                error: "Model permission not found".to_string(),
            };
            (StatusCode::NOT_FOUND, ResponseJson(error)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/sys-model-has-permissions",
    tag = "Model Permissions",
    summary = "Create new model permission",
    description = "Create a new model permission assignment",
    request_body = CreateSysModelHasPermissionRequest,
    responses(
        (status = 201, description = "Model permission created successfully", body = SysModelHasPermissionResponse),
        (status = 400, description = "Validation error or bad request", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn store(State(pool): State<DbPool>, request: CreateSysModelHasPermissionRequest) -> impl IntoResponse {
    let payload = CreateSysModelHasPermission {
        model_type: request.model_type,
        model_id: request.model_id.to_string(),
        permission_id: request.permission_id.to_string(),
        scope_type: request.scope_type,
        scope_id: request.scope_id.map(|id| id.to_string()),
    };

    match SysModelHasPermissionService::create(&pool, payload) {
        Ok(permission) => (StatusCode::CREATED, ResponseJson(permission.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/sys-model-has-permissions/{id}",
    tag = "Model Permissions",
    summary = "Update model permission",
    description = "Update an existing model permission assignment",
    params(
        ("id" = String, Path, description = "Model permission unique identifier (ULID format)")
    ),
    request_body = UpdateSysModelHasPermissionRequest,
    responses(
        (status = 200, description = "Model permission updated successfully", body = SysModelHasPermissionResponse),
        (status = 400, description = "Invalid ID format or validation error", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Model permission not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    request: UpdateSysModelHasPermissionRequest,
) -> impl IntoResponse {
    let permission_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    let payload = UpdateSysModelHasPermission {
        model_type: request.model_type,
        model_id: request.model_id.map(|id| id.to_string()),
        permission_id: request.permission_id.map(|id| id.to_string()),
        scope_type: request.scope_type,
        scope_id: request.scope_id.map(|id| id.to_string()),
    };

    match SysModelHasPermissionService::update(&pool, permission_id.to_string(), payload) {
        Ok(permission) => (StatusCode::OK, ResponseJson(permission.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/sys-model-has-permissions/{id}",
    tag = "Model Permissions",
    summary = "Delete model permission",
    description = "Permanently delete a model permission assignment",
    params(
        ("id" = String, Path, description = "Model permission unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Model permission deleted successfully", body = crate::app::docs::MessageResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Model permission not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn destroy(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    let permission_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match SysModelHasPermissionService::delete(&pool, permission_id.to_string()) {
        Ok(_) => {
            let message = MessageResponse {
                message: "Model permission deleted successfully".to_string(),
            };
            (StatusCode::OK, ResponseJson(message)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/models/{model_type}/{model_id}/permissions",
    tag = "Model Permissions",
    summary = "Get permissions by model",
    description = "Retrieve all permission assignments for a specific model",
    params(
        ("model_type" = String, Path, description = "Model type (e.g., User, Organization)"),
        ("model_id" = String, Path, description = "Model unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "List of model permissions", body = Vec<SysModelHasPermissionResponse>),
        (status = 400, description = "Invalid model ID format", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn by_model(State(pool): State<DbPool>, Path((model_type, model_id)): Path<(String, String)>) -> impl IntoResponse {
    match SysModelHasPermissionService::find_by_model(&pool, &model_type, model_id) {
        Ok(permissions) => {
            let responses: Vec<_> = permissions.into_iter().map(|p| p.to_response()).collect();
            (StatusCode::OK, ResponseJson(responses)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}