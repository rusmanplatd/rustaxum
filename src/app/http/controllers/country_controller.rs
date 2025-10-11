use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    Extension,
};
use serde::Serialize;
use crate::database::DbPool;

use crate::app::models::country::{CreateCountry, UpdateCountry, Country};
use crate::app::services::country_service::CountryService;
use crate::app::http::requests::{CreateCountryRequest, UpdateCountryRequest};
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
    path = "/api/countries",
    tag = "Countries",
    summary = "List all countries",
    description = "Get all countries with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Multi-column sorting. Available fields: id, name, iso_code, phone_code, created_at, updated_at. Syntax: 'field1,-field2,field3:desc' (- prefix or :desc for descending). Example: 'name,-created_at,iso_code:asc'"),
        ("include" = Option<String>, Query, description = "Eager load relationships with comprehensive audit user support. Available: provinces, createdBy, updatedBy, deletedBy, createdBy.organizations, updatedBy.organizations, deletedBy.organizations, createdBy.organizations.position, updatedBy.organizations.position, deletedBy.organizations.position, createdBy.organizations.position.level, updatedBy.organizations.position.level, deletedBy.organizations.position.level. Supports deep nested relationships. Example: 'provinces,createdBy.organizations.position.level'"),
        ("filter" = Option<serde_json::Value>, Query, description = "Advanced filtering with operators. Available filters: name, iso_code, phone_code, created_at, updated_at. Operators: eq, ne, gt, gte, lt, lte, like, ilike, contains, starts_with, ends_with, in, not_in, is_null, is_not_null, between. Examples: filter[name][contains]=united, filter[iso_code][in]=US,CA,GB, filter[created_at][gte]=2023-01-01"),
        ("fields" = Option<String>, Query, description = "Field selection for optimized responses. Available: id, name, iso_code, phone_code, created_at, updated_at. Example: fields[countries]=id,name,iso_code"),
        ("cursor" = Option<String>, Query, description = "Cursor for high-performance pagination. Base64-encoded JSON cursor from previous response"),
        ("pagination_type" = Option<String>, Query, description = "Pagination strategy: 'offset' (traditional page/per_page) or 'cursor' (high-performance, default)"),
    ),
    responses(
        (status = 200, description = "List of countries", body = Vec<crate::app::models::country::CountryResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    match <Country as QueryBuilderService<Country>>::index(Query(params), &pool) {
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
    path = "/api/countries/{id}",
    tag = "Countries",
    summary = "Get country by ID",
    description = "Retrieve a specific country by its unique identifier",
    params(
        ("id" = String, Path, description = "Country unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Country details", body = crate::app::models::country::CountryResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Country not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn show(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    match CountryService::find_by_id(&pool, id) {
        Ok(Some(country)) => (StatusCode::OK, ResponseJson(country.to_response())).into_response(),
        Ok(None) => {
            let error = ErrorResponse {
                error: "Country not found".to_string(),
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

/// Create a new country
///
/// Create a new country with the provided information. All required fields must be provided
/// and will be validated according to the business rules.
///
/// # Request Body
/// The request must contain a valid CreateCountryRequest JSON payload with:
/// - `name`: Country name (2-100 characters)
/// - `iso_code`: ISO country code (2-3 uppercase letters)
/// - `phone_code`: Optional phone country code with + prefix
///
/// # Example
/// ```json
/// {
///   "name": "United States",
///   "iso_code": "US",
///   "phone_code": "+1"
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/countries",
    tag = "Countries",
    summary = "Create new country",
    description = "Create a new country with the provided information",
    request_body = crate::app::http::requests::CreateCountryRequest,
    responses(
        (status = 201, description = "Country created successfully", body = crate::app::models::country::CountryResponse),
        (status = 400, description = "Validation error or bad request", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn store(State(pool): State<DbPool>, request: CreateCountryRequest) -> impl IntoResponse {
    // Get system user ID for audit trail
    use diesel::prelude::*;
    use crate::schema::sys_users;

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Database connection error: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    let system_user_id: String = match sys_users::table
        .filter(sys_users::email.eq("system@seeder.internal"))
        .select(sys_users::id)
        .first(&mut conn)
    {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "System user not found. Please run migrations and seeders first.".to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    let payload = CreateCountry {
        name: request.name,
        iso_code: request.iso_code,
        phone_code: request.phone_code,
    };

    match CountryService::create(&pool, payload, &system_user_id).await {
        Ok(country) => (StatusCode::CREATED, ResponseJson(country.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Update an existing country
///
/// Update an existing country with the provided information. All fields are optional
/// for partial updates. Only provided fields will be updated.
///
/// # Path Parameters
/// - `id`: The unique identifier of the country to update (ULID format)
///
/// # Request Body
/// The request should contain an UpdateCountryRequest JSON payload with optional fields:
/// - `name`: Updated country name (2-100 characters)
/// - `iso_code`: Updated ISO country code (2-3 uppercase letters)
/// - `phone_code`: Updated phone country code with + prefix
#[utoipa::path(
    put,
    path = "/api/countries/{id}",
    tag = "Countries",
    summary = "Update country",
    description = "Update an existing country with the provided information",
    params(
        ("id" = String, Path, description = "Country unique identifier (ULID format)")
    ),
    request_body = crate::app::http::requests::UpdateCountryRequest,
    responses(
        (status = 200, description = "Country updated successfully", body = crate::app::models::country::CountryResponse),
        (status = 400, description = "Invalid ID format or validation error", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Country not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    request: UpdateCountryRequest,
) -> impl IntoResponse {
    let payload = UpdateCountry {
        name: request.name,
        iso_code: request.iso_code,
        phone_code: request.phone_code,
    };

    match CountryService::update(&pool, id, payload) {
        Ok(country) => (StatusCode::OK, ResponseJson(country.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Delete a country
///
/// Permanently delete a country from the system. This action cannot be undone.
///
/// # Path Parameters
/// - `id`: The unique identifier of the country to delete (ULID format)
#[utoipa::path(
    delete,
    path = "/api/countries/{id}",
    tag = "Countries",
    summary = "Delete country",
    description = "Permanently delete a country from the system",
    params(
        ("id" = String, Path, description = "Country unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "Country deleted successfully", body = crate::app::docs::MessageResponse),
        (status = 400, description = "Invalid ID format", body = crate::app::docs::ErrorResponse),
        (status = 404, description = "Country not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn destroy(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    match CountryService::delete(&pool, id) {
        Ok(_) => {
            let message = MessageResponse {
                message: "Country deleted successfully".to_string(),
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