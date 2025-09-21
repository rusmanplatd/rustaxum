use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use ulid::Ulid;
use crate::database::DbPool;
use std::collections::HashMap;

use crate::app::models::sys_model_has_role::{CreateSysModelHasRole, UpdateSysModelHasRole, SysModelHasRoleResponse};
use crate::app::services::sys_model_has_role_service::SysModelHasRoleService;
use crate::app::models::model_types;
use crate::app::http::requests::{CreateSysModelHasRoleRequest, UpdateSysModelHasRoleRequest};

use crate::app::docs::{ErrorResponse, MessageResponse};

#[utoipa::path(
    get,
    path = "/api/sys-model-has-roles",
    tag = "Model Roles",
    summary = "List all model roles",
    description = "Get all model role assignments with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination"),
        ("limit" = Option<u32>, Query, description = "Number of items per page (max 100)"),
        ("sort" = Option<String>, Query, description = "Sort field"),
        ("direction" = Option<String>, Query, description = "Sort direction (asc/desc)"),
    ),
    responses(
        (status = 200, description = "List of model roles", body = Vec<SysModelHasRoleResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    match SysModelHasRoleService::list(&pool, params).await {
        Ok(roles) => {
            let responses: Vec<_> = roles.into_iter().map(|r| r.to_response()).collect();
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
    path = "/api/sys-model-has-roles/{id}",
    tag = "Model Roles",
    summary = "Get model role by ID",
    description = "Retrieve a specific model role assignment by its unique identifier",
    params(
        ("id" = String, Path, description = "Model role unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Model role details", body = SysModelHasRoleResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Model role not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn show(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    let role_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match SysModelHasRoleService::find_by_id(&pool, role_id).await {
        Ok(Some(role)) => (StatusCode::OK, ResponseJson(role.to_response())).into_response(),
        Ok(None) => {
            let error = ErrorResponse {
                error: "Model role not found".to_string(),
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
    path = "/api/sys-model-has-roles",
    tag = "Model Roles",
    summary = "Create new model role",
    description = "Create a new model role assignment",
    request_body = CreateSysModelHasRoleRequest,
    responses(
        (status = 201, description = "Model role created successfully", body = SysModelHasRoleResponse),
        (status = 400, description = "Validation error or bad request", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn store(State(pool): State<DbPool>, request: CreateSysModelHasRoleRequest) -> impl IntoResponse {
    let payload = CreateSysModelHasRole {
        model_type: request.model_type,
        model_id: request.model_id,
        role_id: request.role_id,
        scope_type: request.scope_type,
        scope_id: request.scope_id,
    };

    match SysModelHasRoleService::create(&pool, payload).await {
        Ok(role) => (StatusCode::CREATED, ResponseJson(role.to_response())).into_response(),
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
    path = "/api/sys-model-has-roles/{id}",
    tag = "Model Roles",
    summary = "Update model role",
    description = "Update an existing model role assignment",
    params(
        ("id" = String, Path, description = "Model role unique identifier (ULID format)")
    ),
    request_body = UpdateSysModelHasRoleRequest,
    responses(
        (status = 200, description = "Model role updated successfully", body = SysModelHasRoleResponse),
        (status = 400, description = "Invalid ID format or validation error", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Model role not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    request: UpdateSysModelHasRoleRequest,
) -> impl IntoResponse {
    let role_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    let payload = UpdateSysModelHasRole {
        model_type: request.model_type,
        model_id: request.model_id,
        role_id: request.role_id,
        scope_type: request.scope_type,
        scope_id: request.scope_id,
    };

    match SysModelHasRoleService::update(&pool, role_id, payload).await {
        Ok(role) => (StatusCode::OK, ResponseJson(role.to_response())).into_response(),
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
    path = "/api/sys-model-has-roles/{id}",
    tag = "Model Roles",
    summary = "Delete model role",
    description = "Permanently delete a model role assignment",
    params(
        ("id" = String, Path, description = "Model role unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Model role deleted successfully", body = crate::app::docs::MessageResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Model role not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn destroy(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    let role_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match SysModelHasRoleService::delete(&pool, role_id).await {
        Ok(_) => {
            let message = MessageResponse {
                message: "Model role deleted successfully".to_string(),
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
    path = "/api/models/{model_type}/{model_id}/roles",
    tag = "Model Roles",
    summary = "Get roles by model",
    description = "Retrieve all role assignments for a specific model",
    params(
        ("model_type" = String, Path, description = "Model type (e.g., User, Organization)"),
        ("model_id" = String, Path, description = "Model unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "List of model roles", body = Vec<SysModelHasRoleResponse>),
        (status = 400, description = "Invalid model ID format", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn by_model(State(pool): State<DbPool>, Path((model_type, model_id)): Path<(String, String)>) -> impl IntoResponse {
    // Validate model type
    if !model_types::is_valid_model_type(&model_type) {
        let error = ErrorResponse {
            error: format!("Invalid model type: {}. Valid types are: {:?}", model_type, model_types::valid_model_types()),
        };
        return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
    }

    let model_ulid = match Ulid::from_string(&model_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid model ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match SysModelHasRoleService::find_by_model(&pool, &model_type, model_ulid).await {
        Ok(roles) => {
            let responses: Vec<_> = roles.into_iter().map(|r| r.to_response()).collect();
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