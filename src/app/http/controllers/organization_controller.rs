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
use crate::app::models::DieselUlid;
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
    match OrganizationService::list(&pool, params) {
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

    match OrganizationService::find_by_id(&pool, organization_id.to_string()) {
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
    // Convert parent_id from String to DieselUlid if provided
    let parent_id = match request.parent_id {
        Some(id_str) => {
            match Ulid::from_string(&id_str) {
                Ok(ulid) => Some(DieselUlid(ulid)),
                Err(_) => return (StatusCode::BAD_REQUEST, ResponseJson(ErrorResponse {
                    error: "Invalid parent_id format".to_string(),
                })).into_response(),
            }
        }
        None => None,
    };

    let payload = CreateOrganization {
        name: request.name,
        organization_type: request.organization_type,
        parent_id,
        code: request.code,
        level: request.level,
        address: request.address,
        authorized_capital: request.authorized_capital,
        business_activities: request.business_activities,
        contact_persons: request.contact_persons,
        description: request.description,
        email: request.email,
        establishment_date: request.establishment_date,
        governance_structure: request.governance_structure,
        legal_status: request.legal_status,
        paid_capital: request.paid_capital,
        path: None, // path will be auto-generated based on hierarchy
        phone: request.phone,
        registration_number: request.registration_number,
        tax_number: request.tax_number,
        website: request.website,
    };

    match OrganizationService::create(&pool, payload) {
        Ok(organization) => (StatusCode::CREATED, ResponseJson(organization.to_response())).into_response(),
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

    // Convert parent_id from String to DieselUlid if provided
    let parent_id = match request.parent_id {
        Some(id_str) => {
            match Ulid::from_string(&id_str) {
                Ok(ulid) => Some(Some(DieselUlid(ulid))),
                Err(_) => return (StatusCode::BAD_REQUEST, ResponseJson(ErrorResponse {
                    error: "Invalid parent_id format".to_string(),
                })).into_response(),
            }
        }
        None => None,
    };

    let payload = UpdateOrganization {
        name: request.name,
        organization_type: request.organization_type,
        parent_id,
        code: Some(request.code),
        level: None,
        address: None,
        authorized_capital: None,
        business_activities: None,
        contact_persons: None,
        description: Some(request.description),
        email: None,
        establishment_date: None,
        governance_structure: None,
        legal_status: None,
        paid_capital: None,
        path: None, // path is typically auto-generated
        phone: None,
        registration_number: None,
        tax_number: None,
        website: None,
        is_active: request.is_active,
    };

    match OrganizationService::update(&pool, organization_id.to_string(), payload) {
        Ok(organization) => (StatusCode::OK, ResponseJson(organization.to_response())).into_response(),
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
    match OrganizationService::delete(&pool, id) {
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
    match OrganizationService::find_children(&pool, id) {
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
    match OrganizationService::find_root_organizations(&pool) {
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