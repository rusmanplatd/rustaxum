use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
    middleware::Next,
    response::Response,
};
use bytes::Bytes;
use std::collections::HashMap;

use crate::app::services::csrf_service::CSRFService;
use crate::app::services::session::SessionStore;

#[derive(Clone)]
pub struct CSRFMiddleware {
    csrf_service: CSRFService,
    except: Vec<String>, // Routes to exempt from CSRF protection
}

impl CSRFMiddleware {
    pub fn new(csrf_service: CSRFService) -> Self {
        Self {
            csrf_service,
            except: Vec::new(),
        }
    }

    pub fn except(mut self, routes: Vec<String>) -> Self {
        self.except = routes;
        self
    }

    fn is_exempt_route(&self, path: &str) -> bool {
        self.except.iter().any(|route| {
            if route.ends_with('*') {
                let prefix = &route[..route.len() - 1];
                path.starts_with(prefix)
            } else {
                path == route
            }
        })
    }

    fn should_verify(&self, request: &Request) -> bool {
        // Skip CSRF for safe methods
        if matches!(request.method(), &Method::GET | &Method::HEAD | &Method::OPTIONS) {
            return false;
        }

        // Check if route is exempted
        let path = request.uri().path();
        if self.is_exempt_route(path) {
            return false;
        }

        true
    }

    async fn extract_form_token(&self, body: Bytes) -> Option<String> {
        if let Ok(body_str) = std::str::from_utf8(&body) {
            // Check if it's form data
            if body_str.contains(&format!("{}=", self.csrf_service.token_name())) {
                return self.csrf_service.extract_token_from_form(body_str);
            }

            // Check if it's JSON and contains token
            if let Ok(json) = serde_json::from_str::<HashMap<String, serde_json::Value>>(body_str) {
                if let Some(token_value) = json.get(self.csrf_service.token_name()) {
                    if let Some(token_str) = token_value.as_str() {
                        return Some(token_str.to_string());
                    }
                }
            }
        }
        None
    }

    fn csrf_error_response(&self) -> Response {
        let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>419 - Page Expired</title>
            <style>
                body { font-family: Arial, sans-serif; text-align: center; margin-top: 50px; }
                .error-container { max-width: 500px; margin: 0 auto; }
                h1 { color: #e74c3c; }
                p { color: #666; margin-bottom: 20px; }
                .back-button {
                    background: #3498db;
                    color: white;
                    padding: 10px 20px;
                    text-decoration: none;
                    border-radius: 5px;
                    display: inline-block;
                }
                .back-button:hover { background: #2980b9; }
            </style>
        </head>
        <body>
            <div class="error-container">
                <h1>419 - Page Expired</h1>
                <p>Sorry, your session has expired. Please refresh the page and try again.</p>
                <a href="javascript:history.back()" class="back-button">Go Back</a>
            </div>
        </body>
        </html>
        "#;

        Response::builder()
            .status(StatusCode::from_u16(419).unwrap())
            .header("Content-Type", "text/html")
            .body(Body::from(html))
            .unwrap()
    }
}

pub async fn csrf_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get session store from request extensions
    let session_store = request
        .extensions()
        .get::<SessionStore>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .clone();

    let csrf_service = CSRFService::new();
    let csrf_middleware = CSRFMiddleware::new(csrf_service.clone());

    // Check if we should verify this request
    if !csrf_middleware.should_verify(&request) {
        return Ok(next.run(request).await);
    }

    // Try to extract token from headers first
    let token_from_headers = csrf_service.extract_token_from_headers(request.headers());

    let (parts, body) = request.into_parts();

    // If no token in headers, try to extract from body
    let token = if let Some(header_token) = token_from_headers {
        Some(header_token)
    } else {
        // Collect body to check for token
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let body_token = csrf_middleware.extract_form_token(body_bytes.clone()).await;

        // Reconstruct request with the body
        let new_body = Body::from(body_bytes);
        let mut request = Request::from_parts(parts, new_body);

        // Add session store back to extensions
        request.extensions_mut().insert(session_store.clone());

        if let Some(token) = body_token {
            // Verify token
            match csrf_service.verify_token(&token, &session_store).await {
                Ok(true) => return Ok(next.run(request).await),
                Ok(false) => return Ok(csrf_middleware.csrf_error_response()),
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        } else {
            return Ok(csrf_middleware.csrf_error_response());
        }
    };

    // Reconstruct request if we only had header token
    let mut request = Request::from_parts(parts, Body::empty());
    request.extensions_mut().insert(session_store.clone());

    // Verify the token
    if let Some(token) = token {
        match csrf_service.verify_token(&token, &session_store).await {
            Ok(true) => Ok(next.run(request).await),
            Ok(false) => Ok(csrf_middleware.csrf_error_response()),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Ok(csrf_middleware.csrf_error_response())
    }
}

// CSRF middleware is applied globally in lib.rs

// Exempted middleware (for routes that should skip CSRF)
pub async fn csrf_exempt_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exempt_route_matching() {
        let csrf_service = CSRFService::new();
        let middleware = CSRFMiddleware::new(csrf_service)
            .except(vec![
                "/api/public/*".to_string(),
                "/webhook/exact".to_string(),
            ]);

        assert!(middleware.is_exempt_route("/api/public/test"));
        assert!(middleware.is_exempt_route("/api/public/users/123"));
        assert!(middleware.is_exempt_route("/webhook/exact"));
        assert!(!middleware.is_exempt_route("/webhook/other"));
        assert!(!middleware.is_exempt_route("/api/private/test"));
    }
}