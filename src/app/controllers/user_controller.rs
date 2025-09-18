use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sqlx::PgPool;
use crate::query_builder::{QueryBuilder, QueryParams};
use crate::app::models::user::User;

pub async fn index(
    State(pool): State<PgPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    let request = params.parse();
    let query_builder = QueryBuilder::<User>::new(pool, request);

    match query_builder.paginate().await {
        Ok(result) => {
            (StatusCode::OK, Json(serde_json::json!(result))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch users"
            }))).into_response()
        }
    }
}

pub async fn show(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let query = "SELECT * FROM users WHERE id = $1";

    match sqlx::query_as::<_, User>(query)
        .bind(&id)
        .fetch_optional(&pool)
        .await
    {
        Ok(Some(user)) => {
            (StatusCode::OK, Json(serde_json::json!(user.to_response()))).into_response()
        },
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "User not found"
            }))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch user"
            }))).into_response()
        }
    }
}