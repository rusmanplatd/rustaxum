use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};

use crate::app::http::requests::job_level_requests::{
    CreateJobLevelRequest, UpdateJobLevelRequest, IndexJobLevelRequest
};
use crate::app::services::job_level_service::JobLevelService;
use crate::AppState;

pub struct JobLevelController;

#[utoipa::path(
    get,
    path = "/api/job-levels",
    tag = "Job Levels",
    summary = "List job levels",
    description = "Retrieve a paginated list of job levels with optional filtering by active status, level ranges, and sorting options",
    params(
        ("page" = Option<u32>, Query, description = "Page number (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Items per page (1-100, default: 15)"),
        ("sort_by" = Option<String>, Query, description = "Sort field: name, code, level, created_at, updated_at"),
        ("sort_direction" = Option<String>, Query, description = "Sort direction: asc, desc"),
        ("is_active" = Option<bool>, Query, description = "Filter by active status"),
        ("min_level" = Option<i32>, Query, description = "Filter by minimum level (1-20)"),
        ("max_level" = Option<i32>, Query, description = "Filter by maximum level (1-20)")
    ),
    responses(
        (status = 200, description = "Job levels retrieved successfully"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn index(
    State(state): State<AppState>,
    Query(mut request): Query<IndexJobLevelRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        // Authorization check - uncomment when auth middleware is ready
        // let user = auth::get_user(&state, &headers).await?;
        // if !JobLevelPolicy::view_any(&user).await? {
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

        match JobLevelService::index(&state.db, &request).await {
            Ok(response) => Ok(Json(json!(response))),
            Err(e) => {
                tracing::error!("Failed to fetch job levels: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to fetch job levels"})),
                ))
            }
        }
    }

#[utoipa::path(
    get,
    path = "/api/job-levels/{id}",
    tag = "Job Levels",
    summary = "Get job level",
    description = "Retrieve a specific job level by its ULID",
    params(
        ("id" = String, Path, description = "Job level ULID")
    ),
    responses(
        (status = 200, description = "Job level retrieved successfully"),
        (status = 404, description = "Job level not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        // Authorization check - uncomment when auth middleware is ready
        // let user = auth::get_user(&state, &headers).await?;
        // let job_level = JobLevelService::find(&state.db, &id).await?;
        // if !JobLevelPolicy::view(&user, &job_level).await? {
        //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
        // }

        match JobLevelService::show(&state.db, &id).await {
            Ok(job_level) => Ok(Json(json!(job_level))),
            Err(e) => {
                tracing::error!("Failed to fetch job level {}: {}", id, e);
                if e.to_string().contains("not found") {
                    Err((
                        StatusCode::NOT_FOUND,
                        Json(json!({"error": "Job level not found"})),
                    ))
                } else {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to fetch job level"})),
                    ))
                }
            }
        }
    }

#[utoipa::path(
    post,
    path = "/api/job-levels",
    tag = "Job Levels",
    summary = "Create job level",
    description = "Create a new job level with name, code, level number, and optional description",
    request_body = CreateJobLevelRequest,
    responses(
        (status = 201, description = "Job level created successfully"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn store(
    State(state): State<AppState>,
    Json(request): Json<CreateJobLevelRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        // Authorization check - uncomment when auth middleware is ready
        // let user = auth::get_user(&state, &headers).await?;
        // if !JobLevelPolicy::create(&user).await? {
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

        match JobLevelService::create(&state.db, &request).await {
            Ok(job_level) => Ok(Json(json!(job_level))),
            Err(e) => {
                tracing::error!("Failed to create job level: {}", e);
                if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                    Err((
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(json!({
                            "message": "The given data was invalid.",
                            "errors": {
                                "level": ["A job level with this level number already exists"],
                                "code": ["A job level with this code already exists"]
                            }
                        })),
                    ))
                } else {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to create job level"})),
                    ))
                }
            }
        }
    }

#[utoipa::path(
    put,
    path = "/api/job-levels/{id}",
    tag = "Job Levels",
    summary = "Update job level",
    description = "Update an existing job level's properties including name, code, level, description, and active status",
    params(
        ("id" = String, Path, description = "Job level ULID")
    ),
    request_body = UpdateJobLevelRequest,
    responses(
        (status = 200, description = "Job level updated successfully"),
        (status = 404, description = "Job level not found"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateJobLevelRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        // Authorization check - uncomment when auth middleware is ready
        // let user = auth::get_user(&state, &headers).await?;
        // let job_level = JobLevelService::find(&state.db, &id).await?;
        // if !JobLevelPolicy::update(&user, &job_level).await? {
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

        match JobLevelService::update(&state.db, &id, &request).await {
            Ok(job_level) => Ok(Json(json!(job_level))),
            Err(e) => {
                tracing::error!("Failed to update job level {}: {}", id, e);
                if e.to_string().contains("not found") {
                    Err((
                        StatusCode::NOT_FOUND,
                        Json(json!({"error": "Job level not found"})),
                    ))
                } else if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                    Err((
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(json!({
                            "message": "The given data was invalid.",
                            "errors": {
                                "level": ["A job level with this level number already exists"],
                                "code": ["A job level with this code already exists"]
                            }
                        })),
                    ))
                } else {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to update job level"})),
                    ))
                }
            }
        }
    }

#[utoipa::path(
    delete,
    path = "/api/job-levels/{id}",
    tag = "Job Levels",
    summary = "Delete job level",
    description = "Delete a job level by its ULID. This will fail if there are job positions associated with this level.",
    params(
        ("id" = String, Path, description = "Job level ULID")
    ),
    responses(
        (status = 200, description = "Job level deleted successfully"),
        (status = 404, description = "Job level not found"),
        (status = 422, description = "Cannot delete job level with associated positions"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn destroy(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        // Authorization check - uncomment when auth middleware is ready
        // let user = auth::get_user(&state, &headers).await?;
        // let job_level = JobLevelService::find(&state.db, &id).await?;
        // if !JobLevelPolicy::delete(&user, &job_level).await? {
        //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
        // }

        match JobLevelService::delete(&state.db, &id).await {
            Ok(_) => Ok(Json(json!({"message": "Job level deleted successfully"}))),
            Err(e) => {
                tracing::error!("Failed to delete job level {}: {}", id, e);
                if e.to_string().contains("not found") {
                    Err((
                        StatusCode::NOT_FOUND,
                        Json(json!({"error": "Job level not found"})),
                    ))
                } else if e.to_string().contains("foreign key") || e.to_string().contains("referenced") {
                    Err((
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(json!({"error": "Cannot delete job level with associated job positions"})),
                    ))
                } else {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to delete job level"})),
                    ))
                }
            }
        }
    }

#[utoipa::path(
    post,
    path = "/api/job-levels/{id}/activate",
    tag = "Job Levels",
    summary = "Activate job level",
    description = "Activate a job level by setting its is_active status to true",
    params(
        ("id" = String, Path, description = "Job level ULID")
    ),
    responses(
        (status = 200, description = "Job level activated successfully"),
        (status = 404, description = "Job level not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn activate(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        // Authorization check - uncomment when auth middleware is ready
        // let user = auth::get_user(&state, &headers).await?;
        // let job_level = JobLevelService::find(&state.db, &id).await?;
        // if !JobLevelPolicy::update(&user, &job_level).await? {
        //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
        // }

        match JobLevelService::activate(&state.db, &id).await {
            Ok(job_level) => Ok(Json(json!(job_level))),
            Err(e) => {
                tracing::error!("Failed to activate job level {}: {}", id, e);
                if e.to_string().contains("not found") {
                    Err((
                        StatusCode::NOT_FOUND,
                        Json(json!({"error": "Job level not found"})),
                    ))
                } else {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to activate job level"})),
                    ))
                }
            }
        }
    }

#[utoipa::path(
    post,
    path = "/api/job-levels/{id}/deactivate",
    tag = "Job Levels",
    summary = "Deactivate job level",
    description = "Deactivate a job level by setting its is_active status to false",
    params(
        ("id" = String, Path, description = "Job level ULID")
    ),
    responses(
        (status = 200, description = "Job level deactivated successfully"),
        (status = 404, description = "Job level not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn deactivate(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        // Authorization check - uncomment when auth middleware is ready
        // let user = auth::get_user(&state, &headers).await?;
        // let job_level = JobLevelService::find(&state.db, &id).await?;
        // if !JobLevelPolicy::update(&user, &job_level).await? {
        //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
        // }

        match JobLevelService::deactivate(&state.db, &id).await {
            Ok(job_level) => Ok(Json(json!(job_level))),
            Err(e) => {
                tracing::error!("Failed to deactivate job level {}: {}", id, e);
                if e.to_string().contains("not found") {
                    Err((
                        StatusCode::NOT_FOUND,
                        Json(json!({"error": "Job level not found"})),
                    ))
                } else {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to deactivate job level"})),
                    ))
                }
            }
        }
    }

impl JobLevelController {}