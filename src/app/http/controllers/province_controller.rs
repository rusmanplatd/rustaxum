use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::Serialize;
use ulid::Ulid;
use crate::database::DbPool;
use std::collections::HashMap;

use crate::app::models::province::{CreateProvince, UpdateProvince};
use crate::app::services::province_service::ProvinceService;
use crate::app::http::requests::{CreateProvinceRequest, UpdateProvinceRequest};

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

pub async fn index(
    State(pool): State<DbPool>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    match ProvinceService::list(&pool, params) {
        Ok(provinces) => {
            let responses: Vec<_> = provinces.into_iter().map(|p| p.to_response()).collect();
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

pub async fn show(State(pool): State<DbPool>, Path(id): Path<String>) -> impl IntoResponse {
    let province_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match ProvinceService::find_by_id(&pool, province_id.to_string()) {
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
    let province_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    let payload = UpdateProvince {
        country_id: request.country_id,
        name: request.name,
        code: request.code,
    };

    match ProvinceService::update(&pool, province_id.to_string(), payload) {
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
    let province_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match ProvinceService::delete(&pool, province_id) {
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

pub async fn by_country(State(pool): State<DbPool>, Path(country_id): Path<String>) -> impl IntoResponse {
    match ProvinceService::find_by_country_id(&pool, country_id) {
        Ok(provinces) => {
            let responses: Vec<_> = provinces.into_iter().map(|p| p.to_response()).collect();
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