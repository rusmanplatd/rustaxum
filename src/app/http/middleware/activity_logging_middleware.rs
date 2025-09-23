use axum::{
    extract::{Request, MatchedPath},
    http::{Method, StatusCode},
    middleware::Next,
    response::Response,
};
use serde_json::{json, Value};
use std::time::Instant;

use crate::app::http::middleware::correlation_middleware::CorrelationContext;
use crate::app::models::activity_log::NewActivityLog;
use crate::app::services::activity_log_service::ActivityLogService;
use crate::app::models::DieselUlid;

/// Middleware for automatic activity logging of HTTP requests
pub async fn activity_logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let query = request.uri().query().map(|q| q.to_string());

    // Extract correlation context if available
    let correlation_id = request.extensions()
        .get::<CorrelationContext>()
        .map(|ctx| ctx.correlation_id)
        .unwrap_or_else(DieselUlid::new);

    // Extract matched path pattern if available
    let matched_path = request.extensions()
        .get::<MatchedPath>()
        .map(|mp| mp.as_str().to_string())
        .unwrap_or_else(|| path.clone());

    // Get remote IP if available
    let remote_ip = extract_client_ip(&request);

    // Call the next handler
    let response = next.run(request).await;

    // Calculate request duration
    let duration = start_time.elapsed();
    let duration_ms = duration.as_millis() as u64;

    // Extract status code before moving response
    let status_code = response.status();

    // Log the activity asynchronously (fire and forget)
    tokio::spawn(async move {
        let _ = log_http_request_activity(
            method,
            path,
            matched_path,
            query,
            status_code,
            duration_ms,
            correlation_id,
            remote_ip,
        ).await;
    });

    response
}

/// Log HTTP request activity
async fn log_http_request_activity(
    method: Method,
    path: String,
    matched_path: String,
    query: Option<String>,
    status_code: StatusCode,
    duration_ms: u64,
    correlation_id: DieselUlid,
    remote_ip: Option<String>,
) -> Result<(), anyhow::Error> {
    let service = ActivityLogService::new();

    let description = format!(
        "{} {} - {} ({}ms)",
        method.as_str(),
        path,
        status_code.as_u16(),
        duration_ms
    );

    let mut properties = json!({
        "method": method.as_str(),
        "path": path,
        "matched_path": matched_path,
        "status_code": status_code.as_u16(),
        "duration_ms": duration_ms,
        "success": status_code.is_success(),
        "client_error": status_code.is_client_error(),
        "server_error": status_code.is_server_error(),
    });

    // Add query parameters if present
    if let Some(query_string) = query {
        properties["query"] = Value::String(query_string);
    }

    // Add remote IP if available
    if let Some(ip) = remote_ip {
        properties["remote_ip"] = Value::String(ip);
    }

    let new_activity = NewActivityLog {
        log_name: Some("http_request".to_string()),
        description,
        subject_type: None,
        subject_id: None,
        causer_type: None,
        causer_id: None,
        properties: Some(properties),
        correlation_id: Some(correlation_id),
        batch_uuid: None,
        event: Some(format!("http.{}", method.as_str().to_lowercase())),
    };

    service.create(new_activity).await?;
    Ok(())
}

/// Extract client IP from request headers
fn extract_client_ip(request: &Request) -> Option<String> {
    // Try various headers in order of preference
    let headers_to_check = [
        "cf-connecting-ip",      // Cloudflare
        "x-real-ip",            // Nginx
        "x-forwarded-for",      // Standard proxy header
        "x-client-ip",          // Alternative
        "x-cluster-client-ip",  // Cluster environments
    ];

    for header_name in &headers_to_check {
        if let Some(header_value) = request.headers().get(*header_name) {
            if let Ok(ip_str) = header_value.to_str() {
                // For X-Forwarded-For, take the first IP (client)
                let ip = ip_str.split(',').next().unwrap_or(ip_str).trim();
                if !ip.is_empty() && ip != "unknown" {
                    return Some(ip.to_string());
                }
            }
        }
    }

    None
}

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
        let new_activity = NewActivityLog {
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

        let new_activity = NewActivityLog {
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
        };

        service.create(new_activity).await?;
        Ok(())
    }

    /// Log a custom activity
    pub async fn log_custom(&self, description: &str, event: Option<&str>, properties: Option<Value>) -> Result<(), anyhow::Error> {
        let service = ActivityLogService::new();

        let new_activity = NewActivityLog {
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

    #[test]
    fn test_extract_client_ip_from_cf_connecting_ip() {
        let request = Request::builder()
            .header("cf-connecting-ip", "192.168.1.1")
            .body(Body::empty())
            .unwrap();

        assert_eq!(extract_client_ip(&request), Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_extract_client_ip_from_x_forwarded_for() {
        let request = Request::builder()
            .header("x-forwarded-for", "192.168.1.1, 10.0.0.1, 172.16.0.1")
            .body(Body::empty())
            .unwrap();

        assert_eq!(extract_client_ip(&request), Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_extract_client_ip_none() {
        let request = Request::builder()
            .body(Body::empty())
            .unwrap();

        assert_eq!(extract_client_ip(&request), None);
    }

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