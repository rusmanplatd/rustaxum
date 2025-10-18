use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
    Extension,
};
use serde_json::json;

use crate::app::models::organization_type::{CreateOrganizationType, UpdateOrganizationType};
use crate::app::services::organization_type_service::OrganizationTypeService;
use crate::app::query_builder::{QueryBuilderService, QueryParams};
use crate::app::models::organization_type::OrganizationType;
use crate::database::DbPool;

/// List all organization types with query builder support
///
/// Supports advanced querying with filters, sorting, pagination, and field selection.
/// Only returns non-deleted records (soft delete support).
///
/// # Query Parameters
/// - `filter[field]=value` - Filter by field
/// - `sort=field` or `sort=-field` - Sort ascending/descending
/// - `page=1&per_page=15` - Pagination
/// - `fields=id,name,code` - Field selection
/// - `include=domain,organizations` - Eager load relationships
///
/// # Implementation
/// - Uses QueryBuilderService trait for consistent API
/// - Filters out soft-deleted records automatically
/// - Returns paginated results with metadata
#[utoipa::path(
    get,
    path = "/api/organization-types",
    summary = "List all organization types",
    description = "Get all organization types with advanced filtering, sorting, pagination, and field selection",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Multi-column sorting. Available fields: id, domain_id, code, name, level, created_at, updated_at. Syntax: 'field1,-field2,field3:desc'. Example: 'level,name,-created_at'"),
        ("include" = Option<String>, Query, description = "Eager load relationships. Available: domain, organizations, createdBy, updatedBy, deletedBy. Supports nested relationships. Example: 'domain,organizations,createdBy'"),
        ("filter" = Option<serde_json::Value>, Query, description = "Advanced filtering with comprehensive operators. Available filters: id, domain_id, code, name, description, level, created_at, updated_at, deleted_at, created_by_id, updated_by_id, deleted_by_id. Operators: eq, ne, gt, gte, lt, lte, like, ilike, contains, starts_with, ends_with, in, not_in, is_null, is_not_null, between. Examples: filter[name][contains]=dept, filter[level][gte]=2, filter[domain_id][eq]=01ARZ3"),
        ("fields" = Option<String>, Query, description = "Field selection for optimized responses. Available: id, domain_id, code, name, description, level, created_at, updated_at, deleted_at, created_by_id, updated_by_id, deleted_by_id. Example: fields[organization_types]=id,code,name,level"),
        ("cursor" = Option<String>, Query, description = "Cursor for high-performance pagination. Base64-encoded JSON cursor from previous response"),
        ("pagination_type" = Option<String>, Query, description = "Pagination strategy: 'offset' (traditional) or 'cursor' (high-performance, default)"),
    ),
    responses(
        (status = 200, description = "List of organization types with pagination", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value)
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
    Extension(auth_user): Extension<crate::app::http::middleware::auth_guard::AuthUser>,
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

    match OrganizationTypeService::create(&pool, data, &auth_user.user_id).await {
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
    match OrganizationTypeService::find_by_id(&pool, id) {
        Ok(Some(org_type)) => (StatusCode::OK, Json(org_type.to_response())).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Organization type not found"}))
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        ).into_response(),
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
    Extension(auth_user): Extension<crate::app::http::middleware::auth_guard::AuthUser>,
    Path(id): Path<String>,
    Json(data): Json<UpdateOrganizationType>,
) -> impl IntoResponse {
    // Get the existing type to check domain
    let existing_type = match OrganizationTypeService::find_by_id(&pool, id.clone()) {
        Ok(Some(t)) => t,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Organization type not found"}))
            ).into_response();
        },
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()}))
            ).into_response();
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

    match OrganizationTypeService::update(&pool, id, data, &auth_user.user_id).await {
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
    Extension(auth_user): Extension<crate::app::http::middleware::auth_guard::AuthUser>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match OrganizationTypeService::delete(&pool, id, &auth_user.user_id).await {
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
