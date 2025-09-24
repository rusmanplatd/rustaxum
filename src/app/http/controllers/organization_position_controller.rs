use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};

use crate::app::http::requests::organization_position_requests::{
    CreateOrganizationPositionRequest, UpdateOrganizationPositionRequest
};
use crate::app::services::organization_position_service::OrganizationPositionService;
use crate::app::query_builder::{QueryParams, QueryBuilderService};
use crate::app::models::organization_position::OrganizationPosition;
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
    ),
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Sort field and direction. Available fields: id, name, level_id, organization_id, status, created_at, updated_at (prefix with '-' for descending)"),
        ("include" = Option<String>, Query, description = "Comma-separated list of relationships to include. Available: level, organization"),
        ("filter" = Option<serde_json::Value>, Query, description = "Filter parameters. Available filters: name, level_id, organization_id, status (e.g., filter[name]=Manager, filter[status]=active)"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to select. Available: id, name, level_id, organization_id, status, created_at, updated_at"),
        ("cursor" = Option<String>, Query, description = "Cursor for cursor-based pagination"),
        ("pagination_type" = Option<String>, Query, description = "Pagination type: 'offset' or 'cursor' (default: cursor)"),
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match <OrganizationPosition as QueryBuilderService<OrganizationPosition>>::index(Query(params), &pool) {
        Ok(result) => Ok(Json(serde_json::json!(result))),
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
    // let organization_position = OrganizationPositionService::find(&pool, &id)?;
    // if !OrganizationPositionPolicy::view(&user, &organization_position)? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    // TODO: Extract user_id from auth context when available
    let user_id = None; // Replace with actual user extraction

    match OrganizationPositionService::show(&pool, &id, user_id).await {
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
    request_body = CreateOrganizationPositionRequest,
    responses(
        (status = 201, description = "Organization position created successfully"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn store(
    State(pool): State<DbPool>,
    request: CreateOrganizationPositionRequest,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers)?;
    // if !OrganizationPositionPolicy::create(&user)? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    // TODO: Extract user_id from auth context when available
    let created_by = None; // Replace with actual user extraction

    match OrganizationPositionService::create(&pool, &request, created_by).await {
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
    request_body = UpdateOrganizationPositionRequest,
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
    request: UpdateOrganizationPositionRequest,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Authorization check - uncomment when auth middleware is ready
    // let user = auth::get_user(&state, &headers)?;
    // let organization_position = OrganizationPositionService::find(&pool, &id)?;
    // if !OrganizationPositionPolicy::update(&user, &organization_position)? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    // TODO: Extract user_id from auth context when available
    let updated_by = None; // Replace with actual user extraction

    match OrganizationPositionService::update(&pool, &id, &request, updated_by).await {
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
    // let organization_position = OrganizationPositionService::find(&pool, &id)?;
    // if !OrganizationPositionPolicy::delete(&user, &organization_position)? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    // TODO: Extract user_id from auth context when available
    let deleted_by = None; // Replace with actual user extraction

    match OrganizationPositionService::delete(&pool, &id, deleted_by).await {
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
    // let organization_position = OrganizationPositionService::find(&pool, &id)?;
    // if !OrganizationPositionPolicy::update(&user, &organization_position)? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    // TODO: Extract user_id from auth context when available
    let activated_by = None; // Replace with actual user extraction

    match OrganizationPositionService::activate(&pool, &id, activated_by).await {
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
    // let organization_position = OrganizationPositionService::find(&pool, &id)?;
    // if !OrganizationPositionPolicy::update(&user, &organization_position)? {
    //     return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Insufficient permissions"}))));
    // }

    // TODO: Extract user_id from auth context when available
    let deactivated_by = None; // Replace with actual user extraction

    match OrganizationPositionService::deactivate(&pool, &id, deactivated_by).await {
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
        ("organization_position_level_id" = String, Path, description = "Organization position level unique identifier (ULID format)"),
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Sort field and direction. Available fields: id, name, status, created_at, updated_at (prefix with '-' for descending)"),
        ("include" = Option<String>, Query, description = "Comma-separated list of relationships to include. Available: organization"),
        ("filter" = Option<serde_json::Value>, Query, description = "Filter parameters. Available filters: name, status (e.g., filter[name]=Manager, filter[status]=active)"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to select. Available: id, name, status, created_at, updated_at"),
        ("cursor" = Option<String>, Query, description = "Cursor for cursor-based pagination"),
        ("pagination_type" = Option<String>, Query, description = "Pagination type: 'offset' or 'cursor' (default: cursor)"),
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
    Query(mut params): Query<QueryParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Add organization_position_level_id filter to the query parameters
    params.filter.insert("organization_position_level_id".to_string(), serde_json::json!(organization_position_level_id));

    match <OrganizationPosition as QueryBuilderService<OrganizationPosition>>::index(Query(params), &pool) {
        Ok(result) => Ok(Json(serde_json::json!(result))),
        Err(e) => {
            tracing::error!("Failed to fetch organization positions by level: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch organization positions"})),
            ))
        }
    }
}

pub struct OrganizationPositionController;

impl OrganizationPositionController {
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