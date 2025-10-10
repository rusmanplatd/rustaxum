use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::Serialize;
use crate::database::DbPool;

use crate::app::models::district::{CreateDistrict, UpdateDistrict, District};
use crate::app::services::district_service::DistrictService;
use crate::app::http::requests::{CreateDistrictRequest, UpdateDistrictRequest};
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
    path = "/api/districts",
    tag = "Districts",
    summary = "List all districts with comprehensive filtering and hierarchical relationships",
    description = "Retrieve districts with Laravel-style advanced querying, multi-column sorting, nested city/province relationships, and optimized pagination. Perfect for urban planning and administrative boundary management.",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Items per page (default: 15, max: 100). Recommended: 25-50 for administrative dashboards"),
        ("sort" = Option<String>, Query, description = "Multi-column sorting with administrative hierarchy support. Available fields: id, city_id, name, code, created_at, updated_at. Syntax: 'field1,-field2,field3:desc'. Examples: 'name', 'city_id,name:asc', '-created_at,name'"),
        ("include" = Option<String>, Query, description = "Eager load relationships with JOIN optimization. Available: city, city.province, city.province.country, villages, createdBy, updatedBy, deletedBy, createdBy.organizations.position.level, updatedBy.organizations.position.level, deletedBy.organizations.position.level. Supports deep nesting for administrative hierarchies. Examples: 'city.province', 'villages', 'city.province.country,createdBy.organizations.position.level'"),
        ("filter" = Option<serde_json::Value>, Query, description = "Advanced filtering with 15+ operators for administrative data. Available filters: id, city_id, name, code, created_at, updated_at. Operators: eq, ne, gt, gte, lt, lte, like, ilike, contains, starts_with, ends_with, in, not_in, is_null, is_not_null, between. Examples: filter[name][contains]=downtown, filter[city_id][in]=id1,id2, filter[code][starts_with]=DT"),
        ("fields" = Option<String>, Query, description = "Field selection for optimized administrative queries. Available: id, city_id, name, code, created_at, updated_at. Supports relationship field selection. Examples: fields[districts]=id,name,code, fields[cities]=id,name"),
        ("cursor" = Option<String>, Query, description = "Cursor for high-performance pagination with administrative data indexing"),
        ("pagination_type" = Option<String>, Query, description = "Pagination strategy: 'offset' (traditional) or 'cursor' (high-performance for large administrative datasets, recommended default)"),
    ),
    responses(
        (status = 200, description = "List of districts with administrative metadata", body = Vec<crate::app::models::district::DistrictResponse>),
        (status = 400, description = "Invalid query parameters", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    match <District as QueryBuilderService<District>>::index(Query(params), &pool) {
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
    path = "/api/districts/{id}",
    tag = "Districts",
    summary = "Get district by ID",
    description = "Retrieve a specific district by its unique identifier",
    params(
        ("id" = String, Path, description = "District unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "District details", body = crate::app::models::district::DistrictResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "District not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn show(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    match DistrictService::find_by_id(&pool, id) {
        Ok(Some(district)) => (StatusCode::OK, ResponseJson(district.to_response())).into_response(),
        Ok(None) => {
            let error = ErrorResponse {
                error: "District not found".to_string(),
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

/// Create a new district
///
/// Create a new district with the provided information. All required fields must be provided
/// and will be validated according to the business rules.
///
/// # Request Body
/// The request must contain a valid CreateDistrictRequest JSON payload with:
/// - `city_id`: ID of the city this district belongs to
/// - `name`: District name (2-100 characters)
/// - `code`: Optional district code (2-10 characters)
///
/// # Example
/// ```json
/// {
///   "city_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
///   "name": "Downtown",
///   "code": "DT"
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/districts",
    tag = "Districts",
    summary = "Create new district",
    description = "Create a new district with the provided information",
    request_body = crate::app::http::requests::CreateDistrictRequest,
    responses(
        (status = 201, description = "District created successfully", body = crate::app::models::district::DistrictResponse),
        (status = 400, description = "Validation error or bad request", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn store(State(pool): State<DbPool>, request: CreateDistrictRequest) -> impl IntoResponse {
    let payload = CreateDistrict {
        city_id: request.city_id,
        name: request.name,
        code: request.code,
    };

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Database connection error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    use diesel::prelude::*;
    use crate::schema::sys_users;

    let system_user_id: String = match sys_users::table
        .filter(sys_users::email.eq("system@seeder.internal"))
        .select(sys_users::id)
        .first(&mut conn)
    {
        Ok(id) => id,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to get system user: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    match DistrictService::create(&pool, payload, &system_user_id).await {
        Ok(district) => (StatusCode::CREATED, ResponseJson(district.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Update an existing district
///
/// Update an existing district with the provided information. All fields are optional
/// for partial updates. Only provided fields will be updated.
///
/// # Path Parameters
/// - `id`: The unique identifier of the district to update (ULID format)
///
/// # Request Body
/// The request should contain an UpdateDistrictRequest JSON payload with optional fields:
/// - `city_id`: Updated city ID
/// - `name`: Updated district name (2-100 characters)
/// - `code`: Updated district code (2-10 characters)
#[utoipa::path(
    put,
    path = "/api/districts/{id}",
    tag = "Districts",
    summary = "Update district",
    description = "Update an existing district with the provided information",
    params(
        ("id" = String, Path, description = "District unique identifier (ULID format)")
    ),
    request_body = crate::app::http::requests::UpdateDistrictRequest,
    responses(
        (status = 200, description = "District updated successfully", body = crate::app::models::district::DistrictResponse),
        (status = 400, description = "Invalid ID format or validation error", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "District not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    request: UpdateDistrictRequest,
) -> impl IntoResponse {
    let payload = UpdateDistrict {
        city_id: request.city_id,
        name: request.name,
        code: request.code,
    };

    match DistrictService::update(&pool, id, payload) {
        Ok(district) => (StatusCode::OK, ResponseJson(district.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Delete a district
///
/// Permanently delete a district from the system. This action cannot be undone.
///
/// # Path Parameters
/// - `id`: The unique identifier of the district to delete (ULID format)
#[utoipa::path(
    delete,
    path = "/api/districts/{id}",
    tag = "Districts",
    summary = "Delete district",
    description = "Permanently delete a district from the system",
    params(
        ("id" = String, Path, description = "District unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "District deleted successfully", body = crate::app::docs::MessageResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "District not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn destroy(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    match DistrictService::delete(&pool, id) {
        Ok(_) => {
            let message = MessageResponse {
                message: "District deleted successfully".to_string(),
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