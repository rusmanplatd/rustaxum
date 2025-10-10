use axum::{
    body::Bytes,
    extract::{Request, MatchedPath},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use serde_json::{json, Value};
use std::time::Instant;

use crate::app::http::middleware::correlation_middleware::CorrelationContext;
use crate::app::models::activity_log::ActivityLog;
use crate::app::services::activity_log_service::ActivityLogService;
use crate::app::models::DieselUlid;
use crate::logging::Log;

/// Middleware for automatic activity logging of HTTP requests
pub async fn activity_logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();

    // Extract correlation context if available (with request data)
    let correlation_context = request.extensions()
        .get::<CorrelationContext>()
        .cloned();

    // Extract matched path pattern if available
    let matched_path = request.extensions()
        .get::<MatchedPath>()
        .map(|mp| mp.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());

    // Capture request body for logging (limit to first 1KB for safety)
    let (parts, body) = request.into_parts();
    let body_bytes = axum::body::to_bytes(body, 1024).await.unwrap_or_default();
    let body_preview = if !body_bytes.is_empty() {
        extract_body_preview(&body_bytes, parts.headers.get("content-type"))
    } else {
        None
    };

    // Reconstruct request with captured body
    let request = Request::from_parts(parts, axum::body::Body::from(body_bytes.clone()));

    // Call the next handler
    let response = next.run(request).await;

    // Calculate request duration
    let duration = start_time.elapsed();
    let duration_ms = duration.as_millis() as u64;

    // Extract status code before moving response
    let status_code = response.status();

    // Log the activity asynchronously (fire and forget)
    tokio::spawn(async move {
        if let Err(e) = log_comprehensive_http_activity(
            correlation_context,
            matched_path,
            body_preview,
            status_code,
            duration_ms,
        ).await {
            tracing::error!("Failed to log HTTP activity to database: {}", e);
        }
    });

    response
}

/// Extract body preview for logging (limit content for safety)
fn extract_body_preview(body_bytes: &Bytes, content_type: Option<&axum::http::HeaderValue>) -> Option<String> {
    if body_bytes.is_empty() {
        return None;
    }

    let content_type_str = content_type
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("");

    // Limit body preview to 1KB
    let preview_limit = 1024;
    let limited_bytes = if body_bytes.len() > preview_limit {
        &body_bytes[..preview_limit]
    } else {
        body_bytes
    };

    // Handle different content types
    if content_type_str.contains("application/json") {
        // Try to parse and pretty-print JSON
        if let Ok(json_str) = std::str::from_utf8(limited_bytes) {
            if let Ok(json_value) = serde_json::from_str::<Value>(json_str) {
                return Some(serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| json_str.to_string()));
            }
            return Some(json_str.to_string());
        }
    } else if content_type_str.starts_with("text/") || content_type_str.contains("application/x-www-form-urlencoded") {
        // Handle text content
        if let Ok(text) = std::str::from_utf8(limited_bytes) {
            return Some(text.to_string());
        }
    } else if content_type_str.contains("multipart/form-data") {
        // For multipart data, just show summary
        return Some(format!("[MULTIPART_DATA: {} bytes]", body_bytes.len()));
    }

    // For binary content, show size and type
    Some(format!("[BINARY_DATA: {} bytes, type: {}]", body_bytes.len(), content_type_str))
}

/// Log comprehensive HTTP request activity with all captured data
async fn log_comprehensive_http_activity(
    correlation_context: Option<CorrelationContext>,
    matched_path: String,
    body_preview: Option<String>,
    status_code: StatusCode,
    duration_ms: u64,
) -> Result<(), anyhow::Error> {
    let service = ActivityLogService::new();

    let (correlation_id, request_data) = if let Some(ctx) = correlation_context {
        (ctx.correlation_id, ctx.request_data)
    } else {
        (DieselUlid::new(), None)
    };

    let description = if let Some(ref data) = request_data {
        format!(
            "{} {} - {} ({}ms)",
            data.method,
            data.path,
            status_code.as_u16(),
            duration_ms
        )
    } else {
        format!("HTTP Request - {} ({}ms)", status_code.as_u16(), duration_ms)
    };

    // Enhanced properties with all available context data
    let mut properties = json!({
        "correlation_id": correlation_id.to_string(),
        "http": {
            "status_code": status_code.as_u16(),
            "status_class": {
                "success": status_code.is_success(),
                "client_error": status_code.is_client_error(),
                "server_error": status_code.is_server_error(),
                "informational": status_code.is_informational(),
                "redirection": status_code.is_redirection()
            },
            "matched_path": matched_path
        },
        "performance": {
            "duration_ms": duration_ms,
            "duration_category": classify_duration(duration_ms)
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    // Add comprehensive request data if available
    let event_name = if let Some(ref data) = request_data {
        // Core request information
        properties["request"] = json!({
            "method": data.method,
            "uri": data.uri,
            "path": data.path,
            "query_string": data.query_string,
            "content_info": {
                "type": data.content_type,
                "length": data.content_length,
                "has_body": body_preview.is_some()
            }
        });

        // Client information
        properties["client"] = json!({
            "ip": data.remote_ip,
            "user_agent": data.user_agent,
        });

        // Headers (sanitized)
        properties["headers"] = json!(data.headers);

        // Body preview if available
        if let Some(body) = &body_preview {
            properties["request"]["body_preview"] = json!({
                "content": body,
                "truncated": body.len() >= 1024,
                "size_bytes": body.len()
            });
        }

        // Security context
        properties["security"] = json!({
            "has_auth_header": data.headers.contains_key("authorization"),
            "has_api_key": data.headers.contains_key("x-api-key") || data.headers.contains_key("x-auth-token"),
            "https": data.uri.starts_with("https://"),
            "sensitive_headers_redacted": true
        });

        format!("http.{}", data.method.to_lowercase())
    } else {
        "http.unknown".to_string()
    };

    // Add structured console logging
    log_to_console(&correlation_id, &request_data, status_code, duration_ms, &body_preview);

    // Add enhanced file logging to storage/logs
    log_to_file(&correlation_id, &request_data, status_code, duration_ms, &body_preview, &properties);

    // Store in database
    let now = chrono::Utc::now();
    let new_activity = ActivityLog {
        id: DieselUlid::new(),
        log_name: Some("http_request".to_string()),
        description,
        subject_type: None,
        subject_id: None,
        causer_type: None,
        causer_id: None,
        properties: Some(properties),
        correlation_id: Some(correlation_id),
        batch_uuid: None,
        event: Some(event_name),
        created_at: now,
        updated_at: now,
    };

    service.create(new_activity).await?;
    Ok(())
}

/// Classify request duration for analysis
fn classify_duration(duration_ms: u64) -> &'static str {
    match duration_ms {
        0..=50 => "very_fast",
        51..=200 => "fast",
        201..=500 => "normal",
        501..=1000 => "slow",
        1001..=2000 => "very_slow",
        _ => "extremely_slow"
    }
}

/// Enhanced console logging with structured data
fn log_to_console(
    correlation_id: &DieselUlid,
    request_data: &Option<crate::app::http::middleware::correlation_middleware::RequestData>,
    status_code: StatusCode,
    duration_ms: u64,
    body_preview: &Option<String>
) {
    if let Some(data) = request_data {
        let status_emoji = match status_code.as_u16() {
            200..=299 => "✅",
            300..=399 => "🔄",
            400..=499 => "⚠️",
            500..=599 => "❌",
            _ => "ℹ️"
        };

        let duration_emoji = match duration_ms {
            0..=50 => "⚡",
            51..=200 => "🚀",
            201..=500 => "⏱️",
            501..=1000 => "🐌",
            _ => "🚨"
        };

        // Main request log line
        tracing::info!(
            correlation_id = %correlation_id,
            method = %data.method,
            path = %data.path,
            status = status_code.as_u16(),
            duration_ms = duration_ms,
            remote_ip = %data.remote_ip.as_deref().unwrap_or("unknown"),
            user_agent = %data.user_agent.as_deref().unwrap_or("unknown"),
            "{} {} {} {} - {} {}ms from {}",
            status_emoji,
            data.method,
            data.path,
            status_code.as_u16(),
            duration_emoji,
            duration_ms,
            data.remote_ip.as_deref().unwrap_or("unknown")
        );

        // Additional debug info
        if !data.headers.is_empty() {
            tracing::debug!(
                correlation_id = %correlation_id,
                headers = ?data.headers,
                "Request headers (sensitive data redacted)"
            );
        }

        if let Some(query) = &data.query_string {
            tracing::debug!(
                correlation_id = %correlation_id,
                query_string = %query,
                "Request query parameters"
            );
        }

        if let Some(body) = body_preview {
            tracing::debug!(
                correlation_id = %correlation_id,
                body_size = body.len(),
                content_type = %data.content_type.as_deref().unwrap_or("unknown"),
                body_preview = %body,
                "Request body preview (truncated to 1KB)"
            );
        }

        // Performance warning for slow requests
        if duration_ms > 1000 {
            tracing::warn!(
                correlation_id = %correlation_id,
                duration_ms = duration_ms,
                method = %data.method,
                path = %data.path,
                "🚨 Slow request detected - consider optimization"
            );
        }

        // Security events
        if data.headers.contains_key("authorization") && status_code == 401 {
            tracing::warn!(
                correlation_id = %correlation_id,
                method = %data.method,
                path = %data.path,
                remote_ip = %data.remote_ip.as_deref().unwrap_or("unknown"),
                "🔒 Authentication failed"
            );
        }

    } else {
        // Fallback logging when no request data is available
        tracing::info!(
            correlation_id = %correlation_id,
            status = status_code.as_u16(),
            duration_ms = duration_ms,
            "HTTP Request - {} ({}ms)",
            status_code.as_u16(),
            duration_ms
        );
    }
}

/// Enhanced file logging with comprehensive context data
fn log_to_file(
    correlation_id: &DieselUlid,
    request_data: &Option<crate::app::http::middleware::correlation_middleware::RequestData>,
    status_code: StatusCode,
    duration_ms: u64,
    body_preview: &Option<String>,
    properties: &Value
) {
    use std::collections::HashMap;

    // Create comprehensive context data for file logging
    let mut file_context = HashMap::new();

    // Core tracking data
    file_context.insert("correlation_id".to_string(), json!(correlation_id.to_string()));
    file_context.insert("timestamp".to_string(), json!(chrono::Utc::now().to_rfc3339()));
    file_context.insert("component".to_string(), json!("activity_logging_middleware"));

    // HTTP response data
    file_context.insert("http_status_code".to_string(), json!(status_code.as_u16()));
    file_context.insert("http_status_class".to_string(), json!({
        "success": status_code.is_success(),
        "client_error": status_code.is_client_error(),
        "server_error": status_code.is_server_error(),
        "informational": status_code.is_informational(),
        "redirection": status_code.is_redirection()
    }));

    // Performance data
    file_context.insert("performance_duration_ms".to_string(), json!(duration_ms));
    file_context.insert("performance_category".to_string(), json!(classify_duration(duration_ms)));

    if let Some(data) = request_data {
        // HTTP request data
        file_context.insert("http_method".to_string(), json!(data.method));
        file_context.insert("http_uri".to_string(), json!(data.uri));
        file_context.insert("http_path".to_string(), json!(data.path));
        file_context.insert("http_query_string".to_string(), json!(data.query_string));

        // Content information
        file_context.insert("content_type".to_string(), json!(data.content_type));
        file_context.insert("content_length".to_string(), json!(data.content_length));
        file_context.insert("has_request_body".to_string(), json!(body_preview.is_some()));

        // Client information
        file_context.insert("client_ip".to_string(), json!(data.remote_ip));
        file_context.insert("user_agent".to_string(), json!(data.user_agent));

        // Security context
        file_context.insert("security_has_auth_header".to_string(), json!(data.headers.contains_key("authorization")));
        file_context.insert("security_has_api_key".to_string(), json!(
            data.headers.contains_key("x-api-key") || data.headers.contains_key("x-auth-token")
        ));
        file_context.insert("security_https".to_string(), json!(data.uri.starts_with("https://")));

        // Headers (sanitized)
        file_context.insert("request_headers".to_string(), json!(data.headers));

        // Body preview if available (with size information)
        if let Some(body) = body_preview {
            file_context.insert("request_body_preview".to_string(), json!({
                "content": body,
                "size_bytes": body.len(),
                "truncated": body.len() >= 1024
            }));
        }
    }

    // Add all the structured properties from the database logging
    file_context.insert("full_properties".to_string(), properties.clone());

    // Create the main log message
    let log_message = if let Some(data) = request_data {
        format!("HTTP {} {} - {} ({}ms) from {}",
            data.method,
            data.path,
            status_code.as_u16(),
            duration_ms,
            data.remote_ip.as_deref().unwrap_or("unknown")
        )
    } else {
        format!("HTTP Request - {} ({}ms)", status_code.as_u16(), duration_ms)
    };

    // Log to file with comprehensive context
    if status_code.is_server_error() {
        Log::error_with_context(&log_message, file_context);
    } else if status_code.is_client_error() {
        Log::warning_with_context(&log_message, file_context);
    } else if duration_ms > 1000 {
        Log::warning_with_context(&format!("SLOW REQUEST: {}", log_message), file_context);
    } else {
        Log::info_with_context(&log_message, file_context);
    }
}

// Note: Old client IP extraction function removed - now handled in correlation_middleware.rs
// This reduces duplication and ensures consistent IP extraction across the application

/// Helper struct for operation-specific activity logging
pub struct ActivityLogger {
    pub log_name: String,
    pub correlation_id: Option<DieselUlid>,
    pub causer_type: Option<String>,
    pub causer_id: Option<String>,
}

impl ActivityLogger {
    pub fn new(log_name: &str) -> Self {
        Self {
            log_name: log_name.to_string(),
            correlation_id: None,
            causer_type: None,
            causer_id: None,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: DieselUlid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_causer(mut self, causer_type: &str, causer_id: &str) -> Self {
        self.causer_type = Some(causer_type.to_string());
        self.causer_id = Some(causer_id.to_string());
        self
    }

    /// Log a create operation
    pub async fn log_create(&self, subject_type: &str, subject_id: &str, properties: Option<Value>) -> Result<(), anyhow::Error> {
        self.log_operation("create", subject_type, subject_id, properties).await
    }

    /// Log an update operation
    pub async fn log_update(&self, subject_type: &str, subject_id: &str, properties: Option<Value>) -> Result<(), anyhow::Error> {
        self.log_operation("update", subject_type, subject_id, properties).await
    }

    /// Log a delete operation
    pub async fn log_delete(&self, subject_type: &str, subject_id: &str, properties: Option<Value>) -> Result<(), anyhow::Error> {
        self.log_operation("delete", subject_type, subject_id, properties).await
    }

    /// Log a view operation
    pub async fn log_view(&self, subject_type: &str, subject_id: &str, properties: Option<Value>) -> Result<(), anyhow::Error> {
        self.log_operation("view", subject_type, subject_id, properties).await
    }

    /// Log a login operation
    pub async fn log_login(&self, user_id: &str, properties: Option<Value>) -> Result<(), anyhow::Error> {
        let mut props = properties.unwrap_or_else(|| json!({}));
        props["event_type"] = Value::String("authentication".to_string());

        self.log_operation("login", "User", user_id, Some(props)).await
    }

    /// Log a logout operation
    pub async fn log_logout(&self, user_id: &str, properties: Option<Value>) -> Result<(), anyhow::Error> {
        let mut props = properties.unwrap_or_else(|| json!({}));
        props["event_type"] = Value::String("authentication".to_string());

        self.log_operation("logout", "User", user_id, Some(props)).await
    }

    /// Log a failed login attempt
    pub async fn log_failed_login(&self, email: &str, reason: &str, properties: Option<Value>) -> Result<(), anyhow::Error> {
        let mut props = properties.unwrap_or_else(|| json!({}));
        props["event_type"] = Value::String("authentication_failed".to_string());
        props["email"] = Value::String(email.to_string());
        props["failure_reason"] = Value::String(reason.to_string());

        let service = ActivityLogService::new();
        let now = chrono::Utc::now();
        let new_activity = ActivityLog {
            id: DieselUlid::new(),
            log_name: Some(self.log_name.clone()),
            description: format!("Failed login attempt for {}: {}", email, reason),
            subject_type: None,
            subject_id: None,
            causer_type: self.causer_type.clone(),
            causer_id: self.causer_id.clone(),
            properties: Some(props),
            correlation_id: self.correlation_id,
            batch_uuid: None,
            event: Some("auth.login_failed".to_string()),
            created_at: now,
            updated_at: now,
        };

        service.create(new_activity).await?;
        Ok(())
    }

    /// Generic operation logger
    async fn log_operation(&self, operation: &str, subject_type: &str, subject_id: &str, properties: Option<Value>) -> Result<(), anyhow::Error> {
        let service = ActivityLogService::new();

        let description = format!(
            "{} {} with ID: {}",
            operation.to_uppercase(),
            subject_type,
            subject_id
        );

        let now = chrono::Utc::now();
        let new_activity = ActivityLog {
            id: DieselUlid::new(),
            log_name: Some(self.log_name.clone()),
            description,
            subject_type: Some(subject_type.to_string()),
            subject_id: Some(subject_id.to_string()),
            causer_type: self.causer_type.clone(),
            causer_id: self.causer_id.clone(),
            properties,
            correlation_id: self.correlation_id,
            batch_uuid: None,
            event: Some(format!("{}.{}", subject_type.to_lowercase(), operation)),
            created_at: now,
            updated_at: now,
        };

        service.create(new_activity).await?;
        Ok(())
    }

    /// Log a custom activity
    pub async fn log_custom(&self, description: &str, event: Option<&str>, properties: Option<Value>) -> Result<(), anyhow::Error> {
        let service = ActivityLogService::new();

        let now = chrono::Utc::now();
        let new_activity = ActivityLog {
            id: DieselUlid::new(),
            log_name: Some(self.log_name.clone()),
            description: description.to_string(),
            subject_type: None,
            subject_id: None,
            causer_type: self.causer_type.clone(),
            causer_id: self.causer_id.clone(),
            properties,
            correlation_id: self.correlation_id,
            batch_uuid: None,
            event: event.map(|e| e.to_string()),
            created_at: now,
            updated_at: now,
        };

        service.create(new_activity).await?;
        Ok(())
    }
}

/// Helper function to create an ActivityLogger from request context
pub fn activity_logger_from_request(request: &Request, log_name: &str) -> ActivityLogger {
    let correlation_id = request.extensions()
        .get::<CorrelationContext>()
        .map(|ctx| ctx.correlation_id);

    let mut logger = ActivityLogger::new(log_name);

    if let Some(correlation_id) = correlation_id {
        logger = logger.with_correlation_id(correlation_id);
    }

    logger
}

/// Helper function to create an ActivityLogger from Extensions
pub fn activity_logger_from_extensions(extensions: &axum::http::Extensions, log_name: &str) -> ActivityLogger {
    let correlation_id = extensions
        .get::<CorrelationContext>()
        .map(|ctx| ctx.correlation_id);

    let mut logger = ActivityLogger::new(log_name);

    if let Some(correlation_id) = correlation_id {
        logger = logger.with_correlation_id(correlation_id);
    }

    logger
}

/// Macro for easy activity logging in controllers
#[macro_export]
macro_rules! log_activity {
    ($request:expr, $log_name:expr, $operation:expr, $subject_type:expr, $subject_id:expr) => {
        {
            let logger = $crate::app::http::middleware::activity_logging_middleware::activity_logger_from_request(&$request, $log_name);
            match $operation {
                "create" => logger.log_create($subject_type, $subject_id, None).await,
                "update" => logger.log_update($subject_type, $subject_id, None).await,
                "delete" => logger.log_delete($subject_type, $subject_id, None).await,
                "view" => logger.log_view($subject_type, $subject_id, None).await,
                _ => logger.log_custom(&format!("{} {} with ID: {}", $operation, $subject_type, $subject_id), None, None).await,
            }
        }
    };

    ($request:expr, $log_name:expr, $operation:expr, $subject_type:expr, $subject_id:expr, $properties:expr) => {
        {
            let logger = $crate::app::http::middleware::activity_logging_middleware::activity_logger_from_request(&$request, $log_name);
            match $operation {
                "create" => logger.log_create($subject_type, $subject_id, Some($properties)).await,
                "update" => logger.log_update($subject_type, $subject_id, Some($properties)).await,
                "delete" => logger.log_delete($subject_type, $subject_id, Some($properties)).await,
                "view" => logger.log_view($subject_type, $subject_id, Some($properties)).await,
                _ => logger.log_custom(&format!("{} {} with ID: {}", $operation, $subject_type, $subject_id), None, Some($properties)).await,
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};


    #[tokio::test]
    async fn test_activity_logger_create() {
        let logger = ActivityLogger::new("test");

        // This would normally create an actual log entry
        // For testing, we just verify the struct is created correctly
        assert_eq!(logger.log_name, "test");
        assert!(logger.correlation_id.is_none());
        assert!(logger.causer_type.is_none());
        assert!(logger.causer_id.is_none());
    }

    #[test]
    fn test_activity_logger_with_correlation_id() {
        let correlation_id = DieselUlid::new();
        let logger = ActivityLogger::new("test")
            .with_correlation_id(correlation_id);

        assert_eq!(logger.correlation_id, Some(correlation_id));
    }

    #[test]
    fn test_activity_logger_with_causer() {
        let logger = ActivityLogger::new("test")
            .with_causer("User", "123");

        assert_eq!(logger.causer_type, Some("User".to_string()));
        assert_eq!(logger.causer_id, Some("123".to_string()));
    }
}