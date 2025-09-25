use axum::{
    extract::Extension,
    response::{IntoResponse, Json},
    http::StatusCode,
};
use serde_json::{json, Value};

use crate::app::services::session::SessionStore;

pub async fn put_session(
    Extension(session): Extension<SessionStore>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some(obj) = payload.as_object() {
        for (key, value) in obj {
            session.put(key, value.clone()).await;
        }
    }

    Json(json!({
        "success": true,
        "message": "Session data stored successfully"
    }))
}

pub async fn get_session(
    Extension(session): Extension<SessionStore>,
) -> impl IntoResponse {
    let session_id = session.get_session_id().await;

    Json(json!({
        "session_id": session_id,
        "csrf_token": session.token().await
    }))
}

pub async fn get_session_value(
    axum::extract::Path(key): axum::extract::Path<String>,
    Extension(session): Extension<SessionStore>,
) -> impl IntoResponse {
    match session.get(&key).await {
        Some(value) => (StatusCode::OK, Json(json!({
            "key": key,
            "value": value,
            "exists": true
        }))),
        None => (StatusCode::NOT_FOUND, Json(json!({
            "key": key,
            "value": null,
            "exists": false
        })))
    }
}

pub async fn forget_session_value(
    axum::extract::Path(key): axum::extract::Path<String>,
    Extension(session): Extension<SessionStore>,
) -> impl IntoResponse {
    let value = session.forget(&key).await;

    Json(json!({
        "key": key,
        "previous_value": value,
        "removed": value.is_some()
    }))
}

pub async fn flush_session(
    Extension(session): Extension<SessionStore>,
) -> impl IntoResponse {
    session.flush().await;

    Json(json!({
        "success": true,
        "message": "Session data cleared successfully"
    }))
}

pub async fn regenerate_session(
    Extension(session): Extension<SessionStore>,
) -> impl IntoResponse {
    match session.regenerate().await {
        Ok(_) => {
            let new_session_id = session.get_session_id().await;
            Json(json!({
                "success": true,
                "message": "Session ID regenerated successfully",
                "session_id": new_session_id
            }))
        }
        Err(_) => Json(json!({
            "success": false,
            "message": "Failed to regenerate session ID"
        }))
    }
}

pub async fn flash_session(
    Extension(session): Extension<SessionStore>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some(obj) = payload.as_object() {
        for (key, value) in obj {
            session.flash(key, value.clone()).await;
        }
    }

    Json(json!({
        "success": true,
        "message": "Flash data stored successfully"
    }))
}

pub async fn regenerate_token(
    Extension(session): Extension<SessionStore>,
) -> impl IntoResponse {
    if let Some(token) = session.regenerate_token().await {
        Json(json!({
            "success": true,
            "csrf_token": token
        }))
    } else {
        Json(json!({
            "success": false,
            "message": "Failed to regenerate CSRF token"
        }))
    }
}