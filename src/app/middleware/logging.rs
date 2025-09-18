use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use std::collections::HashMap;
use serde_json::Value;
use tower_http::trace::TraceLayer;
use crate::logging::Log;

pub async fn request_logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();

    // Get user agent and IP if available
    let user_agent = request.headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown")
        .to_string();

    let ip = request.headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown")
        .to_string();

    // Log incoming request
    let mut request_context = HashMap::new();
    request_context.insert("method".to_string(), Value::String(method.to_string()));
    request_context.insert("uri".to_string(), Value::String(uri.to_string()));
    request_context.insert("version".to_string(), Value::String(format!("{:?}", version)));
    request_context.insert("user_agent".to_string(), Value::String(user_agent));
    request_context.insert("ip".to_string(), Value::String(ip));

    Log::info_with_context("Incoming request", request_context);

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    // Log response
    let mut response_context = HashMap::new();
    response_context.insert("method".to_string(), Value::String(method.to_string()));
    response_context.insert("uri".to_string(), Value::String(uri.to_string()));
    response_context.insert("status".to_string(), Value::Number(serde_json::Number::from(status.as_u16())));
    response_context.insert("duration_ms".to_string(), Value::Number(
        serde_json::Number::from_f64(duration.as_secs_f64() * 1000.0).unwrap_or(serde_json::Number::from(0))
    ));

    let message = format!("Request completed: {} {} - {} in {:.2}ms",
        method, uri, status, duration.as_secs_f64() * 1000.0);

    if status.is_server_error() {
        Log::error_with_context(&message, response_context);
    } else if status.is_client_error() {
        Log::warning_with_context(&message, response_context);
    } else {
        Log::info_with_context(&message, response_context);
    }

    response
}

pub fn logging_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    tower_http::trace::DefaultMakeSpan,
    tower_http::trace::DefaultOnRequest,
    tower_http::trace::DefaultOnResponse,
    tower_http::trace::DefaultOnBodyChunk,
    tower_http::trace::DefaultOnEos,
    tower_http::trace::DefaultOnFailure,
> {
    TraceLayer::new_for_http()
}