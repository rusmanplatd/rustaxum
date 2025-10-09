use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::app::models::organization_type::{CreateOrganizationType, UpdateOrganizationType};
use crate::app::services::organization_type_service::OrganizationTypeService;
use crate::app::query_builder::{QueryBuilderService, QueryParams};
use crate::app::models::organization_type::OrganizationType;
use crate::database::DbPool;

/// List all organization types with query builder support
#[utoipa::path(
    get,
    path = "/api/organization-types",
    responses(
        (status = 200, description = "List of organization types"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Organization Types"
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    match <OrganizationType as QueryBuilderService<OrganizationType>>::index(Query(params), &pool) {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        ).into_response(),
    }
}

/// Create a new organization type
#[utoipa::path(
    post,
    path = "/api/organization-types",
    request_body = CreateOrganizationType,
    responses(
        (status = 201, description = "Organization type created successfully"),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Organization Types"
)]
pub async fn store(
    State(pool): State<DbPool>,
    Json(data): Json<CreateOrganizationType>,
) -> impl IntoResponse {
    // Check if code is unique within domain
    if let Some(ref code) = data.code {
        match OrganizationTypeService::is_code_unique(&pool, &data.domain_id.to_string(), code, None) {
            Ok(false) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Type code already exists in this domain"}))
                ).into_response();
            },
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()}))
                ).into_response();
            },
            _ => {}
        }
    }

    match OrganizationTypeService::create(&pool, data, None) {
        Ok(org_type) => (StatusCode::CREATED, Json(org_type.to_response())).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        ).into_response(),
    }
}

/// Get a single organization type by ID
#[utoipa::path(
    get,
    path = "/api/organization-types/{id}",
    params(
        ("id" = String, Path, description = "Organization type ID (ULID)")
    ),
    responses(
        (status = 200, description = "Organization type found"),
        (status = 404, description = "Organization type not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Organization Types"
)]
pub async fn show(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match OrganizationTypeService::find_by_id(&pool, &id) {
        Ok(org_type) => (StatusCode::OK, Json(org_type.to_response())).into_response(),
        Err(e) => {
            if e.to_string().contains("NotFound") || e.to_string().contains("not found") {
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Organization type not found"}))
                ).into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()}))
                ).into_response()
            }
        }
    }
}

/// Update an organization type
#[utoipa::path(
    put,
    path = "/api/organization-types/{id}",
    params(
        ("id" = String, Path, description = "Organization type ID (ULID)")
    ),
    request_body = UpdateOrganizationType,
    responses(
        (status = 200, description = "Organization type updated successfully"),
        (status = 400, description = "Invalid request data"),
        (status = 404, description = "Organization type not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Organization Types"
)]
pub async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    Json(data): Json<UpdateOrganizationType>,
) -> impl IntoResponse {
    // Get the existing type to check domain
    let existing_type = match OrganizationTypeService::find_by_id(&pool, &id) {
        Ok(t) => t,
        Err(e) => {
            if e.to_string().contains("NotFound") || e.to_string().contains("not found") {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Organization type not found"}))
                ).into_response();
            } else {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()}))
                ).into_response();
            }
        }
    };

    // Check if code is unique within domain (if provided)
    if let Some(Some(ref code)) = data.code {
        let domain_id = data.domain_id.as_ref().unwrap_or(&existing_type.domain_id);
        match OrganizationTypeService::is_code_unique(&pool, &domain_id.to_string(), code, Some(&id)) {
            Ok(false) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Type code already exists in this domain"}))
                ).into_response();
            },
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()}))
                ).into_response();
            },
            _ => {}
        }
    }

    match OrganizationTypeService::update(&pool, &id, data, None) {
        Ok(org_type) => (StatusCode::OK, Json(org_type.to_response())).into_response(),
        Err(e) => {
            if e.to_string().contains("NotFound") || e.to_string().contains("not found") {
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Organization type not found"}))
                ).into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()}))
                ).into_response()
            }
        }
    }
}

/// Delete an organization type (soft delete)
#[utoipa::path(
    delete,
    path = "/api/organization-types/{id}",
    params(
        ("id" = String, Path, description = "Organization type ID (ULID)")
    ),
    responses(
        (status = 204, description = "Organization type deleted successfully"),
        (status = 404, description = "Organization type not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Organization Types"
)]
pub async fn destroy(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match OrganizationTypeService::delete(&pool, &id, None) {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            if e.to_string().contains("NotFound") || e.to_string().contains("not found") {
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Organization type not found"}))
                ).into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()}))
                ).into_response()
            }
        }
    }
}
