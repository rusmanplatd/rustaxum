use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    name: String,
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    token: String,
    user: UserResponse,
}

#[derive(Serialize)]
pub struct UserResponse {
    id: String,
    name: String,
    email: String,
}

pub async fn login(Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    // TODO: Implement actual authentication logic
    let response = AuthResponse {
        token: "dummy-jwt-token".to_string(),
        user: UserResponse {
            id: "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
            name: "John Doe".to_string(),
            email: payload.email,
        },
    };

    (StatusCode::OK, ResponseJson(response))
}

pub async fn register(Json(payload): Json<RegisterRequest>) -> impl IntoResponse {
    // TODO: Implement actual registration logic
    let response = AuthResponse {
        token: "dummy-jwt-token".to_string(),
        user: UserResponse {
            id: "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
            name: payload.name,
            email: payload.email,
        },
    };

    (StatusCode::CREATED, ResponseJson(response))
}