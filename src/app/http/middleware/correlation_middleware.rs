use axum::{
    extract::{Request, ConnectInfo},
    http::{HeaderMap, HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use uuid::Uuid;

use crate::app::models::DieselUlid;

pub const CORRELATION_ID_HEADER: &str = "X-Correlation-ID";
pub const CORRELATION_ID_HEADER_LOWERCASE: &str = "x-correlation-id";

#[derive(Clone, Debug)]
pub struct CorrelationContext {
    pub correlation_id: DieselUlid,
    pub request_data: Option<RequestData>,
}

#[derive(Clone, Debug)]
pub struct RequestData {
    pub method: String,
    pub uri: String,
    pub path: String,
    pub query_string: Option<String>,
    pub headers: HashMap<String, String>,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub user_agent: Option<String>,
    pub remote_ip: Option<String>,
}

impl CorrelationContext {
    pub fn new() -> Self {
        Self {
            correlation_id: DieselUlid::new(),
            request_data: None,
        }
    }

    pub fn with_id(correlation_id: DieselUlid) -> Self {
        Self {
            correlation_id,
            request_data: None,
        }
    }

    pub fn with_id_and_request_data(correlation_id: DieselUlid, request_data: RequestData) -> Self {
        Self {
            correlation_id,
            request_data: Some(request_data),
        }
    }

    pub fn id(&self) -> DieselUlid {
        self.correlation_id
    }

    pub fn id_string(&self) -> String {
        self.correlation_id.to_string()
    }
}

impl Default for CorrelationContext {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn correlation_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    let correlation_id = extract_or_generate_correlation_id(&request);

    // Extract socket address from extensions (if available)
    let socket_addr = request.extensions().get::<ConnectInfo<SocketAddr>>()
        .map(|connect_info| connect_info.0);

    // Extract comprehensive request data
    let request_data = extract_request_data(&request, socket_addr);

    // Add correlation context with request data to request extensions
    request.extensions_mut().insert(CorrelationContext::with_id_and_request_data(
        correlation_id,
        request_data,
    ));

    // Call the next handler
    let mut response = next.run(request).await;

    // Add correlation ID to response headers
    if let Ok(header_value) = HeaderValue::from_str(&correlation_id.to_string()) {
        response.headers_mut().insert(
            HeaderName::from_static(CORRELATION_ID_HEADER_LOWERCASE),
            header_value,
        );
    }

    response
}

fn extract_or_generate_correlation_id(request: &Request) -> DieselUlid {
    // Try to extract correlation ID from headers
    if let Some(correlation_header) = request.headers().get(CORRELATION_ID_HEADER) {
        if let Ok(correlation_str) = correlation_header.to_str() {
            // Try to parse as ULID first
            if let Ok(ulid) = correlation_str.parse::<ulid::Ulid>() {
                return DieselUlid::from(ulid);
            }

            // Try to parse as UUID and convert to ULID
            if let Ok(_uuid) = Uuid::from_str(correlation_str) {
                // For UUID, we'll generate a new ULID to maintain consistency
                return DieselUlid::new();
            }
        }
    }

    // Try the lowercase version
    if let Some(correlation_header) = request.headers().get(CORRELATION_ID_HEADER_LOWERCASE) {
        if let Ok(correlation_str) = correlation_header.to_str() {
            if let Ok(ulid) = correlation_str.parse::<ulid::Ulid>() {
                return DieselUlid::from(ulid);
            }
        }
    }

    // Generate new correlation ID if none found or invalid
    DieselUlid::new()
}

/// Extract comprehensive request data for logging
fn extract_request_data(request: &Request, socket_addr: Option<SocketAddr>) -> RequestData {
    let method = request.method().to_string();
    let uri = request.uri().to_string();
    let path = request.uri().path().to_string();
    let query_string = request.uri().query().map(|q| q.to_string());

    // Extract headers (sanitize sensitive ones)
    let headers = extract_safe_headers(request.headers());

    // Extract content type and length
    let content_type = request.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let content_length = request.headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    // Extract user agent
    let user_agent = request.headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Extract client IP (check headers first, then fallback to socket address)
    let remote_ip = extract_client_ip_from_headers(request.headers())
        .or_else(|| socket_addr.map(|addr| addr.ip().to_string()));

    RequestData {
        method,
        uri,
        path,
        query_string,
        headers,
        content_type,
        content_length,
        user_agent,
        remote_ip,
    }
}

/// Extract headers while sanitizing sensitive information
fn extract_safe_headers(headers: &HeaderMap) -> HashMap<String, String> {
    let sensitive_headers = [
        "authorization", "cookie", "set-cookie", "x-api-key",
        "x-auth-token", "x-csrf-token", "x-access-token",
        "proxy-authorization", "www-authenticate",
    ];

    headers
        .iter()
        .filter_map(|(name, value)| {
            let name_str = name.as_str().to_lowercase();
            if sensitive_headers.contains(&name_str.as_str()) {
                // Redact sensitive headers
                Some((name_str, "[REDACTED]".to_string()))
            } else {
                value.to_str().ok().map(|v| (name_str, v.to_string()))
            }
        })
        .collect()
}

/// Extract client IP from headers
fn extract_client_ip_from_headers(headers: &HeaderMap) -> Option<String> {
    let headers_to_check = [
        "cf-connecting-ip",      // Cloudflare
        "x-real-ip",            // Nginx
        "x-forwarded-for",      // Standard proxy header
        "x-client-ip",          // Alternative
        "x-cluster-client-ip",  // Cluster environments
    ];

    for header_name in &headers_to_check {
        if let Some(header_value) = headers.get(*header_name) {
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

// Extension trait to easily get correlation context from request
pub trait CorrelationExt {
    fn correlation_id(&self) -> Option<DieselUlid>;
    fn correlation_context(&self) -> Option<&CorrelationContext>;
    fn request_data(&self) -> Option<&RequestData>;
}

impl CorrelationExt for Request {
    fn correlation_id(&self) -> Option<DieselUlid> {
        self.extensions()
            .get::<CorrelationContext>()
            .map(|ctx| ctx.correlation_id)
    }

    fn correlation_context(&self) -> Option<&CorrelationContext> {
        self.extensions().get::<CorrelationContext>()
    }

    fn request_data(&self) -> Option<&RequestData> {
        self.extensions()
            .get::<CorrelationContext>()
            .and_then(|ctx| ctx.request_data.as_ref())
    }
}

// Helper functions for extracting correlation ID from various contexts
pub mod extractors {
    use super::*;

    /// Simple function to extract correlation ID from request extensions
    pub fn get_correlation_id_from_extensions(extensions: &axum::http::Extensions) -> Option<DieselUlid> {
        extensions
            .get::<CorrelationContext>()
            .map(|ctx| ctx.correlation_id)
    }

    /// Extract correlation ID from request extensions, generating a new one if not found
    pub fn get_or_generate_correlation_id(extensions: &axum::http::Extensions) -> DieselUlid {
        get_correlation_id_from_extensions(extensions)
            .unwrap_or_else(DieselUlid::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, Method};

    #[test]
    fn test_extract_correlation_id_from_header() {
        use axum::body::Body;
        let ulid = DieselUlid::new();
        let request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .header(CORRELATION_ID_HEADER, ulid.to_string())
            .body(Body::empty())
            .unwrap();

        let extracted_id = extract_or_generate_correlation_id(&request);
        assert_eq!(extracted_id, ulid);
    }

    #[test]
    fn test_generate_correlation_id_when_missing() {
        use axum::body::Body;
        let request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let correlation_id = extract_or_generate_correlation_id(&request);
        assert!(!correlation_id.to_string().is_empty());
    }

    #[test]
    fn test_generate_correlation_id_when_invalid() {
        use axum::body::Body;
        let request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .header(CORRELATION_ID_HEADER, "invalid-correlation-id")
            .body(Body::empty())
            .unwrap();

        let correlation_id = extract_or_generate_correlation_id(&request);
        assert!(!correlation_id.to_string().is_empty());
    }
}