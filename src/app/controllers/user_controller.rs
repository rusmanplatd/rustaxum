use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Serialize;

#[derive(Serialize)]
pub struct User {
    id: String,
    name: String,
    email: String,
}

pub async fn index() -> impl IntoResponse {
    let users = vec![
        User {
            id: "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        },
        User {
            id: "01ARZ3NDEKTSV4RRFFQ69G5FB0".to_string(),
            name: "Jane Smith".to_string(),
            email: "jane@example.com".to_string(),
        },
    ];

    (StatusCode::OK, Json(users))
}

pub async fn show(Path(id): Path<String>) -> impl IntoResponse {
    let user = User {
        id,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };

    (StatusCode::OK, Json(user))
}