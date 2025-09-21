use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::Serialize;
use ulid::Ulid;
use crate::database::DbPool;
use std::collections::HashMap;

use crate::app::models::organization::{CreateOrganization, UpdateOrganization};
use crate::app::services::organization_service::OrganizationService;
use crate::app::http::requests::{CreateOrganizationRequest, UpdateOrganizationRequest};

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

/// Get all organizations with optional filtering and pagination
///
/// Retrieve a list of all organizations with support for filtering and pagination.
/// You can filter by any field and sort by any column.
///
/// # Query Parameters
/// - `page`: Page number for pagination (default: 1)
/// - `limit`: Number of items per page (default: 10, max: 100)
/// - `sort`: Sort field (default: name)
/// - `direction`: Sort direction - asc or desc (default: asc)
/// - `filter[field]`: Filter by field value
///
/// # Example
/// ```
/// GET /api/organizations?page=1&limit=10&sort=name&direction=asc&filter[organization_type]=department
/// ```
#[utoipa::path(
    get,
    path = "/api/organizations",
    tag = "Organizations",
    summary = "List all organizations",
    description = "Get all organizations with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination"),
        ("limit" = Option<u32>, Query, description = "Number of items per page (max 100)"),
        ("sort" = Option<String>, Query, description = "Sort field"),
        ("direction" = Option<String>, Query, description = "Sort direction (asc/desc)"),
    ),
    responses(
        (status = 200, description = "List of organizations", body = Vec<crate::app::models::organization::OrganizationResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    match OrganizationService::list(&pool, params).await {
        Ok(organizations) => {
            let responses: Vec<_> = organizations.into_iter().map(|o| o.to_response()).collect();
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

/// Get a specific organization by ID
///
/// Retrieve detailed information about a specific organization using its unique identifier.
/// The ID should be a valid ULID format.
///
/// # Path Parameters
/// - `id`: The unique identifier of the organization (ULID format)
///
/// # Example
/// ```
/// GET /api/organizations/01ARZ3NDEKTSV4RRFFQ69G5FAV
/// ```
#[utoipa::path(
    get,
    path = "/api/organizations/{id}",
    tag = "Organizations",
    summary = "Get organization by ID",
    description = "Retrieve a specific organization by its unique identifier",
    params(
        ("id" = String, Path, description = "Organization unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Organization details", body = crate::app::models::organization::OrganizationResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Organization not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn show(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    let organization_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match OrganizationService::find_by_id(&pool, organization_id).await {
        Ok(Some(organization)) => (StatusCode::OK, ResponseJson(organization.to_response())).into_response(),
        Ok(None) => {
            let error = ErrorResponse {
                error: "Organization not found".to_string(),
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

/// Create a new organization
///
/// Create a new organization with the provided information. All required fields must be provided
/// and will be validated according to the business rules.
///
/// # Request Body
/// The request must contain a valid CreateOrganizationRequest JSON payload with:
/// - `name`: Organization name (2-100 characters)
/// - `organization_type`: Type of organization (2-50 characters)
/// - `parent_id`: Optional parent organization ID (ULID format)
/// - `code`: Optional organization code (2-20 characters, uppercase)
/// - `description`: Optional description (max 500 characters)
///
/// # Example
/// ```json
/// {
///   "name": "Engineering Department",
///   "organization_type": "department",
///   "code": "ENG-001",
///   "description": "Software engineering and development department"
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/organizations",
    tag = "Organizations",
    summary = "Create new organization",
    description = "Create a new organization with the provided information",
    request_body = crate::app::http::requests::CreateOrganizationRequest,
    responses(
        (status = 201, description = "Organization created successfully", body = crate::app::models::organization::OrganizationResponse),
        (status = 400, description = "Validation error or bad request", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn store(State(pool): State<DbPool>, request: CreateOrganizationRequest) -> impl IntoResponse {
    let payload = CreateOrganization {
        name: request.name,
        organization_type: request.organization_type,
        parent_id: request.parent_id,
        code: request.code,
        description: request.description,
    };

    match OrganizationService::create(&pool, payload).await {
        Ok(organization) => (StatusCode::CREATED, ResponseJson(organization.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Update an existing organization
///
/// Update an existing organization with the provided information. All fields are optional
/// for partial updates. Only provided fields will be updated.
///
/// # Path Parameters
/// - `id`: The unique identifier of the organization to update (ULID format)
///
/// # Request Body
/// The request should contain an UpdateOrganizationRequest JSON payload with optional fields:
/// - `name`: Updated organization name (2-100 characters)
/// - `organization_type`: Updated organization type (2-50 characters)
/// - `parent_id`: Updated parent organization ID (ULID format)
/// - `code`: Updated organization code (2-20 characters, uppercase)
/// - `description`: Updated description (max 500 characters)
/// - `is_active`: Updated active status
#[utoipa::path(
    put,
    path = "/api/organizations/{id}",
    tag = "Organizations",
    summary = "Update organization",
    description = "Update an existing organization with the provided information",
    params(
        ("id" = String, Path, description = "Organization unique identifier (ULID format)")
    ),
    request_body = crate::app::http::requests::UpdateOrganizationRequest,
    responses(
        (status = 200, description = "Organization updated successfully", body = crate::app::models::organization::OrganizationResponse),
        (status = 400, description = "Invalid ID format or validation error", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Organization not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    request: UpdateOrganizationRequest,
) -> impl IntoResponse {
    let organization_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    let payload = UpdateOrganization {
        name: request.name,
        organization_type: request.organization_type,
        parent_id: request.parent_id,
        code: request.code,
        description: request.description,
        is_active: request.is_active,
    };

    match OrganizationService::update(&pool, organization_id, payload).await {
        Ok(organization) => (StatusCode::OK, ResponseJson(organization.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Delete an organization
///
/// Permanently delete an organization from the system. This action cannot be undone.
///
/// # Path Parameters
/// - `id`: The unique identifier of the organization to delete (ULID format)
#[utoipa::path(
    delete,
    path = "/api/organizations/{id}",
    tag = "Organizations",
    summary = "Delete organization",
    description = "Permanently delete an organization from the system",
    params(
        ("id" = String, Path, description = "Organization unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Organization deleted successfully", body = crate::app::docs::MessageResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Organization not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn destroy(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    let organization_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match OrganizationService::delete(&pool, organization_id).await {
        Ok(_) => {
            let message = MessageResponse {
                message: "Organization deleted successfully".to_string(),
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

/// Get children organizations
///
/// Retrieve all child organizations of a specific parent organization.
///
/// # Path Parameters
/// - `id`: The unique identifier of the parent organization (ULID format)
#[utoipa::path(
    get,
    path = "/api/organizations/{id}/children",
    tag = "Organizations",
    summary = "Get organization children",
    description = "Retrieve all child organizations of a specific parent organization",
    params(
        ("id" = String, Path, description = "Parent organization unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "List of child organizations", body = Vec<crate::app::models::organization::OrganizationResponse>),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn children(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    let parent_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match OrganizationService::find_children(&pool, parent_id).await {
        Ok(organizations) => {
            let responses: Vec<_> = organizations.into_iter().map(|o| o.to_response()).collect();
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

/// Get root organizations
///
/// Retrieve all root-level organizations (organizations without a parent).
#[utoipa::path(
    get,
    path = "/api/organizations/roots",
    tag = "Organizations",
    summary = "Get root organizations",
    description = "Retrieve all root-level organizations (organizations without a parent)",
    responses(
        (status = 200, description = "List of root organizations", body = Vec<crate::app::models::organization::OrganizationResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn roots(State(pool): State<DbPool>) -> impl IntoResponse {
    match OrganizationService::find_root_organizations(&pool).await {
        Ok(organizations) => {
            let responses: Vec<_> = organizations.into_iter().map(|o| o.to_response()).collect();
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