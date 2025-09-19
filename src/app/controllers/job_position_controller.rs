use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use utoipa::path;

use crate::app::http::requests::job_position_requests::{
    CreateJobPositionRequest, UpdateJobPositionRequest, IndexJobPositionRequest, JobPositionsByLevelRequest
};
use crate::app::services::job_position_service::JobPositionService;
use crate::AppState;

/// List job positions with filtering, sorting and pagination
#[utoipa::path(
    get,
    path = "/api/job-positions",
    tag = "Job Positions",
    summary = "List job positions",
    description = "Retrieve a paginated list of job positions with optional filtering by active status, job level, name search, and sorting options",
    params(
        ("page" = Option<u32>, Query, description = "Page number (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Items per page (1-100, default: 15)"),
        ("sort_by" = Option<String>, Query, description = "Sort field: name, code, created_at, updated_at"),
        ("sort_direction" = Option<String>, Query, description = "Sort direction: asc, desc"),
        ("is_active" = Option<bool>, Query, description = "Filter by active status"),
        ("job_level_id" = Option<String>, Query, description = "Filter by job level ULID"),
        ("name_search" = Option<String>, Query, description = "Search job positions by name (partial match)")
    ),
    responses(
        (status = 200, description = "Job positions retrieved successfully"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn index(
    State(state): State<AppState>,
    Query(mut request): Query<IndexJobPositionRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers).await?;
    // if !JobPositionPolicy::view_any(&user).await? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    match request.validate().await {
        Ok(_) => {}
        Err(errors) => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "message": "The given data was invalid.",
                    "errors": errors
                })),
            ));
        }
    }

    match JobPositionService::index(&state.db, &request).await {
        Ok(response) => Ok(Json(json!(response))),
        Err(e) => {
            tracing::error!("Failed to fetch job positions: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch job positions"})),
            ))
        }
    }
}

/// Get a specific job position by ID
#[utoipa::path(
    get,
    path = "/api/job-positions/{id}",
    tag = "Job Positions",
    summary = "Get job position",
    description = "Retrieve a specific job position by its ULID, including job level information",
    params(
        ("id" = String, Path, description = "Job position ULID")
    ),
    responses(
        (status = 200, description = "Job position retrieved successfully"),
        (status = 404, description = "Job position not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers).await?;
    // let job_position = JobPositionService::find(&state.db, &id).await?;
    // if !JobPositionPolicy::view(&user, &job_position).await? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    match JobPositionService::show(&state.db, &id).await {
        Ok(job_position) => Ok(Json(json!(job_position))),
        Err(e) => {
            tracing::error!("Failed to fetch job position {}: {}", id, e);
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Job position not found"})),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to fetch job position"})),
                ))
            }
        }
    }
}

/// Create a new job position
#[utoipa::path(
    post,
    path = "/api/job-positions",
    tag = "Job Positions",
    summary = "Create job position",
    description = "Create a new job position with name, optional code, job level association, and optional description",
    request_body = CreateJobPositionRequest,
    responses(
        (status = 201, description = "Job position created successfully"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn store(
    State(state): State<AppState>,
    Json(request): Json<CreateJobPositionRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers).await?;
    // if !JobPositionPolicy::create(&user).await? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    match request.validate().await {
        Ok(_) => {}
        Err(errors) => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "message": "The given data was invalid.",
                    "errors": errors
                })),
            ));
        }
    }

    match JobPositionService::create(&state.db, &request).await {
        Ok(job_position) => Ok(Json(json!(job_position))),
        Err(e) => {
            tracing::error!("Failed to create job position: {}", e);
            if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({
                        "message": "The given data was invalid.",
                        "errors": {
                            "name": ["A job position with this name already exists"],
                            "code": ["A job position with this code already exists"]
                        }
                    })),
                ))
            } else if e.to_string().contains("foreign key") {
                Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({
                        "message": "The given data was invalid.",
                        "errors": {
                            "job_level_id": ["The specified job level does not exist"]
                        }
                    })),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to create job position"})),
                ))
            }
        }
    }
}

/// Update an existing job position
#[utoipa::path(
    put,
    path = "/api/job-positions/{id}",
    tag = "Job Positions",
    summary = "Update job position",
    description = "Update an existing job position's properties including name, code, job level, description, and active status",
    params(
        ("id" = String, Path, description = "Job position ULID")
    ),
    request_body = UpdateJobPositionRequest,
    responses(
        (status = 200, description = "Job position updated successfully"),
        (status = 404, description = "Job position not found"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateJobPositionRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers).await?;
    // let job_position = JobPositionService::find(&state.db, &id).await?;
    // if !JobPositionPolicy::update(&user, &job_position).await? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    match request.validate().await {
        Ok(_) => {}
        Err(errors) => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "message": "The given data was invalid.",
                    "errors": errors
                })),
            ));
        }
    }

    match JobPositionService::update(&state.db, &id, &request).await {
        Ok(job_position) => Ok(Json(json!(job_position))),
        Err(e) => {
            tracing::error!("Failed to update job position {}: {}", id, e);
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Job position not found"})),
                ))
            } else if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({
                        "message": "The given data was invalid.",
                        "errors": {
                            "name": ["A job position with this name already exists"],
                            "code": ["A job position with this code already exists"]
                        }
                    })),
                ))
            } else if e.to_string().contains("foreign key") {
                Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({
                        "message": "The given data was invalid.",
                        "errors": {
                            "job_level_id": ["The specified job level does not exist"]
                        }
                    })),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to update job position"})),
                ))
            }
        }
    }
}

/// Delete a job position
#[utoipa::path(
    delete,
    path = "/api/job-positions/{id}",
    tag = "Job Positions",
    summary = "Delete job position",
    description = "Delete a job position by its ULID. This will fail if there are user organizations associated with this position.",
    params(
        ("id" = String, Path, description = "Job position ULID")
    ),
    responses(
        (status = 200, description = "Job position deleted successfully"),
        (status = 404, description = "Job position not found"),
        (status = 422, description = "Cannot delete job position with associated users"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn destroy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers).await?;
    // let job_position = JobPositionService::find(&state.db, &id).await?;
    // if !JobPositionPolicy::delete(&user, &job_position).await? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    match JobPositionService::delete(&state.db, &id).await {
        Ok(_) => Ok(Json(json!({"message": "Job position deleted successfully"}))),
        Err(e) => {
            tracing::error!("Failed to delete job position {}: {}", id, e);
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Job position not found"})),
                ))
            } else if e.to_string().contains("foreign key") || e.to_string().contains("referenced") {
                Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({"error": "Cannot delete job position with associated user organizations"})),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to delete job position"})),
                ))
            }
        }
    }
}

/// Activate a job position
#[utoipa::path(
    post,
    path = "/api/job-positions/{id}/activate",
    tag = "Job Positions",
    summary = "Activate job position",
    description = "Activate a job position by setting its is_active status to true",
    params(
        ("id" = String, Path, description = "Job position ULID")
    ),
    responses(
        (status = 200, description = "Job position activated successfully"),
        (status = 404, description = "Job position not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn activate(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers).await?;
    // let job_position = JobPositionService::find(&state.db, &id).await?;
    // if !JobPositionPolicy::update(&user, &job_position).await? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    match JobPositionService::activate(&state.db, &id).await {
        Ok(job_position) => Ok(Json(json!(job_position))),
        Err(e) => {
            tracing::error!("Failed to activate job position {}: {}", id, e);
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Job position not found"})),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to activate job position"})),
                ))
            }
        }
    }
}

/// Deactivate a job position
#[utoipa::path(
    post,
    path = "/api/job-positions/{id}/deactivate",
    tag = "Job Positions",
    summary = "Deactivate job position",
    description = "Deactivate a job position by setting its is_active status to false",
    params(
        ("id" = String, Path, description = "Job position ULID")
    ),
    responses(
        (status = 200, description = "Job position deactivated successfully"),
        (status = 404, description = "Job position not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn deactivate(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers).await?;
    // let job_position = JobPositionService::find(&state.db, &id).await?;
    // if !JobPositionPolicy::update(&user, &job_position).await? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    match JobPositionService::deactivate(&state.db, &id).await {
        Ok(job_position) => Ok(Json(json!(job_position))),
        Err(e) => {
            tracing::error!("Failed to deactivate job position {}: {}", id, e);
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Job position not found"})),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to deactivate job position"})),
                ))
            }
        }
    }
}

/// Get job positions by job level
#[utoipa::path(
    get,
    path = "/api/job-levels/{job_level_id}/positions",
    tag = "Job Positions",
    summary = "Get positions by job level",
    description = "Retrieve all job positions for a specific job level with optional inclusion of inactive positions",
    params(
        ("job_level_id" = String, Path, description = "Job level ULID"),
        ("include_inactive" = Option<bool>, Query, description = "Include inactive positions (default: false)")
    ),
    responses(
        (status = 200, description = "Job positions retrieved successfully"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn by_level(
    State(state): State<AppState>,
    Path(job_level_id): Path<String>,
    Query(mut request): Query<JobPositionsByLevelRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers).await?;
    // if !JobPositionPolicy::view_any(&user).await? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    // Set the job_level_id from the path parameter
    request.job_level_id = job_level_id;

    match request.validate().await {
        Ok(_) => {}
        Err(errors) => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "message": "The given data was invalid.",
                    "errors": errors
                })),
            ));
        }
    }

    match JobPositionService::by_level(&state.db, &request).await {
        Ok(positions) => Ok(Json(json!(positions))),
        Err(e) => {
            tracing::error!("Failed to fetch job positions by level: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch job positions"})),
            ))
        }
    }
}

pub struct JobPositionController;

impl JobPositionController {
    pub fn index() -> &'static str {
        "index"
    }

    pub fn show() -> &'static str {
        "show"
    }

    pub fn store() -> &'static str {
        "store"
    }

    pub fn update() -> &'static str {
        "update"
    }

    pub fn destroy() -> &'static str {
        "destroy"
    }

    pub fn activate() -> &'static str {
        "activate"
    }

    pub fn deactivate() -> &'static str {
        "deactivate"
    }

    pub fn by_level() -> &'static str {
        "by_level"
    }
}