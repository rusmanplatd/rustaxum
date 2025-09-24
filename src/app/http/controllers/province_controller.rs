use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::Serialize;
use ulid::Ulid;
use crate::database::DbPool;

use crate::app::models::province::{CreateProvince, UpdateProvince, Province};
use crate::app::services::province_service::ProvinceService;
use crate::app::http::requests::{CreateProvinceRequest, UpdateProvinceRequest};
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
    path = "/api/provinces",
    tag = "Provinces",
    summary = "List all provinces",
    description = "Get all provinces with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Sort field and direction. Available fields: id, name, code, country_id, created_at, updated_at (prefix with '-' for descending)"),
        ("include" = Option<String>, Query, description = "Comma-separated list of relationships to include. Available: country, cities"),
        ("filter" = Option<serde_json::Value>, Query, description = "Filter parameters. Available filters: name, code, country_id (e.g., filter[name]=Ontario, filter[code]=ON)"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to select. Available: id, name, code, country_id, created_at, updated_at"),
        ("cursor" = Option<String>, Query, description = "Cursor for cursor-based pagination"),
        ("pagination_type" = Option<String>, Query, description = "Pagination type: 'offset' or 'cursor' (default: cursor)"),
    ),
    responses(
        (status = 200, description = "List of provinces", body = Vec<crate::app::models::province::ProvinceResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    match <Province as QueryBuilderService<Province>>::index(Query(params), &pool) {
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
    match ProvinceService::find_by_id(&pool, id) {
        Ok(Some(province)) => (StatusCode::OK, ResponseJson(province.to_response())).into_response(),
        Ok(None) => {
            let error = ErrorResponse {
                error: "Province not found".to_string(),
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

pub async fn store(State(pool): State<DbPool>, request: CreateProvinceRequest) -> impl IntoResponse {
    let payload = CreateProvince {
        country_id: request.country_id,
        name: request.name,
        code: request.code,
    };

    match ProvinceService::create(&pool, payload) {
        Ok(province) => (StatusCode::CREATED, ResponseJson(province.to_response())).into_response(),
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
    request: UpdateProvinceRequest,
) -> impl IntoResponse {
    let payload = UpdateProvince {
        country_id: request.country_id,
        name: request.name,
        code: request.code,
    };

    match ProvinceService::update(&pool, id, payload) {
        Ok(province) => (StatusCode::OK, ResponseJson(province.to_response())).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

pub async fn destroy(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    let _province_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match ProvinceService::delete(&pool, id) {
        Ok(_) => {
            let message = MessageResponse {
                message: "Province deleted successfully".to_string(),
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
    path = "/api/countries/{country_id}/provinces",
    tag = "Provinces",
    summary = "List provinces by country",
    description = "Get all provinces for a specific country with optional filtering, sorting, and pagination",
    params(
        ("country_id" = String, Path, description = "Country unique identifier (ULID format)"),
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Sort field and direction. Available fields: id, name, code, created_at, updated_at (prefix with '-' for descending)"),
        ("include" = Option<String>, Query, description = "Comma-separated list of relationships to include. Available: cities"),
        ("filter" = Option<serde_json::Value>, Query, description = "Filter parameters. Available filters: name, code (e.g., filter[name]=Ontario, filter[code]=ON)"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to select. Available: id, name, code, created_at, updated_at"),
        ("cursor" = Option<String>, Query, description = "Cursor for cursor-based pagination"),
        ("pagination_type" = Option<String>, Query, description = "Pagination type: 'offset' or 'cursor' (default: cursor)"),
    ),
    responses(
        (status = 200, description = "List of provinces for the country", body = Vec<crate::app::models::province::ProvinceResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn by_country(
    State(pool): State<DbPool>,
    Path(country_id): Path<String>,
    Query(mut params): Query<QueryParams>,
) -> impl IntoResponse {
    // Add country_id filter to the query parameters
    params.filter.insert("country_id".to_string(), serde_json::json!(country_id));

    match <Province as QueryBuilderService<Province>>::index(Query(params), &pool) {
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