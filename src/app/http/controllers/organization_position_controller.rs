use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};

use crate::app::http::requests::organization_position_requests::{
    CreateJobPositionRequest, UpdateJobPositionRequest, IndexJobPositionRequest, JobPositionsByLevelRequest
};
use crate::app::services::organization_position_service::JobPositionService;
use crate::database::DbPool;

/// List organization positions with filtering, sorting and pagination
#[utoipa::path(
    get,
    path = "/api/organization-positions",
    tag = "Organization Positions",
    summary = "List organization positions",
    description = "Retrieve a paginated list of organization positions with optional filtering by active status, organization position level, name search, and sorting options",
    params(
        ("page" = Option<u32>, Query, description = "Page number (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Items per page (1-100, default: 15)"),
        ("sort_by" = Option<String>, Query, description = "Sort field: name, code, created_at, updated_at"),
        ("sort_direction" = Option<String>, Query, description = "Sort direction: asc, desc"),
        ("is_active" = Option<bool>, Query, description = "Filter by active status"),
        ("organization_position_level_id" = Option<String>, Query, description = "Filter by organization position level ULID"),
        ("name_search" = Option<String>, Query, description = "Search organization positions by name (partial match)")
    ),
    responses(
        (status = 200, description = "Organization positions retrieved successfully"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    request: IndexJobPositionRequest,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers)?;
    // if !JobPositionPolicy::view_any(&user)? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    match JobPositionService::index(&pool, &request) {
        Ok(response) => Ok(Json(json!(response))),
        Err(e) => {
            tracing::error!("Failed to fetch organization positions: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch organization positions"})),
            ))
        }
    }
}

/// Get a specific organization position by ID
#[utoipa::path(
    get,
    path = "/api/organization-positions/{id}",
    tag = "Organization Positions",
    summary = "Get organization position",
    description = "Retrieve a specific organization position by its ULID, including organization position level information",
    params(
        ("id" = String, Path, description = "Organization position ULID")
    ),
    responses(
        (status = 200, description = "Organization position retrieved successfully"),
        (status = 404, description = "Organization position not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn show(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers)?;
    // let organization_position = JobPositionService::find(&pool, &id)?;
    // if !JobPositionPolicy::view(&user, &organization_position)? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    match JobPositionService::show(&pool, &id) {
        Ok(organization_position) => Ok(Json(json!(organization_position))),
        Err(e) => {
            tracing::error!("Failed to fetch organization position {}: {}", id, e);
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Organization position not found"})),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to fetch organization position"})),
                ))
            }
        }
    }
}

/// Create a new organization position
#[utoipa::path(
    post,
    path = "/api/organization-positions",
    tag = "Organization Positions",
    summary = "Create organization position",
    description = "Create a new organization position with name, optional code, organization position level association, and optional description",
    request_body = CreateJobPositionRequest,
    responses(
        (status = 201, description = "Organization position created successfully"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn store(
    State(pool): State<DbPool>,
    request: CreateJobPositionRequest,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers)?;
    // if !JobPositionPolicy::create(&user)? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    match JobPositionService::create(&pool, &request) {
        Ok(organization_position) => Ok(Json(json!(organization_position))),
        Err(e) => {
            tracing::error!("Failed to create organization position: {}", e);
            if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({
                        "message": "The given data was invalid.",
                        "errors": {
                            "name": ["A organization position with this name already exists"],
                            "code": ["A organization position with this code already exists"]
                        }
                    })),
                ))
            } else if e.to_string().contains("foreign key") {
                Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({
                        "message": "The given data was invalid.",
                        "errors": {
                            "organization_position_level_id": ["The specified organization position level does not exist"]
                        }
                    })),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to create organization position"})),
                ))
            }
        }
    }
}

/// Update an existing organization position
#[utoipa::path(
    put,
    path = "/api/organization-positions/{id}",
    tag = "Organization Positions",
    summary = "Update organization position",
    description = "Update an existing organization position's properties including name, code, organization position level, description, and active status",
    params(
        ("id" = String, Path, description = "Organization position ULID")
    ),
    request_body = UpdateJobPositionRequest,
    responses(
        (status = 200, description = "Organization position updated successfully"),
        (status = 404, description = "Organization position not found"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    request: UpdateJobPositionRequest,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers)?;
    // let organization_position = JobPositionService::find(&pool, &id)?;
    // if !JobPositionPolicy::update(&user, &organization_position)? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    match JobPositionService::update(&pool, &id, &request) {
        Ok(organization_position) => Ok(Json(json!(organization_position))),
        Err(e) => {
            tracing::error!("Failed to update organization position {}: {}", id, e);
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Organization position not found"})),
                ))
            } else if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({
                        "message": "The given data was invalid.",
                        "errors": {
                            "name": ["A organization position with this name already exists"],
                            "code": ["A organization position with this code already exists"]
                        }
                    })),
                ))
            } else if e.to_string().contains("foreign key") {
                Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({
                        "message": "The given data was invalid.",
                        "errors": {
                            "organization_position_level_id": ["The specified organization position level does not exist"]
                        }
                    })),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to update organization position"})),
                ))
            }
        }
    }
}

/// Delete a organization position
#[utoipa::path(
    delete,
    path = "/api/organization-positions/{id}",
    tag = "Organization Positions",
    summary = "Delete organization position",
    description = "Delete a organization position by its ULID. This will fail if there are user organizations associated with this position.",
    params(
        ("id" = String, Path, description = "Organization position ULID")
    ),
    responses(
        (status = 200, description = "Organization position deleted successfully"),
        (status = 404, description = "Organization position not found"),
        (status = 422, description = "Cannot delete organization position with associated users"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn destroy(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers)?;
    // let organization_position = JobPositionService::find(&pool, &id)?;
    // if !JobPositionPolicy::delete(&user, &organization_position)? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    match JobPositionService::delete(&pool, &id) {
        Ok(_) => Ok(Json(json!({"message": "Organization position deleted successfully"}))),
        Err(e) => {
            tracing::error!("Failed to delete organization position {}: {}", id, e);
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Organization position not found"})),
                ))
            } else if e.to_string().contains("foreign key") || e.to_string().contains("referenced") {
                Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({"error": "Cannot delete organization position with associated user organizations"})),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to delete organization position"})),
                ))
            }
        }
    }
}

/// Activate a organization position
#[utoipa::path(
    post,
    path = "/api/organization-positions/{id}/activate",
    tag = "Organization Positions",
    summary = "Activate organization position",
    description = "Activate a organization position by setting its is_active status to true",
    params(
        ("id" = String, Path, description = "Organization position ULID")
    ),
    responses(
        (status = 200, description = "Organization position activated successfully"),
        (status = 404, description = "Organization position not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn activate(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers)?;
    // let organization_position = JobPositionService::find(&pool, &id)?;
    // if !JobPositionPolicy::update(&user, &organization_position)? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    match JobPositionService::activate(&pool, &id) {
        Ok(organization_position) => Ok(Json(json!(organization_position))),
        Err(e) => {
            tracing::error!("Failed to activate organization position {}: {}", id, e);
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Organization position not found"})),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to activate organization position"})),
                ))
            }
        }
    }
}

/// Deactivate a organization position
#[utoipa::path(
    post,
    path = "/api/organization-positions/{id}/deactivate",
    tag = "Organization Positions",
    summary = "Deactivate organization position",
    description = "Deactivate a organization position by setting its is_active status to false",
    params(
        ("id" = String, Path, description = "Organization position ULID")
    ),
    responses(
        (status = 200, description = "Organization position deactivated successfully"),
        (status = 404, description = "Organization position not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn deactivate(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers)?;
    // let organization_position = JobPositionService::find(&pool, &id)?;
    // if !JobPositionPolicy::update(&user, &organization_position)? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    match JobPositionService::deactivate(&pool, &id) {
        Ok(organization_position) => Ok(Json(json!(organization_position))),
        Err(e) => {
            tracing::error!("Failed to deactivate organization position {}: {}", id, e);
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Organization position not found"})),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to deactivate organization position"})),
                ))
            }
        }
    }
}

/// Get organization positions by organization position level
#[utoipa::path(
    get,
    path = "/api/organization-position-levels/{organization_position_level_id}/positions",
    tag = "Organization Positions",
    summary = "Get positions by organization position level",
    description = "Retrieve all organization positions for a specific organization position level with optional inclusion of inactive positions",
    params(
        ("organization_position_level_id" = String, Path, description = "Job level ULID"),
        ("include_inactive" = Option<bool>, Query, description = "Include inactive positions (default: false)")
    ),
    responses(
        (status = 200, description = "Organization positions retrieved successfully"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn by_level(
    State(pool): State<DbPool>,
    Path(organization_position_level_id): Path<String>,
    mut request: JobPositionsByLevelRequest,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers)?;
    // if !JobPositionPolicy::view_any(&user)? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    // Set the organization_position_level_id from the path parameter
    request.organization_position_level_id = organization_position_level_id;

    match JobPositionService::by_level(&pool, &request) {
        Ok(positions) => Ok(Json(json!(positions))),
        Err(e) => {
            tracing::error!("Failed to fetch organization positions by level: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch organization positions"})),
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