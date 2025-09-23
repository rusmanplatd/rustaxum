use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::str::FromStr;
use uuid::Uuid;

use crate::app::models::DieselUlid;

pub const CORRELATION_ID_HEADER: &str = "X-Correlation-ID";
pub const CORRELATION_ID_HEADER_LOWERCASE: &str = "x-correlation-id";

#[derive(Clone, Debug)]
pub struct CorrelationContext {
    pub correlation_id: DieselUlid,
}

impl CorrelationContext {
    pub fn new() -> Self {
        Self {
            correlation_id: DieselUlid::new(),
        }
    }

    pub fn with_id(correlation_id: DieselUlid) -> Self {
        Self { correlation_id }
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

    // Add correlation context to request extensions
    request.extensions_mut().insert(CorrelationContext::with_id(correlation_id));

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

// Extension trait to easily get correlation context from request
pub trait CorrelationExt {
    fn correlation_id(&self) -> Option<DieselUlid>;
    fn correlation_context(&self) -> Option<&CorrelationContext>;
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