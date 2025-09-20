use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::Serialize;
use ulid::Ulid;
use sqlx::PgPool;
use std::collections::HashMap;

use crate::app::models::joblevel::{CreateJobLevel, UpdateJobLevel};
use crate::app::services::job_level_service::JobLevelService;
use crate::app::http::requests::{CreateJobLevelRequest, UpdateJobLevelRequest};

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

/// Get all job levels with optional filtering and pagination
///
/// Retrieve a list of all job levels with support for filtering and pagination.
/// You can filter by any field and sort by any column.
///
/// # Query Parameters
/// - `page`: Page number for pagination (default: 1)
/// - `limit`: Number of items per page (default: 10, max: 100)
/// - `sort`: Sort field (default: level)
/// - `direction`: Sort direction - asc or desc (default: asc)
/// - `filter[field]`: Filter by field value
///
/// # Example
/// ```
/// GET /api/job-levels?page=1&limit=10&sort=level&direction=asc&filter[is_active]=true
/// ```
#[utoipa::path(
    get,
    path = "/api/job-levels",
    tag = "Job Levels",
    summary = "List all job levels",
    description = "Get all job levels with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination"),
        ("limit" = Option<u32>, Query, description = "Number of items per page (max 100)"),
        ("sort" = Option<String>, Query, description = "Sort field"),
        ("direction" = Option<String>, Query, description = "Sort direction (asc/desc)"),
    ),
    responses(
        (status = 200, description = "List of job levels", body = Vec<crate::app::models::joblevel::JobLevelResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<PgPool>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    match JobLevelService::list(&pool, params).await {
        Ok(job_levels) => {
            let responses: Vec<_> = job_levels.into_iter().map(|jl| jl.to_response()).collect();
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

/// Get a specific job level by ID
///
/// Retrieve detailed information about a specific job level using its unique identifier.
/// The ID should be a valid ULID format.
///
/// # Path Parameters
/// - `id`: The unique identifier of the job level (ULID format)
///
/// # Example
/// ```
/// GET /api/job-levels/01ARZ3NDEKTSV4RRFFQ69G5FAV
/// ```
#[utoipa::path(
    get,
    path = "/api/job-levels/{id}",
    tag = "Job Levels",
    summary = "Get job level by ID",
    description = "Retrieve a specific job level by its unique identifier",
    params(
        ("id" = String, Path, description = "Job level unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Job level details", body = crate::app::models::joblevel::JobLevelResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Job level not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn show(State(pool): State<PgPool>, Path(id): Path<String>) -> impl IntoResponse {
    let job_level_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match JobLevelService::find_by_id(&pool, job_level_id).await {
        Ok(Some(job_level)) => (StatusCode::OK, ResponseJson(job_level.to_response())).into_response(),
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

/// Create a new job level
///
/// Create a new job level with the provided information. All required fields must be provided
/// and will be validated according to the business rules.
///
/// # Request Body
/// The request must contain a valid CreateJobLevelRequest JSON payload with:
/// - `name`: Job level name (2-100 characters)
/// - `code`: Optional job level code (2-20 characters)
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
    path = "/api/job-levels",
    tag = "Job Levels",
    summary = "Create new job level",
    description = "Create a new job level with the provided information",
    request_body = crate::app::http::requests::CreateJobLevelRequest,
    responses(
        (status = 201, description = "Job level created successfully", body = crate::app::models::joblevel::JobLevelResponse),
        (status = 400, description = "Validation error or bad request", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn store(State(pool): State<PgPool>, request: CreateJobLevelRequest) -> impl IntoResponse {
    let payload = CreateJobLevel {
        name: request.name,
        code: request.code,
        level: request.level,
        description: request.description,
    };

    match JobLevelService::create(&pool, payload).await {
        Ok(job_level) => (StatusCode::CREATED, ResponseJson(job_level.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Update an existing job level
///
/// Update an existing job level with the provided information. All fields are optional
/// for partial updates. Only provided fields will be updated.
///
/// # Path Parameters
/// - `id`: The unique identifier of the job level to update (ULID format)
///
/// # Request Body
/// The request should contain an UpdateJobLevelRequest JSON payload with optional fields:
/// - `name`: Updated job level name (2-100 characters)
/// - `code`: Updated job level code (2-20 characters)
/// - `level`: Updated numeric level ranking (1-20)
/// - `description`: Updated description (max 500 characters)
/// - `is_active`: Updated active status
#[utoipa::path(
    put,
    path = "/api/job-levels/{id}",
    tag = "Job Levels",
    summary = "Update job level",
    description = "Update an existing job level with the provided information",
    params(
        ("id" = String, Path, description = "Job level unique identifier (ULID format)")
    ),
    request_body = crate::app::http::requests::UpdateJobLevelRequest,
    responses(
        (status = 200, description = "Job level updated successfully", body = crate::app::models::joblevel::JobLevelResponse),
        (status = 400, description = "Invalid ID format or validation error", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Job level not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn update(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
    request: UpdateJobLevelRequest,
) -> impl IntoResponse {
    let job_level_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    let payload = UpdateJobLevel {
        name: request.name,
        code: request.code,
        level: request.level,
        description: request.description,
        is_active: request.is_active,
    };

    match JobLevelService::update(&pool, job_level_id, payload).await {
        Ok(job_level) => (StatusCode::OK, ResponseJson(job_level.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Delete a job level
///
/// Permanently delete a job level from the system. This action cannot be undone.
///
/// # Path Parameters
/// - `id`: The unique identifier of the job level to delete (ULID format)
#[utoipa::path(
    delete,
    path = "/api/job-levels/{id}",
    tag = "Job Levels",
    summary = "Delete job level",
    description = "Permanently delete a job level from the system",
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
pub async fn destroy(State(pool): State<PgPool>, Path(id): Path<String>) -> impl IntoResponse {
    let job_level_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match JobLevelService::delete(&pool, job_level_id).await {
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

/// Activate a job level
///
/// Set a job level's active status to true.
///
/// # Path Parameters
/// - `id`: The unique identifier of the job level to activate (ULID format)
#[utoipa::path(
    post,
    path = "/api/job-levels/{id}/activate",
    tag = "Job Levels",
    summary = "Activate job level",
    description = "Set a job level's active status to true",
    params(
        ("id" = String, Path, description = "Job level unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Job level activated successfully", body = crate::app::models::joblevel::JobLevelResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Job level not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn activate(State(pool): State<PgPool>, Path(id): Path<String>) -> impl IntoResponse {
    let job_level_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    // Get current job level and update its active status
    match JobLevelService::find_by_id(&pool, job_level_id).await {
        Ok(Some(_job_level)) => {
            let payload = UpdateJobLevel {
                name: None,
                code: None,
                level: None,
                description: None,
                is_active: Some(true),
            };

            match JobLevelService::update(&pool, job_level_id, payload).await {
                Ok(updated_job_level) => (StatusCode::OK, ResponseJson(updated_job_level.to_response())).into_response(),
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

/// Deactivate a job level
///
/// Set a job level's active status to false.
///
/// # Path Parameters
/// - `id`: The unique identifier of the job level to deactivate (ULID format)
#[utoipa::path(
    post,
    path = "/api/job-levels/{id}/deactivate",
    tag = "Job Levels",
    summary = "Deactivate job level",
    description = "Set a job level's active status to false",
    params(
        ("id" = String, Path, description = "Job level unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Job level deactivated successfully", body = crate::app::models::joblevel::JobLevelResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Job level not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn deactivate(State(pool): State<PgPool>, Path(id): Path<String>) -> impl IntoResponse {
    let job_level_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    // Get current job level and update its active status
    match JobLevelService::find_by_id(&pool, job_level_id).await {
        Ok(Some(_job_level)) => {
            let payload = UpdateJobLevel {
                name: None,
                code: None,
                level: None,
                description: None,
                is_active: Some(false),
            };

            match JobLevelService::update(&pool, job_level_id, payload).await {
                Ok(updated_job_level) => (StatusCode::OK, ResponseJson(updated_job_level.to_response())).into_response(),
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