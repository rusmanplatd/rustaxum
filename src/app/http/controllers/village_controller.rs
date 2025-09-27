use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::Serialize;
use crate::database::DbPool;

use crate::app::models::village::{CreateVillage, UpdateVillage, Village};
use crate::app::services::village_service::VillageService;
use crate::app::http::requests::{CreateVillageRequest, UpdateVillageRequest};
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
    path = "/api/villages",
    tag = "Villages",
    summary = "List all villages with enhanced filtering and geospatial support",
    description = "Retrieve villages with Laravel-style filtering, multi-column sorting, nested relationship loading, and high-performance pagination. Supports geospatial queries for location-based searches and rural community management.",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Items per page (default: 15, max: 100). Use smaller values for mobile/bandwidth-constrained environments"),
        ("sort" = Option<String>, Query, description = "Multi-column sorting with geospatial optimization. Available fields: id, district_id, name, code, latitude, longitude, created_at, updated_at. Syntax: 'field1,-field2,field3:desc'. Examples: 'name,-created_at', 'latitude:desc,longitude', 'distance,population:desc'"),
        ("include" = Option<String>, Query, description = "Eager load relationships with optimized JOIN queries. Available: district, district.city, district.city.province, district.city.province.country, createdBy, updatedBy, deletedBy, createdBy.organizations.position.level, updatedBy.organizations.position.level, deletedBy.organizations.position.level. Supports dot notation for nested relationships. Examples: 'district', 'district.city.province.country,createdBy.organizations.position.level'"),
        ("filter" = Option<serde_json::Value>, Query, description = "Advanced filtering with 15+ operators and geospatial support. Available filters: id, district_id, name, code, latitude, longitude, created_at, updated_at. Operators: eq, ne, gt, gte, lt, lte, like, ilike, contains, starts_with, ends_with, in, not_in, is_null, is_not_null, between. Examples: filter[name][contains]=green, filter[latitude][between]=-90,90, filter[district_id][in]=id1,id2, filter[name][starts_with]=Mountain"),
        ("fields" = Option<String>, Query, description = "Field selection for optimized responses and bandwidth reduction. Available: id, district_id, name, code, latitude, longitude, created_at, updated_at. Supports relationship field selection. Examples: fields[villages]=id,name,latitude,longitude, fields[districts]=id,name"),
        ("cursor" = Option<String>, Query, description = "Cursor for high-performance pagination with geospatial indexing support. Recommended for datasets > 1,000 records"),
        ("pagination_type" = Option<String>, Query, description = "Pagination strategy: 'offset' (traditional, good for small datasets) or 'cursor' (high-performance with spatial indexing, recommended default)"),
    ),
    responses(
        (status = 200, description = "List of villages with metadata", body = Vec<crate::app::models::village::VillageResponse>),
        (status = 400, description = "Invalid query parameters", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    match <Village as QueryBuilderService<Village>>::index(Query(params), &pool) {
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
    path = "/api/villages/{id}",
    tag = "Villages",
    summary = "Get village by ID",
    description = "Retrieve a specific village by its unique identifier",
    params(
        ("id" = String, Path, description = "Village unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Village details", body = crate::app::models::village::VillageResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Village not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn show(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    match VillageService::find_by_id(&pool, id) {
        Ok(Some(village)) => (StatusCode::OK, ResponseJson(village.to_response())).into_response(),
        Ok(None) => {
            let error = ErrorResponse {
                error: "Village not found".to_string(),
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

/// Create a new village
///
/// Create a new village with the provided information. All required fields must be provided
/// and will be validated according to the business rules.
///
/// # Request Body
/// The request must contain a valid CreateVillageRequest JSON payload with:
/// - `district_id`: ID of the district this village belongs to
/// - `name`: Village name (2-100 characters)
/// - `code`: Optional village code (2-10 characters)
/// - `latitude`: Optional latitude coordinate
/// - `longitude`: Optional longitude coordinate
///
/// # Example
/// ```json
/// {
///   "district_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
///   "name": "Green Valley",
///   "code": "GV",
///   "latitude": 40.7128,
///   "longitude": -74.0060
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/villages",
    tag = "Villages",
    summary = "Create new village",
    description = "Create a new village with the provided information",
    request_body = crate::app::http::requests::CreateVillageRequest,
    responses(
        (status = 201, description = "Village created successfully", body = crate::app::models::village::VillageResponse),
        (status = 400, description = "Validation error or bad request", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn store(State(pool): State<DbPool>, request: CreateVillageRequest) -> impl IntoResponse {
    let payload = CreateVillage {
        district_id: request.district_id,
        name: request.name,
        code: request.code,
        latitude: request.latitude,
        longitude: request.longitude,
    };

    match VillageService::create(&pool, payload, None).await {
        Ok(village) => (StatusCode::CREATED, ResponseJson(village.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Update an existing village
///
/// Update an existing village with the provided information. All fields are optional
/// for partial updates. Only provided fields will be updated.
///
/// # Path Parameters
/// - `id`: The unique identifier of the village to update (ULID format)
///
/// # Request Body
/// The request should contain an UpdateVillageRequest JSON payload with optional fields:
/// - `district_id`: Updated district ID
/// - `name`: Updated village name (2-100 characters)
/// - `code`: Updated village code (2-10 characters)
/// - `latitude`: Updated latitude coordinate
/// - `longitude`: Updated longitude coordinate
#[utoipa::path(
    put,
    path = "/api/villages/{id}",
    tag = "Villages",
    summary = "Update village",
    description = "Update an existing village with the provided information",
    params(
        ("id" = String, Path, description = "Village unique identifier (ULID format)")
    ),
    request_body = crate::app::http::requests::UpdateVillageRequest,
    responses(
        (status = 200, description = "Village updated successfully", body = crate::app::models::village::VillageResponse),
        (status = 400, description = "Invalid ID format or validation error", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Village not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    request: UpdateVillageRequest,
) -> impl IntoResponse {
    let payload = UpdateVillage {
        district_id: request.district_id,
        name: request.name,
        code: request.code,
        latitude: request.latitude,
        longitude: request.longitude,
    };

    match VillageService::update(&pool, id, payload) {
        Ok(village) => (StatusCode::OK, ResponseJson(village.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Delete a village
///
/// Permanently delete a village from the system. This action cannot be undone.
///
/// # Path Parameters
/// - `id`: The unique identifier of the village to delete (ULID format)
#[utoipa::path(
    delete,
    path = "/api/villages/{id}",
    tag = "Villages",
    summary = "Delete village",
    description = "Permanently delete a village from the system",
    params(
        ("id" = String, Path, description = "Village unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Village deleted successfully", body = crate::app::docs::MessageResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Village not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn destroy(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    match VillageService::delete(&pool, id) {
        Ok(_) => {
            let message = MessageResponse {
                message: "Village deleted successfully".to_string(),
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