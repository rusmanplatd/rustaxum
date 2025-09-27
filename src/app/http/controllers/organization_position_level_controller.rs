use axum::{
    extract::{State, Path, Query},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json as ResponseJson},
};
use serde::Serialize;
use ulid::Ulid;
use crate::database::DbPool;

use crate::app::models::organization_position_level::{CreateOrganizationPositionLevel, UpdateOrganizationPositionLevel, OrganizationPositionLevel};
use crate::app::services::organization_position_level_service::OrganizationPositionLevelService;
use crate::app::http::requests::{CreateOrganizationPositionLevelRequest, UpdateOrganizationPositionLevelRequest};
use crate::app::query_builder::{QueryParams, QueryBuilderService};

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
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Sort field and direction. Available fields: id, name, code, level, organization_id, is_active, created_at, updated_at (prefix with '-' for descending)"),
        ("include" = Option<String>, Query, description = "Comma-separated list of relationships to include. Available: positions, createdBy, updatedBy, deletedBy, createdBy.organizations.position.level, updatedBy.organizations.position.level, deletedBy.organizations.position.level"),
        ("filter" = Option<serde_json::Value>, Query, description = "Filter parameters. Available filters: name, code, level, organization_id, is_active (e.g., filter[name]=Manager, filter[is_active]=true)"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to select. Available: id, name, code, level, organization_id, is_active, created_at, updated_at"),
        ("cursor" = Option<String>, Query, description = "Cursor for cursor-based pagination"),
        ("pagination_type" = Option<String>, Query, description = "Pagination type: 'offset' or 'cursor' (default: cursor)"),
    ),
    responses(
        (status = 200, description = "List of organization position levels", body = Vec<crate::app::models::organization_position_level::OrganizationPositionLevelResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    match <OrganizationPositionLevel as QueryBuilderService<OrganizationPositionLevel>>::index(Query(params), &pool) {
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
pub async fn show(State(pool): State<DbPool>, Path(id): Path<String>, headers: HeaderMap) -> impl IntoResponse {
    let organization_position_level_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    // Extract user ID from authentication context
    let user_id = crate::app::utils::token_utils::TokenUtils::extract_user_id_from_headers(&headers);
    let user_id_str = user_id.as_ref().map(|id| id.to_string());
    let user_id_ref = user_id_str.as_deref();

    match OrganizationPositionLevelService::find_by_id(&pool, &organization_position_level_id.to_string(), user_id_ref).await {
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
pub async fn store(State(pool): State<DbPool>, headers: HeaderMap, request: CreateOrganizationPositionLevelRequest) -> impl IntoResponse {
    let payload = CreateOrganizationPositionLevel {
        organization_id: request.organization_id,
        code: request.code,
        name: request.name,
        description: request.description,
        level: request.level,
    };

    // Extract user ID from authentication context
    let user_id = crate::app::utils::token_utils::TokenUtils::extract_user_id_from_headers(&headers);
    let user_id_str = user_id.as_ref().map(|id| id.to_string());
    let user_id_ref = user_id_str.as_deref();

    match OrganizationPositionLevelService::create(&pool, payload, user_id_ref).await {
        Ok(organization_position_level) => (StatusCode::CREATED, ResponseJson(organization_position_level.to_response())).into_response(),
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
    headers: HeaderMap,
    request: UpdateOrganizationPositionLevelRequest,
) -> impl IntoResponse {
    let payload = UpdateOrganizationPositionLevel {
        organization_id: request.organization_id,
        code: request.code,
        name: request.name,
        description: request.description,
        level: request.level,
        is_active: request.is_active,
    };

    // Extract user ID from authentication context
    let user_id = crate::app::utils::token_utils::TokenUtils::extract_user_id_from_headers(&headers);
    let user_id_str = user_id.as_ref().map(|id| id.to_string());
    let user_id_ref = user_id_str.as_deref();

    match OrganizationPositionLevelService::update(&pool, id, payload, user_id_ref).await {
        Ok(organization_position_level) => (StatusCode::OK, ResponseJson(organization_position_level.to_response())).into_response(),
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
pub async fn destroy(State(pool): State<DbPool>, Path(id): Path<String>, headers: HeaderMap) -> impl IntoResponse {
    let organization_position_level_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    // Extract user ID from authentication context
    let user_id = crate::app::utils::token_utils::TokenUtils::extract_user_id_from_headers(&headers);
    let user_id_str = user_id.as_ref().map(|id| id.to_string());
    let user_id_ref = user_id_str.as_deref();

    match OrganizationPositionLevelService::delete(&pool, organization_position_level_id.to_string(), user_id_ref).await {
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
pub async fn activate(State(pool): State<DbPool>, Path(id): Path<String>, headers: HeaderMap) -> impl IntoResponse {
    // Get current organization position level and update its active status
    // Extract user ID from authentication context
    let user_id = crate::app::utils::token_utils::TokenUtils::extract_user_id_from_headers(&headers);
    let user_id_str = user_id.as_ref().map(|id| id.to_string());
    let user_id_ref = user_id_str.as_deref();

    match OrganizationPositionLevelService::find_by_id(&pool, &id, user_id_ref).await {
        Ok(Some(_organization_position_level)) => {
            let payload = UpdateOrganizationPositionLevel {
                organization_id: None,
                code: None,
                name: None,
                description: None,
                level: None,
                is_active: Some(true),
            };

            match OrganizationPositionLevelService::update(&pool, id, payload, user_id_ref).await {
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
pub async fn deactivate(State(pool): State<DbPool>, Path(id): Path<String>, headers: HeaderMap) -> impl IntoResponse {
    // Get current organization position level and update its active status
    // Extract user ID from authentication context
    let user_id = crate::app::utils::token_utils::TokenUtils::extract_user_id_from_headers(&headers);
    let user_id_str = user_id.as_ref().map(|id| id.to_string());
    let user_id_ref = user_id_str.as_deref();

    match OrganizationPositionLevelService::find_by_id(&pool, &id, user_id_ref).await {
        Ok(Some(_organization_position_level)) => {
            let payload = UpdateOrganizationPositionLevel {
                organization_id: None,
                code: None,
                name: None,
                description: None,
                level: None,
                is_active: Some(false),
            };

            match OrganizationPositionLevelService::update(&pool, id, payload, user_id_ref).await {
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

