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
    summary = "List all districts",
    description = "Get all districts with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Sort field and direction. Available fields: id, city_id, name, code, created_at, updated_at (prefix with '-' for descending)"),
        ("include" = Option<String>, Query, description = "Comma-separated list of relationships to include. Available: city, villages"),
        ("filter" = Option<serde_json::Value>, Query, description = "Filter parameters. Available filters: city_id, name, code (e.g., filter[name]=Downtown, filter[city_id]=01ARZ3...)"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to select. Available: id, city_id, name, code, created_at, updated_at"),
        ("cursor" = Option<String>, Query, description = "Cursor for cursor-based pagination"),
        ("pagination_type" = Option<String>, Query, description = "Pagination type: 'offset' or 'cursor' (default: cursor)"),
    ),
    responses(
        (status = 200, description = "List of districts", body = Vec<crate::app::models::district::DistrictResponse>),
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

    match DistrictService::create(&pool, payload, None).await {
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