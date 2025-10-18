use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
    Extension,
};
use serde_json::json;

use crate::app::models::organization_domain::{CreateOrganizationDomain, UpdateOrganizationDomain};
use crate::app::services::organization_domain_service::OrganizationDomainService;
use crate::app::query_builder::{QueryBuilderService, QueryParams};
use crate::app::models::organization_domain::OrganizationDomain;
use crate::database::DbPool;

/// List all organization domains with query builder support
///
/// Supports advanced querying with filters, sorting, pagination, and field selection.
/// Only returns non-deleted records (soft delete support).
///
/// # Query Parameters
/// - `filter[field]=value` - Filter by field
/// - `sort=field` or `sort=-field` - Sort ascending/descending
/// - `page=1&per_page=15` - Pagination
/// - `fields=id,name,code` - Field selection
///
/// # Implementation
/// - Uses QueryBuilderService trait for consistent API
/// - Filters out soft-deleted records automatically
/// - Returns paginated results with metadata
#[utoipa::path(
    get,
    path = "/api/organization-domains",
    summary = "List all organization domains",
    description = "Get all organization domains with advanced filtering, sorting, pagination, and field selection",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Multi-column sorting. Available fields: id, code, name, created_at, updated_at. Syntax: 'field1,-field2,field3:desc'. Example: 'name,-created_at'"),
        ("include" = Option<String>, Query, description = "Eager load relationships. Available: types, organizations, createdBy, updatedBy, deletedBy. Supports nested relationships. Example: 'types,organizations,createdBy'"),
        ("filter" = Option<serde_json::Value>, Query, description = "Advanced filtering with comprehensive operators. Available filters: id, code, name, description, created_at, updated_at, deleted_at, created_by_id, updated_by_id, deleted_by_id. Operators: eq, ne, gt, gte, lt, lte, like, ilike, contains, starts_with, ends_with, in, not_in, is_null, is_not_null, between. Examples: filter[name][contains]=gov, filter[code][in]=GOV,EDU"),
        ("fields" = Option<String>, Query, description = "Field selection for optimized responses. Available: id, code, name, description, created_at, updated_at, deleted_at, created_by_id, updated_by_id, deleted_by_id. Example: fields[organization_domains]=id,code,name"),
        ("cursor" = Option<String>, Query, description = "Cursor for high-performance pagination. Base64-encoded JSON cursor from previous response"),
        ("pagination_type" = Option<String>, Query, description = "Pagination strategy: 'offset' (traditional) or 'cursor' (high-performance, default)"),
    ),
    responses(
        (status = 200, description = "List of organization domains with pagination", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    tag = "Organization Domains"
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    match <OrganizationDomain as QueryBuilderService<OrganizationDomain>>::index(Query(params), &pool) {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        ).into_response(),
    }
}

/// Create a new organization domain
///
/// Creates a new organization domain with automatic activity logging.
/// The authenticated user is recorded as the creator for audit purposes.
///
/// # Implementation Details
/// - Uses ULID for primary keys (26-character, URL-safe, sortable)
/// - Validates code uniqueness before creation
/// - Logs creation activity to `activity_log` table
/// - Supports soft deletes via `deleted_at` timestamp
/// - Auto-populates `created_at`, `updated_at`, `created_by_id`, `updated_by_id`
///
/// # Security
/// Requires authentication. The user_id from JWT token is used for tracking.
#[utoipa::path(
    post,
    path = "/api/organization-domains",
    request_body = CreateOrganizationDomain,
    responses(
        (status = 201, description = "Organization domain created successfully", body = crate::app::models::organization_domain::OrganizationDomainResponse),
        (status = 400, description = "Validation error or duplicate code", body = serde_json::Value),
        (status = 401, description = "Unauthorized - authentication required"),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Organization Domains"
)]
pub async fn store(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<crate::app::http::middleware::auth_guard::AuthUser>,
    Json(data): Json<CreateOrganizationDomain>,
) -> impl IntoResponse {
    use crate::app::models::DieselUlid;

    // Check if code is unique
    if let Some(ref code) = data.code {
        match OrganizationDomainService::is_code_unique(&pool, code, None) {
            Ok(false) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Domain code already exists"}))
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

    match OrganizationDomainService::create(&pool, data, &auth_user.user_id).await {
        Ok(domain) => (StatusCode::CREATED, Json(domain.to_response())).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        ).into_response(),
    }
}

/// Get a single organization domain by ID
///
/// Retrieves a single organization domain by its ULID.
/// Returns 404 if not found or soft-deleted.
///
/// # Implementation
/// - Filters by `id` and `deleted_at IS NULL`
/// - Returns full domain details including metadata
/// - ULID format: 26 characters (e.g., "01ARZ3NDEKTSV4RRFFQ69G5FAV")
#[utoipa::path(
    get,
    path = "/api/organization-domains/{id}",
    params(
        ("id" = String, Path, description = "Organization domain ID in ULID format (26 characters)")
    ),
    responses(
        (status = 200, description = "Organization domain found", body = crate::app::models::organization_domain::OrganizationDomainResponse),
        (status = 404, description = "Organization domain not found or deleted", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    tag = "Organization Domains"
)]
pub async fn show(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match OrganizationDomainService::find_by_id(&pool, id) {
        Ok(Some(domain)) => (StatusCode::OK, Json(domain.to_response())).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Organization domain not found"}))
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        ).into_response(),
    }
}

/// Update an organization domain
///
/// Updates an existing organization domain with change tracking and activity logging.
/// Only non-null fields in the request are updated.
///
/// # Implementation Details
/// - Partial updates supported (only provided fields are changed)
/// - Validates code uniqueness (excluding current record)
/// - Logs changes with before/after values to `activity_log`
/// - Auto-updates `updated_at` and `updated_by_id` fields
/// - Returns 404 if domain doesn't exist or is soft-deleted
///
/// # Security
/// Requires authentication. User ID tracked for audit purposes.
#[utoipa::path(
    put,
    path = "/api/organization-domains/{id}",
    params(
        ("id" = String, Path, description = "Organization domain ID in ULID format")
    ),
    request_body = UpdateOrganizationDomain,
    responses(
        (status = 200, description = "Organization domain updated successfully", body = crate::app::models::organization_domain::OrganizationDomainResponse),
        (status = 400, description = "Validation error or duplicate code", body = serde_json::Value),
        (status = 401, description = "Unauthorized - authentication required"),
        (status = 404, description = "Organization domain not found or deleted", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Organization Domains"
)]
pub async fn update(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<crate::app::http::middleware::auth_guard::AuthUser>,
    Path(id): Path<String>,
    Json(data): Json<UpdateOrganizationDomain>,
) -> impl IntoResponse {
    // Check if code is unique (if provided)
    if let Some(Some(ref code)) = data.code {
        match OrganizationDomainService::is_code_unique(&pool, code, Some(&id)) {
            Ok(false) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Domain code already exists"}))
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

    match OrganizationDomainService::update(&pool, id, data, &auth_user.user_id).await {
        Ok(domain) => (StatusCode::OK, Json(domain.to_response())).into_response(),
        Err(e) => {
            if e.to_string().contains("NotFound") || e.to_string().contains("not found") {
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Organization domain not found"}))
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

/// Delete an organization domain (soft delete)
///
/// Performs a soft delete by setting the `deleted_at` timestamp.
/// The record remains in the database but is excluded from queries.
/// Deletion is logged for audit purposes.
///
/// # Implementation Details
/// - Soft delete: Sets `deleted_at` to current UTC timestamp
/// - Sets `deleted_by_id` to authenticated user's ID
/// - Logs deletion activity to `activity_log`
/// - Record still exists in database but filtered from queries
/// - Can be restored by clearing `deleted_at` (requires direct DB access)
///
/// # Security
/// Requires authentication. User ID recorded for accountability.
///
/// # Note
/// Hard deletes are not exposed via API. Use database migrations for cleanup.
#[utoipa::path(
    delete,
    path = "/api/organization-domains/{id}",
    params(
        ("id" = String, Path, description = "Organization domain ID in ULID format")
    ),
    responses(
        (status = 204, description = "Organization domain soft-deleted successfully"),
        (status = 401, description = "Unauthorized - authentication required"),
        (status = 404, description = "Organization domain not found or already deleted"),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Organization Domains"
)]
pub async fn destroy(
    State(pool): State<DbPool>,
    Extension(auth_user): Extension<crate::app::http::middleware::auth_guard::AuthUser>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match OrganizationDomainService::delete(&pool, id, &auth_user.user_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            if e.to_string().contains("NotFound") || e.to_string().contains("not found") {
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Organization domain not found"}))
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
