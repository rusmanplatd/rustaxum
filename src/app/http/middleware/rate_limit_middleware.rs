use axum::{
    extract::{Request, ConnectInfo},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::{Response, IntoResponse},
    Json,
};
use serde_json::json;
use std::{sync::Arc, net::SocketAddr};
use crate::app::utils::{RateLimiter, RateLimitError};
use crate::app::http::middleware::activity_logging_middleware::activity_logger_from_request;

/// Rate limiting middleware state
#[derive(Clone)]
pub struct RateLimitState {
    pub subscription_limiter: Arc<RateLimiter>,
    pub notification_limiter: Arc<RateLimiter>,
    pub api_limiter: Arc<RateLimiter>,
}

impl RateLimitState {
    pub fn new() -> Self {
        Self {
            subscription_limiter: Arc::new(RateLimiter::for_web_push_subscription()),
            notification_limiter: Arc::new(RateLimiter::for_notification_sending()),
            api_limiter: Arc::new(RateLimiter::for_api_endpoints()),
        }
    }
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self::new()
    }
}

/// Web push subscription rate limiting middleware
pub async fn web_push_subscription_rate_limit(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get rate limiter from app state (this would need to be properly injected)
    let rate_limiter = RateLimiter::for_web_push_subscription();

    // Use IP address as identifier, with user ID as fallback if available
    let identifier = extract_identifier(&addr, &headers).await;

    // Create activity logger for this request
    let logger = activity_logger_from_request(&request, "rate_limiting");

    // Check rate limit
    match rate_limiter.check_rate_limit(&identifier) {
        Ok(_) => {
            let response = next.run(request).await;
            Ok(response)
        }
        Err(RateLimitError::Exceeded { seconds }) => {
            // Log rate limit violation
            let properties = json!({
                "rate_limit_type": "web_push_subscription",
                "identifier": identifier,
                "client_ip": addr.ip().to_string(),
                "retry_after_seconds": seconds,
                "path": request.uri().path(),
                "method": request.method().as_str()
            });

            tokio::spawn(async move {
                if let Err(e) = logger.log_custom(
                    &format!("Rate limit exceeded for web push subscription: {}", identifier),
                    Some("security.rate_limit_exceeded"),
                    Some(properties)
                ).await {
                    eprintln!("Failed to log rate limit violation: {}", e);
                }
            });

            let error_response = Json(json!({
                "success": false,
                "error": "rate_limit_exceeded",
                "message": format!("Too many subscription requests. Try again in {} seconds.", seconds),
                "retry_after": seconds
            }));

            let mut response = Response::new(error_response.into_response().into_body());
            *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
            response.headers_mut().insert("Retry-After", seconds.to_string().parse().unwrap());
            Ok(response)
        }
        Err(RateLimitError::Internal(msg)) => {
            tracing::error!("Rate limiter internal error: {}", msg);

            // Log internal rate limiter error
            let properties = json!({
                "rate_limit_type": "web_push_subscription",
                "error": msg,
                "identifier": identifier,
                "client_ip": addr.ip().to_string()
            });

            tokio::spawn(async move {
                if let Err(e) = logger.log_custom(
                    &format!("Rate limiter internal error: {}", msg),
                    Some("system.rate_limiter_error"),
                    Some(properties)
                ).await {
                    eprintln!("Failed to log rate limiter error: {}", e);
                }
            });

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Web push notification sending rate limiting middleware
pub async fn web_push_notification_rate_limit(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let rate_limiter = RateLimiter::for_notification_sending();
    let identifier = extract_identifier(&addr, &headers).await;

    match rate_limiter.check_rate_limit(&identifier) {
        Ok(_) => {
            let response = next.run(request).await;
            Ok(response)
        }
        Err(RateLimitError::Exceeded { seconds }) => {
            let error_response = Json(json!({
                "success": false,
                "error": "rate_limit_exceeded",
                "message": format!("Too many notification requests. Try again in {} seconds.", seconds),
                "retry_after": seconds
            }));

            let mut response = Response::new(error_response.into_response().into_body());
            *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
            response.headers_mut().insert("Retry-After", seconds.to_string().parse().unwrap());
            Ok(response)
        }
        Err(RateLimitError::Internal(msg)) => {
            tracing::error!("Rate limiter internal error: {}", msg);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// General API rate limiting middleware
pub async fn api_rate_limit(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let rate_limiter = RateLimiter::for_api_endpoints();
    let identifier = extract_identifier(&addr, &headers).await;

    match rate_limiter.check_rate_limit(&identifier) {
        Ok(_) => {
            let response = next.run(request).await;
            Ok(response)
        }
        Err(RateLimitError::Exceeded { seconds }) => {
            let error_response = Json(json!({
                "success": false,
                "error": "rate_limit_exceeded",
                "message": format!("Too many API requests. Try again in {} seconds.", seconds),
                "retry_after": seconds
            }));

            let mut response = Response::new(error_response.into_response().into_body());
            *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
            response.headers_mut().insert("Retry-After", seconds.to_string().parse().unwrap());
            Ok(response)
        }
        Err(RateLimitError::Internal(msg)) => {
            tracing::error!("Rate limiter internal error: {}", msg);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Extract identifier for rate limiting from IP and headers
async fn extract_identifier(addr: &SocketAddr, headers: &HeaderMap) -> String {
    // Check for forwarded IP headers (for reverse proxy setups)
    if let Some(forwarded_for) = headers.get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    if let Some(real_ip) = headers.get("X-Real-IP") {
        if let Ok(real_ip_str) = real_ip.to_str() {
            return real_ip_str.to_string();
        }
    }

    // Fallback to direct connection IP
    addr.ip().to_string()
}

/// Rate limiting statistics endpoint handler
pub async fn rate_limit_stats() -> Result<Json<serde_json::Value>, StatusCode> {
    let subscription_limiter = RateLimiter::for_web_push_subscription();
    let notification_limiter = RateLimiter::for_notification_sending();
    let api_limiter = RateLimiter::for_api_endpoints();

    let stats = json!({
        "subscription_rate_limits": subscription_limiter.get_stats(),
        "notification_rate_limits": notification_limiter.get_stats(),
        "api_rate_limits": api_limiter.get_stats(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Ok(Json(stats))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_extract_identifier_with_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Forwarded-For", "192.168.1.1, 10.0.0.1".parse().unwrap());

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let identifier = extract_identifier(&addr, &headers).await;

        assert_eq!(identifier, "192.168.1.1");
    }

    #[tokio::test]
    async fn test_extract_identifier_with_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Real-IP", "203.0.113.1".parse().unwrap());

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let identifier = extract_identifier(&addr, &headers).await;

        assert_eq!(identifier, "203.0.113.1");
    }

    #[tokio::test]
    async fn test_extract_identifier_fallback_to_addr() {
        let headers = HeaderMap::new();
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 8080);
        let identifier = extract_identifier(&addr, &headers).await;

        assert_eq!(identifier, "192.168.1.100");
    }
}