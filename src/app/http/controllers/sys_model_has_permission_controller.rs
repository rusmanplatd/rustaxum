use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use crate::database::DbPool;
use crate::app::models::DieselUlid;

use crate::app::models::sys_model_has_permission::{CreateSysModelHasPermission, UpdateSysModelHasPermission, SysModelHasPermissionResponse, SysModelHasPermission};
use crate::app::services::sys_model_has_permission_service::SysModelHasPermissionService;
use crate::app::http::requests::{CreateSysModelHasPermissionRequest, UpdateSysModelHasPermissionRequest};
use crate::app::query_builder::{QueryParams, QueryBuilderService};

use crate::app::docs::{ErrorResponse, MessageResponse};

#[utoipa::path(
    get,
    path = "/api/sys-model-has-permissions",
    tag = "Model Permissions",
    summary = "List all model permissions",
    description = "Get all model permission assignments with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Sort field and direction. Available fields: id, model_type, model_id, permission_id, scope_type, scope_id, created_at, updated_at (prefix with '-' for descending)"),
        ("include" = Option<String>, Query, description = "Comma-separated list of relationships to include. Available: permission, model, createdBy, updatedBy, deletedBy, createdBy.organizations.position.level, updatedBy.organizations.position.level, deletedBy.organizations.position.level"),
        ("filter" = Option<serde_json::Value>, Query, description = "Filter parameters. Available filters: model_type, model_id, permission_id, scope_type, scope_id (e.g., filter[model_type]=User, filter[scope_type]=organization)"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to select. Available: id, model_type, model_id, permission_id, scope_type, scope_id, created_at, updated_at"),
        ("cursor" = Option<String>, Query, description = "Cursor for cursor-based pagination"),
        ("pagination_type" = Option<String>, Query, description = "Pagination type: 'offset' or 'cursor' (default: cursor)"),
    ),
    responses(
        (status = 200, description = "List of model permissions", body = Vec<SysModelHasPermissionResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    match <SysModelHasPermission as QueryBuilderService<SysModelHasPermission>>::index(Query(params), &pool) {
        Ok(result) => {
            (StatusCode::OK, ResponseJson(serde_json::json!(result))).into_response()
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
    let permission_id = match DieselUlid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match SysModelHasPermissionService::find_by_id(&pool, permission_id) {
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
    let model_id = match DieselUlid::from_string(&request.model_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid model_id format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    let permission_id = match DieselUlid::from_string(&request.permission_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid permission_id format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    let scope_id = match request.scope_id {
        Some(id_str) => {
            match DieselUlid::from_string(&id_str) {
                Ok(id) => Some(id),
                Err(_) => {
                    let error = ErrorResponse {
                        error: "Invalid scope_id format".to_string(),
                    };
                    return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
                }
            }
        }
        None => None,
    };

    let payload = CreateSysModelHasPermission {
        model_type: request.model_type,
        model_id,
        permission_id,
        scope_type: request.scope_type,
        scope_id,
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
    let permission_id = match DieselUlid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    let model_id = match request.model_id {
        Some(id_str) => {
            match DieselUlid::from_string(&id_str) {
                Ok(id) => Some(id),
                Err(_) => {
                    let error = ErrorResponse {
                        error: "Invalid model_id format".to_string(),
                    };
                    return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
                }
            }
        }
        None => None,
    };

    let permission_id_update = match request.permission_id {
        Some(id_str) => {
            match DieselUlid::from_string(&id_str) {
                Ok(id) => Some(id),
                Err(_) => {
                    let error = ErrorResponse {
                        error: "Invalid permission_id format".to_string(),
                    };
                    return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
                }
            }
        }
        None => None,
    };

    let scope_id = match request.scope_id {
        Some(id_str) => {
            match DieselUlid::from_string(&id_str) {
                Ok(id) => Some(id),
                Err(_) => {
                    let error = ErrorResponse {
                        error: "Invalid scope_id format".to_string(),
                    };
                    return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
                }
            }
        }
        None => None,
    };

    let payload = UpdateSysModelHasPermission {
        model_type: request.model_type,
        model_id,
        permission_id: permission_id_update,
        scope_type: request.scope_type,
        scope_id,
    };

    match SysModelHasPermissionService::update(&pool, permission_id, payload) {
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
    let permission_id = match DieselUlid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match SysModelHasPermissionService::delete(&pool, permission_id) {
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
pub async fn by_model(
    State(pool): State<DbPool>,
    Path((model_type, model_id)): Path<(String, String)>,
    Query(mut params): Query<QueryParams>,
) -> impl IntoResponse {
    // Add model filters to the query parameters
    params.filter.insert("model_type".to_string(), serde_json::json!(model_type));
    params.filter.insert("model_id".to_string(), serde_json::json!(model_id));

    match <SysModelHasPermission as QueryBuilderService<SysModelHasPermission>>::index(Query(params), &pool) {
        Ok(result) => {
            (StatusCode::OK, ResponseJson(serde_json::json!(result))).into_response()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}