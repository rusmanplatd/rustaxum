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
#[utoipa::path(
    get,
    path = "/api/organization-domains",
    responses(
        (status = 200, description = "List of organization domains"),
        (status = 500, description = "Internal server error")
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
#[utoipa::path(
    post,
    path = "/api/organization-domains",
    request_body = CreateOrganizationDomain,
    responses(
        (status = 201, description = "Organization domain created successfully"),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
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

    let user_ulid = match DieselUlid::from_string(&auth_user.user_id) {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Invalid user ID: {}", e)}))
            ).into_response();
        }
    };

    match OrganizationDomainService::create(&pool, data, user_ulid) {
        Ok(domain) => (StatusCode::CREATED, Json(domain.to_response())).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        ).into_response(),
    }
}

/// Get a single organization domain by ID
#[utoipa::path(
    get,
    path = "/api/organization-domains/{id}",
    params(
        ("id" = String, Path, description = "Organization domain ID (ULID)")
    ),
    responses(
        (status = 200, description = "Organization domain found"),
        (status = 404, description = "Organization domain not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Organization Domains"
)]
pub async fn show(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match OrganizationDomainService::find_by_id(&pool, &id) {
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

/// Update an organization domain
#[utoipa::path(
    put,
    path = "/api/organization-domains/{id}",
    params(
        ("id" = String, Path, description = "Organization domain ID (ULID)")
    ),
    request_body = UpdateOrganizationDomain,
    responses(
        (status = 200, description = "Organization domain updated successfully"),
        (status = 400, description = "Invalid request data"),
        (status = 404, description = "Organization domain not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Organization Domains"
)]
pub async fn update(
    State(pool): State<DbPool>,
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

    match OrganizationDomainService::update(&pool, &id, data, None) {
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
#[utoipa::path(
    delete,
    path = "/api/organization-domains/{id}",
    params(
        ("id" = String, Path, description = "Organization domain ID (ULID)")
    ),
    responses(
        (status = 204, description = "Organization domain deleted successfully"),
        (status = 404, description = "Organization domain not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Organization Domains"
)]
pub async fn destroy(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match OrganizationDomainService::delete(&pool, &id, None) {
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
