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
    let payload = CreateCity {
        province_id: request.province_id,
        name: request.name,
        code: request.code,
        latitude: request.latitude,
        longitude: request.longitude,
    };

    match CityService::create(&pool, payload) {
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