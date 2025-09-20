use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::Serialize;
use ulid::Ulid;
use sqlx::PgPool;
use std::collections::HashMap;

use crate::app::models::country::{CreateCountry, UpdateCountry};
use crate::app::services::country_service::CountryService;
use crate::app::http::requests::{CreateCountryRequest, UpdateCountryRequest};

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

/// Get all countries with optional filtering and pagination
///
/// Retrieve a list of all countries with support for filtering and pagination.
/// You can filter by any field and sort by any column.
///
/// # Query Parameters
/// - `page`: Page number for pagination (default: 1)
/// - `limit`: Number of items per page (default: 10, max: 100)
/// - `sort`: Sort field (default: name)
/// - `direction`: Sort direction - asc or desc (default: asc)
/// - `filter[field]`: Filter by field value
///
/// # Example
/// ```
/// GET /api/countries?page=1&limit=10&sort=name&direction=asc&filter[name]=united
/// ```
#[utoipa::path(
    get,
    path = "/api/countries",
    tag = "Countries",
    summary = "List all countries",
    description = "Get all countries with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination"),
        ("limit" = Option<u32>, Query, description = "Number of items per page (max 100)"),
        ("sort" = Option<String>, Query, description = "Sort field"),
        ("direction" = Option<String>, Query, description = "Sort direction (asc/desc)"),
    ),
    responses(
        (status = 200, description = "List of countries", body = Vec<crate::app::models::country::CountryResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<PgPool>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    match CountryService::list(&pool, params).await {
        Ok(countries) => {
            let responses: Vec<_> = countries.into_iter().map(|c| c.to_response()).collect();
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

/// Get a specific country by ID
///
/// Retrieve detailed information about a specific country using its unique identifier.
/// The ID should be a valid ULID format.
///
/// # Path Parameters
/// - `id`: The unique identifier of the country (ULID format)
///
/// # Example
/// ```
/// GET /api/countries/01ARZ3NDEKTSV4RRFFQ69G5FAV
/// ```
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
pub async fn show(State(pool): State<PgPool>, Path(id): Path<String>) -> impl IntoResponse {
    let country_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match CountryService::find_by_id(&pool, country_id).await {
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
pub async fn store(State(pool): State<PgPool>, request: CreateCountryRequest) -> impl IntoResponse {
    let payload = CreateCountry {
        name: request.name,
        iso_code: request.iso_code,
        phone_code: request.phone_code,
    };

    match CountryService::create(&pool, payload).await {
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
    State(pool): State<PgPool>,
    Path(id): Path<String>,
    request: UpdateCountryRequest,
) -> impl IntoResponse {
    let country_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    let payload = UpdateCountry {
        name: request.name,
        iso_code: request.iso_code,
        phone_code: request.phone_code,
    };

    match CountryService::update(&pool, country_id, payload).await {
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
pub async fn destroy(State(pool): State<PgPool>, Path(id): Path<String>) -> impl IntoResponse {
    let country_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match CountryService::delete(&pool, country_id).await {
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