use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::Serialize;
use ulid::Ulid;
use crate::database::DbPool;
use std::collections::HashMap;

use crate::app::models::organization_position_level::{CreateOrganizationPositionLevel, UpdateOrganizationPositionLevel};
use crate::app::services::organization_position_level_service::OrganizationPositionLevelService;
use crate::app::http::requests::{CreateOrganizationPositionLevelRequest, UpdateOrganizationPositionLevelRequest};

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

#[utoipa::path(
    get,
    path = "/api/organization-position-levels",
    tag = "Job Levels",
    summary = "List all organization position levels",
    description = "Get all organization position levels with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination"),
        ("limit" = Option<u32>, Query, description = "Number of items per page (max 100)"),
        ("sort" = Option<String>, Query, description = "Sort field"),
        ("direction" = Option<String>, Query, description = "Sort direction (asc/desc)"),
    ),
    responses(
        (status = 200, description = "List of organization position levels", body = Vec<crate::app::models::organization_position_level::OrganizationPositionLevelResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    match OrganizationPositionLevelService::list(&pool, params) {
        Ok(OrganizationPositionLevel) => {
            let responses: Vec<_> = OrganizationPositionLevel.into_iter().map(|jl| jl.to_response()).collect();
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
    path = "/api/organization-position-levels/{id}",
    tag = "Job Levels",
    summary = "Get organization position level by ID",
    description = "Retrieve a specific organization position level by its unique identifier",
    params(
        ("id" = String, Path, description = "Job level unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Job level details", body = crate::app::models::organization_position_level::OrganizationPositionLevelResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Job level not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn show(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    let organization_position_level_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match OrganizationPositionLevelService::find_by_id(&pool, organization_position_level_id) {
        Ok(Some(organization_position_level)) => (StatusCode::OK, ResponseJson(organization_position_level.to_response())).into_response(),
        Ok(None) => {
            let error = ErrorResponse {
                error: "Job level not found".to_string(),
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

/// Create a new organization position level
///
/// Create a new organization position level with the provided information. All required fields must be provided
/// and will be validated according to the business rules.
///
/// # Request Body
/// The request must contain a valid CreateOrganizationPositionLevelRequest JSON payload with:
/// - `name`: Job level name (2-100 characters)
/// - `code`: Optional organization position level code (2-20 characters)
/// - `level`: Numeric level ranking (1-20)
/// - `description`: Optional description (max 500 characters)
///
/// # Example
/// ```json
/// {
///   "name": "Senior Manager",
///   "code": "SM",
///   "level": 5,
///   "description": "Senior management position with team leadership responsibilities"
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/organization-position-levels",
    tag = "Job Levels",
    summary = "Create new organization position level",
    description = "Create a new organization position level with the provided information",
    request_body = crate::app::http::requests::CreateOrganizationPositionLevelRequest,
    responses(
        (status = 201, description = "Job level created successfully", body = crate::app::models::organization_position_level::OrganizationPositionLevelResponse),
        (status = 400, description = "Validation error or bad request", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn store(State(pool): State<DbPool>, request: CreateOrganizationPositionLevelRequest) -> impl IntoResponse {
    let payload = CreateOrganizationPositionLevel {
        name: request.name,
        code: request.code,
        level: request.level,
        description: request.description,
    };

    match OrganizationPositionLevelService::create(&pool, payload) {
        Ok(organization_position_level) => (StatusCode::CREATED, ResponseJson(organization_position_level.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Update an existing organization position level
///
/// Update an existing organization position level with the provided information. All fields are optional
/// for partial updates. Only provided fields will be updated.
///
/// # Path Parameters
/// - `id`: The unique identifier of the organization position level to update (ULID format)
///
/// # Request Body
/// The request should contain an UpdateOrganizationPositionLevelRequest JSON payload with optional fields:
/// - `name`: Updated organization position level name (2-100 characters)
/// - `code`: Updated organization position level code (2-20 characters)
/// - `level`: Updated numeric level ranking (1-20)
/// - `description`: Updated description (max 500 characters)
/// - `is_active`: Updated active status
#[utoipa::path(
    put,
    path = "/api/organization-position-levels/{id}",
    tag = "Job Levels",
    summary = "Update organization position level",
    description = "Update an existing organization position level with the provided information",
    params(
        ("id" = String, Path, description = "Job level unique identifier (ULID format)")
    ),
    request_body = crate::app::http::requests::UpdateOrganizationPositionLevelRequest,
    responses(
        (status = 200, description = "Job level updated successfully", body = crate::app::models::organization_position_level::OrganizationPositionLevelResponse),
        (status = 400, description = "Invalid ID format or validation error", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Job level not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    request: UpdateOrganizationPositionLevelRequest,
) -> impl IntoResponse {
    let organization_position_level_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    let payload = UpdateOrganizationPositionLevel {
        name: request.name,
        code: request.code,
        level: request.level,
        description: request.description,
        is_active: request.is_active,
    };

    match OrganizationPositionLevelService::update(&pool, organization_position_level_id, payload) {
        Ok(organization_position_level) => (StatusCode::OK, ResponseJson(organization_position_level.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Delete a organization position level
///
/// Permanently delete a organization position level from the system. This action cannot be undone.
///
/// # Path Parameters
/// - `id`: The unique identifier of the organization position level to delete (ULID format)
#[utoipa::path(
    delete,
    path = "/api/organization-position-levels/{id}",
    tag = "Job Levels",
    summary = "Delete organization position level",
    description = "Permanently delete a organization position level from the system",
    params(
        ("id" = String, Path, description = "Job level unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Job level deleted successfully", body = crate::app::docs::MessageResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Job level not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn destroy(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    let organization_position_level_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match OrganizationPositionLevelService::delete(&pool, organization_position_level_id) {
        Ok(_) => {
            let message = MessageResponse {
                message: "Job level deleted successfully".to_string(),
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

/// Activate a organization position level
///
/// Set a organization position level's active status to true.
///
/// # Path Parameters
/// - `id`: The unique identifier of the organization position level to activate (ULID format)
#[utoipa::path(
    post,
    path = "/api/organization-position-levels/{id}/activate",
    tag = "Job Levels",
    summary = "Activate organization position level",
    description = "Set a organization position level's active status to true",
    params(
        ("id" = String, Path, description = "Job level unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Job level activated successfully", body = crate::app::models::organization_position_level::OrganizationPositionLevelResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Job level not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn activate(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    let organization_position_level_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    // Get current organization position level and update its active status
    match OrganizationPositionLevelService::find_by_id(&pool, organization_position_level_id) {
        Ok(Some(_organization_position_level)) => {
            let payload = UpdateOrganizationPositionLevel {
                name: None,
                code: None,
                level: None,
                description: None,
                is_active: Some(true),
            };

            match OrganizationPositionLevelService::update(&pool, organization_position_level_id, payload) {
                Ok(updated_organization_position_level) => (StatusCode::OK, ResponseJson(updated_organization_position_level.to_response())).into_response(),
                Err(e) => {
                    let error = ErrorResponse {
                        error: e.to_string(),
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
                }
            }
        }
        Ok(None) => {
            let error = ErrorResponse {
                error: "Job level not found".to_string(),
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

/// Deactivate a organization position level
///
/// Set a organization position level's active status to false.
///
/// # Path Parameters
/// - `id`: The unique identifier of the organization position level to deactivate (ULID format)
#[utoipa::path(
    post,
    path = "/api/organization-position-levels/{id}/deactivate",
    tag = "Job Levels",
    summary = "Deactivate organization position level",
    description = "Set a organization position level's active status to false",
    params(
        ("id" = String, Path, description = "Job level unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Job level deactivated successfully", body = crate::app::models::organization_position_level::OrganizationPositionLevelResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Job level not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn deactivate(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    let organization_position_level_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    // Get current organization position level and update its active status
    match OrganizationPositionLevelService::find_by_id(&pool, organization_position_level_id) {
        Ok(Some(_organization_position_level)) => {
            let payload = UpdateOrganizationPositionLevel {
                name: None,
                code: None,
                level: None,
                description: None,
                is_active: Some(false),
            };

            match OrganizationPositionLevelService::update(&pool, organization_position_level_id, payload) {
                Ok(updated_organization_position_level) => (StatusCode::OK, ResponseJson(updated_organization_position_level.to_response())).into_response(),
                Err(e) => {
                    let error = ErrorResponse {
                        error: e.to_string(),
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
                }
            }
        }
        Ok(None) => {
            let error = ErrorResponse {
                error: "Job level not found".to_string(),
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