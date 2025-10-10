use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::{Serialize, Deserialize};
use crate::database::DbPool;
use rust_decimal::Decimal;

use crate::app::models::city::{CreateCity, UpdateCity, City};
use crate::app::services::city_service::CityService;
use crate::app::http::requests::{CreateCityRequest, UpdateCityRequest};
use crate::app::query_builder::{QueryParams, QueryBuilderService};

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

#[derive(Deserialize)]
pub struct NearbyQuery {
    lat: Decimal,
    lng: Decimal,
    radius: Option<Decimal>,
}

#[utoipa::path(
    get,
    path = "/api/cities",
    tag = "Cities",
    summary = "List all cities",
    description = "Get all cities with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Multi-column sorting with geospatial support. Available fields: id, name, province_id, latitude, longitude, population, created_at, updated_at. Syntax: 'field1,-field2,field3:desc'. Example: 'population:desc,name,-created_at'"),
        ("include" = Option<String>, Query, description = "Eager load relationships. Available: province, province.country, districts, districts.villages, createdBy, updatedBy, deletedBy, createdBy.organizations.position.level, updatedBy.organizations.position.level, deletedBy.organizations.position.level. Supports nested relationships. Example: 'province.country,districts,createdBy.organizations.position.level'"),
        ("filter" = Option<serde_json::Value>, Query, description = "Advanced filtering with geospatial operators. Available filters: name, province_id, latitude, longitude, population, created_at, updated_at. Operators: eq, ne, gt, gte, lt, lte, like, ilike, contains, starts_with, ends_with, in, not_in, is_null, is_not_null, between. Geospatial examples: filter[latitude][between]=-90,90, filter[population][gte]=100000, filter[name][starts_with]=New"),
        ("fields" = Option<String>, Query, description = "Field selection for optimized responses. Available: id, name, province_id, latitude, longitude, population, timezone, created_at, updated_at. Example: fields[cities]=id,name,latitude,longitude"),
        ("cursor" = Option<String>, Query, description = "Cursor for high-performance pagination with geospatial indexing support"),
        ("pagination_type" = Option<String>, Query, description = "Pagination strategy: 'offset' (traditional) or 'cursor' (high-performance with spatial indexing, default)"),
    ),
    responses(
        (status = 200, description = "List of cities", body = Vec<crate::app::models::city::CityResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    match <City as QueryBuilderService<City>>::index(Query(params), &pool) {
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

pub async fn show(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    match CityService::find_by_id(&pool, id) {
        Ok(Some(city)) => (StatusCode::OK, ResponseJson(city.to_response())).into_response(),
        Ok(None) => {
            let error = ErrorResponse {
                error: "City not found".to_string(),
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

pub async fn store(State(pool): State<DbPool>, request: CreateCityRequest) -> impl IntoResponse {
    use diesel::prelude::*;
    use crate::schema::sys_users;

    let payload = CreateCity {
        province_id: request.province_id,
        name: request.name,
        code: request.code,
        latitude: request.latitude,
        longitude: request.longitude,
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

    match CityService::create(&pool, payload, &system_user_id) {
        Ok(city) => (StatusCode::CREATED, ResponseJson(city.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

pub async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    request: UpdateCityRequest,
) -> impl IntoResponse {
    let payload = UpdateCity {
        province_id: request.province_id,
        name: request.name,
        code: request.code,
        latitude: request.latitude,
        longitude: request.longitude,
    };

    match CityService::update(&pool, id, payload) {
        Ok(city) => (StatusCode::OK, ResponseJson(city.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

pub async fn destroy(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    match CityService::delete(&pool, id) {
        Ok(_) => {
            let message = MessageResponse {
                message: "City deleted successfully".to_string(),
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

#[utoipa::path(
    get,
    path = "/api/provinces/{province_id}/cities",
    tag = "Cities",
    summary = "List cities by province",
    description = "Get all cities for a specific province with optional filtering, sorting, and pagination",
    params(
        ("province_id" = String, Path, description = "Province unique identifier (ULID format)"),
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Sort field and direction. Available fields: id, name, latitude, longitude, created_at, updated_at (prefix with '-' for descending)"),
        ("include" = Option<String>, Query, description = "Comma-separated list of relationships to include. Available: none for filtered results"),
        ("filter" = Option<serde_json::Value>, Query, description = "Filter parameters. Available filters: name, latitude, longitude (e.g., filter[name]=Toronto)"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to select. Available: id, name, latitude, longitude, created_at, updated_at"),
        ("cursor" = Option<String>, Query, description = "Cursor for cursor-based pagination"),
        ("pagination_type" = Option<String>, Query, description = "Pagination type: 'offset' or 'cursor' (default: cursor)"),
    ),
    responses(
        (status = 200, description = "List of cities for the province", body = Vec<crate::app::models::city::CityResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn by_province(
    State(pool): State<DbPool>,
    Path(province_id): Path<String>,
    Query(mut params): Query<QueryParams>,
) -> impl IntoResponse {
    // Add province_id filter to the query parameters
    params.filter.insert("province_id".to_string(), serde_json::json!(province_id));

    match <City as QueryBuilderService<City>>::index(Query(params), &pool) {
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

pub async fn nearby(
    State(pool): State<DbPool>,
    Query(query): Query<NearbyQuery>,
) -> impl IntoResponse {
    let radius = query.radius.unwrap_or(Decimal::from(10)); // Default 10km radius

    match CityService::find_by_coordinates(&pool, query.lat, query.lng, radius) {
        Ok(cities) => {
            let responses: Vec<_> = cities.into_iter().map(|c| c.to_response()).collect();
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