use axum::{
    extract::Query,
    http::{StatusCode, HeaderMap},
    response::Json,
    Extension,
};
use serde::Deserialize;
use std::collections::HashMap;
use crate::app::services::web_push_service::{
    WebPushService, SubscribeRequest, SubscriptionResponse, VapidPublicKeyResponse
};
use crate::app::models::user::User;

/// Query parameters for test notification
#[derive(Debug, Deserialize)]
pub struct TestNotificationQuery {
    pub title: Option<String>,
    pub message: Option<String>,
}

/// Get VAPID public key for client-side subscription
/// GET /api/web-push/vapid-public-key
pub async fn get_vapid_public_key() -> Result<Json<VapidPublicKeyResponse>, StatusCode> {
    match WebPushService::get_vapid_public_key() {
        Ok(public_key) => Ok(Json(VapidPublicKeyResponse { public_key })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Subscribe user to web push notifications
/// POST /api/web-push/subscribe
pub async fn subscribe(
    Extension(user): Extension<User>,
    headers: HeaderMap,
    Json(request): Json<SubscribeRequest>,
) -> Result<Json<SubscriptionResponse>, StatusCode> {
    let service = WebPushService::new().await;
    let user_id = user.id.to_string();

    // Extract user agent from request headers if available
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match service.subscribe(&user_id, request, user_agent).await {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Unsubscribe user from web push notifications
/// DELETE /api/web-push/unsubscribe
pub async fn unsubscribe(
    Extension(user): Extension<User>,
    Json(payload): Json<HashMap<String, String>>,
) -> Result<Json<SubscriptionResponse>, StatusCode> {
    let service = WebPushService::new().await;
    let user_id = user.id.to_string();

    let endpoint = match payload.get("endpoint") {
        Some(endpoint) => endpoint,
        None => {
            return Ok(Json(SubscriptionResponse {
                success: false,
                message: "Endpoint required".to_string(),
                subscription_id: None,
            }));
        }
    };

    match service.unsubscribe(&user_id, endpoint).await {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get user's push subscriptions
/// GET /api/web-push/subscriptions
pub async fn get_subscriptions(
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let service = WebPushService::new().await;
    let user_id = user.id.to_string();

    match service.get_user_subscriptions(&user_id).await {
        Ok(subscriptions) => {
            let response = serde_json::json!({
                "success": true,
                "subscriptions": subscriptions.into_iter().map(|sub| {
                    serde_json::json!({
                        "id": sub.id.to_string(),
                        "endpoint": sub.endpoint,
                        "user_agent": sub.user_agent,
                        "created_at": sub.created_at,
                        "updated_at": sub.updated_at
                    })
                }).collect::<Vec<_>>()
            });
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Send a test notification to the authenticated user
/// POST /api/web-push/test
pub async fn send_test_notification(
    Extension(user): Extension<User>,
    Query(query): Query<TestNotificationQuery>,
) -> Result<Json<SubscriptionResponse>, StatusCode> {
    let service = WebPushService::new().await;
    let user_id = user.id.to_string();

    let title = query.title.unwrap_or_else(|| "Test Notification".to_string());
    let message = query.message.unwrap_or_else(|| "This is a test push notification from RustAxum!".to_string());

    match service.send_test_notification(&user_id, &title, &message).await {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Check web push configuration status
/// GET /api/web-push/status
pub async fn get_status() -> Result<Json<serde_json::Value>, StatusCode> {
    let service = WebPushService::new().await;

    match service.test_web_push_configuration() {
        Ok(is_configured) => {
            let response = serde_json::json!({
                "success": true,
                "configured": is_configured,
                "message": if is_configured {
                    "Web push is properly configured"
                } else {
                    "Web push is not configured. Please set VAPID keys."
                }
            });
            Ok(Json(response))
        }
        Err(e) => {
            let response = serde_json::json!({
                "success": false,
                "configured": false,
                "message": format!("Configuration error: {}", e)
            });
            Ok(Json(response))
        }
    }
}

/// Admin endpoint to clean up invalid subscriptions
/// POST /api/web-push/cleanup
pub async fn cleanup_subscriptions() -> Result<Json<serde_json::Value>, StatusCode> {
    let service = WebPushService::new().await;

    match service.cleanup_invalid_subscriptions().await {
        Ok(cleaned_count) => {
            let response = serde_json::json!({
                "success": true,
                "cleaned_subscriptions": cleaned_count,
                "message": format!("Cleaned up {} invalid subscriptions", cleaned_count)
            });
            Ok(Json(response))
        }
        Err(e) => {
            let response = serde_json::json!({
                "success": false,
                "message": format!("Cleanup failed: {}", e)
            });
            Ok(Json(response))
        }
    }
}