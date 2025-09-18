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