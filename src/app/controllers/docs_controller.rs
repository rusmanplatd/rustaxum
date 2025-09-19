use axum::{
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};

use crate::app::docs::ApiDoc;

/// Get OpenAPI specification in JSON format
///
/// Returns the complete OpenAPI 3.0 specification for this API in JSON format.
/// This can be used with any OpenAPI-compatible tools like Swagger UI, Postman, etc.
///
/// # Example
/// ```
/// GET /api/docs/openapi.json
/// ```
pub async fn openapi_json() -> impl IntoResponse {
    let spec = ApiDoc::openapi_json();
    (
        StatusCode::OK,
        [("Content-Type", "application/json")],
        spec
    )
}

/// Get OpenAPI specification in YAML format
///
/// Returns the complete OpenAPI 3.0 specification for this API in YAML format.
/// This format is often preferred for human-readable documentation files.
///
/// # Example
/// ```
/// GET /api/docs/openapi.yaml
/// ```
pub async fn openapi_yaml() -> impl IntoResponse {
    let spec = ApiDoc::openapi_yaml();
    (
        StatusCode::OK,
        [("Content-Type", "application/yaml")],
        spec
    )
}

/// Get API documentation information
///
/// Returns basic information about the API documentation endpoints available.
///
/// # Example
/// ```
/// GET /api/docs
/// ```
pub async fn docs_info() -> impl IntoResponse {
    let info = serde_json::json!({
        "message": "RustAxum API Documentation",
        "version": "1.0.0",
        "endpoints": {
            "openapi_json": "/api/docs/openapi.json",
            "openapi_yaml": "/api/docs/openapi.yaml",
            "swagger_ui": "/docs/swagger",
            "redoc": "/docs/redoc",
            "rapidoc": "/docs/rapidoc"
        },
        "description": "Access points for API documentation in various formats"
    });

    (StatusCode::OK, ResponseJson(info))
}