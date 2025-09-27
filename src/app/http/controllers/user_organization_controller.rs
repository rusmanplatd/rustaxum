use axum::{
    extract::{Path, State, Query},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json},
};
use crate::database::DbPool;
use ulid::Ulid;

use crate::app::models::user_organization::UserOrganization;
use crate::app::models::DieselUlid;
use crate::app::http::requests::user_organization_requests::{
    CreateUserOrganizationRequest,
    UpdateUserOrganizationRequest,
};
use crate::app::services::user_organization_service::UserOrganizationService;
use crate::app::query_builder::{QueryParams, QueryBuilderService};

/// Get all user organization relationships with filtering and pagination
#[utoipa::path(
    get,
    path = "/api/user-organizations",
    tag = "User Organizations",
    summary = "List all user organization relationships",
    description = "Get all user organization relationships with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Sort field and direction. Available fields: id, user_id, organization_id, position_id, is_active, started_at, ended_at, created_at, updated_at (prefix with '-' for descending)"),
        ("include" = Option<String>, Query, description = "Comma-separated list of relationships to include. Available: user, organization, position, createdBy, updatedBy, deletedBy, createdBy.organizations.position.level, updatedBy.organizations.position.level, deletedBy.organizations.position.level"),
        ("filter" = Option<serde_json::Value>, Query, description = "Filter parameters. Available filters: user_id, organization_id, position_id, is_active, started_at, ended_at (e.g., filter[is_active]=true, filter[user_id]=01ARZ3NDEK)"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to select. Available: id, user_id, organization_id, position_id, is_active, started_at, ended_at, created_at, updated_at"),
        ("cursor" = Option<String>, Query, description = "Cursor for cursor-based pagination"),
        ("pagination_type" = Option<String>, Query, description = "Pagination type: 'offset' or 'cursor' (default: cursor)"),
    ),
    responses(
        (status = 200, description = "List of user organization relationships", body = Vec<crate::app::models::user_organization::UserOrganizationResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    // Authentication is handled by middleware

    match <UserOrganization as QueryBuilderService<UserOrganization>>::index(Query(params), &pool) {
        Ok(result) => {
            (StatusCode::OK, Json(serde_json::json!(result))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch user organization relationships"
            }))).into_response()
        }
    }
}

/// Get a specific user organization relationship by ID
#[utoipa::path(
    get,
    path = "/api/user-organizations/{id}",
    tag = "User Organizations",
    summary = "Get user organization relationship by ID",
    description = "Retrieve a specific user organization relationship by its unique identifier",
    params(
        ("id" = String, Path, description = "User organization relationship unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "User organization relationship details", body = crate::app::models::user_organization::UserOrganizationResponse),
        (status = 404, description = "User organization relationship not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn show(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Authentication is handled by middleware
    let user_org_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid user organization ID format"
            }))).into_response();
        }
    };

    match UserOrganizationService::find_by_id(&pool, &user_org_id.to_string())
        {
        Ok(Some(user_org)) => {
            (StatusCode::OK, Json(serde_json::json!(user_org.to_response()))).into_response()
        },
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "User organization relationship not found"
            }))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch user organization relationship"
            }))).into_response()
        }
    }
}

/// Create a new user organization relationship
#[utoipa::path(
    post,
    path = "/api/user-organizations",
    tag = "User Organizations",
    summary = "Create user organization relationship",
    description = "Create a new user organization relationship with validation",
    request_body = CreateUserOrganizationRequest,
    responses(
        (status = 201, description = "User organization relationship created successfully", body = crate::app::models::user_organization::UserOrganizationResponse),
        (status = 422, description = "Validation error", body = crate::app::http::form_request::ValidationErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn store(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    request: CreateUserOrganizationRequest,
) -> impl IntoResponse {
    // Authentication is handled by middleware

    // Convert request to CreateUserOrganization
    let create_data = crate::app::models::user_organization::CreateUserOrganization {
        user_id: request.user_id,
        organization_id: request.organization_id,
        organization_position_id: request.organization_position_id,
        started_at: request.started_at,
    };

    // Extract user ID from authentication context
    let user_id = crate::app::utils::token_utils::TokenUtils::extract_user_id_from_headers(&headers);
    let user_id_str = user_id.as_ref().map(|id| id.to_string());
    let user_id_ref = user_id_str.as_deref();

    match UserOrganizationService::create(&pool, create_data, user_id_ref).await
        {
        Ok(user_org) => {
            (StatusCode::CREATED, Json(serde_json::json!(user_org.to_response()))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to create user organization relationship"
            }))).into_response()
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/user-organizations/{id}",
    tag = "User Organizations",
    summary = "Update user organization relationship",
    description = "Update an existing user organization relationship with validation",
    params(
        ("id" = String, Path, description = "User organization relationship unique identifier (ULID format)")
    ),
    request_body = UpdateUserOrganizationRequest,
    responses(
        (status = 200, description = "User organization relationship updated successfully", body = crate::app::models::user_organization::UserOrganizationResponse),
        (status = 404, description = "User organization relationship not found", body = crate::app::docs::ErrorResponse),
        (status = 422, description = "Validation error", body = crate::app::http::form_request::ValidationErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    request: UpdateUserOrganizationRequest,
) -> impl IntoResponse {
    // Authentication is handled by middleware

    let user_org_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid user organization ID format"
            }))).into_response();
        }
    };

    // Fetch the existing user organization relationship
    let _user_org = match UserOrganizationService::find_by_id(&pool, &user_org_id.to_string())
        {
        Ok(Some(user_org)) => user_org,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "User organization relationship not found"
            }))).into_response();
        },
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch user organization relationship"
            }))).into_response();
        }
    };

    // Convert string IDs to DieselUlid types
    let organization_id = match request.organization_id {
        Some(id_str) => {
            match Ulid::from_string(&id_str) {
                Ok(ulid) => Some(DieselUlid(ulid)),
                Err(_) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "error": "Invalid organization_id format"
                }))).into_response(),
            }
        }
        None => None,
    };

    let organization_position_id = match request.organization_position_id {
        Some(id_str) => {
            match Ulid::from_string(&id_str) {
                Ok(ulid) => Some(DieselUlid(ulid)),
                Err(_) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "error": "Invalid organization_position_id format"
                }))).into_response(),
            }
        }
        None => None,
    };

    // Create update data structure and validate
    let update_data = crate::app::models::user_organization::UpdateUserOrganization {
        organization_id,
        organization_position_id,
        is_active: request.is_active,
        started_at: request.started_at,
        ended_at: Some(request.ended_at),
    };

    // Update in database using service
    match UserOrganizationService::update(&pool, user_org_id.to_string(), update_data)
        {
        Ok(updated_user_org) => {
            (StatusCode::OK, Json(serde_json::json!(updated_user_org.to_response()))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to update user organization relationship"
            }))).into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/user-organizations/{id}",
    tag = "User Organizations",
    summary = "Delete user organization relationship",
    description = "Delete an existing user organization relationship",
    params(
        ("id" = String, Path, description = "User organization relationship unique identifier (ULID format)")
    ),
    responses(
        (status = 204, description = "User organization relationship deleted successfully"),
        (status = 404, description = "User organization relationship not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn destroy(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Authentication is handled by middleware

    let user_org_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid user organization ID format"
            }))).into_response();
        }
    };

    // Check if exists first and delete if found
    let exists = match UserOrganizationService::find_by_id(&pool, &user_org_id.to_string()) {
        Ok(result) => result.is_some(),
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to check if user organization relationship exists"
            }))).into_response();
        }
    };

    if !exists {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "User organization relationship not found"
        }))).into_response();
    }

    // Delete from database using service
    match UserOrganizationService::delete(&pool, user_org_id.to_string())
            {
        Ok(_) => {
            (StatusCode::NO_CONTENT, Json(serde_json::json!({}))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to delete user organization relationship"
            }))).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/user-organizations/{id}/transfer",
    tag = "User Organizations",
    summary = "Transfer user to different organization",
    description = "Transfer a user to a different organization",
    params(
        ("id" = String, Path, description = "Current user organization relationship unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "User transferred successfully"),
        (status = 404, description = "User organization relationship not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn transfer(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Authentication is handled by middleware

    let user_org_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid user organization ID format"
            }))).into_response();
        }
    };

    // Extract new organization and organization position IDs from payload
    let new_organization_id = match payload.get("organization_id").and_then(|v| v.as_str()) {
        Some(id_str) => match Ulid::from_string(id_str) {
            Ok(id) => id,
            Err(_) => {
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "error": "Invalid new organization ID format"
                }))).into_response();
            }
        },
        None => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "New organization ID is required"
            }))).into_response();
        }
    };

    let new_organization_position_id = match payload.get("organization_position_id").and_then(|v| v.as_str()) {
        Some(id_str) => match Ulid::from_string(id_str) {
            Ok(id) => id,
            Err(_) => {
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "error": "Invalid new organization position ID format"
                }))).into_response();
            }
        },
        None => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "New organization position ID is required"
            }))).into_response();
        }
    };

    // Fetch the current user organization relationship
    let mut user_org = match UserOrganizationService::find_by_id(&pool, &user_org_id.to_string())
        {
        Ok(Some(user_org)) => user_org,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "User organization relationship not found"
            }))).into_response();
        },
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch user organization relationship"
            }))).into_response();
        }
    };

    // Perform the transfer
    match user_org.transfer_to_organization(&pool, new_organization_id.to_string(), new_organization_position_id.to_string()) {
        Ok(_) => {
            (StatusCode::OK, Json(serde_json::json!({
                "message": "User transferred successfully"
            }))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to transfer user"
            }))).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/user-organizations/{id}/activate",
    tag = "User Organizations",
    summary = "Activate user organization relationship",
    description = "Activate a deactivated user organization relationship",
    params(
        ("id" = String, Path, description = "User organization relationship unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "User organization relationship activated successfully", body = crate::app::models::user_organization::UserOrganizationResponse),
        (status = 404, description = "User organization relationship not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn activate(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Authentication is handled by middleware

    let user_org_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid user organization ID format"
            }))).into_response();
        }
    };

    // Fetch the user organization relationship
    let mut user_org = match UserOrganizationService::find_by_id(&pool, &user_org_id.to_string())
        {
        Ok(Some(user_org)) => user_org,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "User organization relationship not found"
            }))).into_response();
        },
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch user organization relationship"
            }))).into_response();
        }
    };

    // Activate the relationship
    match user_org.activate(&pool) {
        Ok(_) => {
            (StatusCode::OK, Json(serde_json::json!(user_org.to_response()))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to activate user organization relationship"
            }))).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/user-organizations/{id}/deactivate",
    tag = "User Organizations",
    summary = "Deactivate user organization relationship",
    description = "Deactivate an active user organization relationship",
    params(
        ("id" = String, Path, description = "User organization relationship unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "User organization relationship deactivated successfully", body = crate::app::models::user_organization::UserOrganizationResponse),
        (status = 404, description = "User organization relationship not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn deactivate(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Authentication is handled by middleware

    let user_org_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid user organization ID format"
            }))).into_response();
        }
    };

    // Fetch the user organization relationship
    let mut user_org = match UserOrganizationService::find_by_id(&pool, &user_org_id.to_string())
        {
        Ok(Some(user_org)) => user_org,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "User organization relationship not found"
            }))).into_response();
        },
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch user organization relationship"
            }))).into_response();
        }
    };

    // Deactivate the relationship
    match user_org.deactivate(&pool) {
        Ok(_) => {
            (StatusCode::OK, Json(serde_json::json!(user_org.to_response()))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to deactivate user organization relationship"
            }))).into_response()
        }
    }
}