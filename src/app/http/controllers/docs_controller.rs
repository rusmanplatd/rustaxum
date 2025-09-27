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
        "description": "Access points for API documentation in various formats",
        "query_builder": {
            "description": "Advanced Laravel-style query builder with comprehensive filtering, sorting, pagination, and relationship inclusion",
            "documentation": "/docs/API_USAGE_GUIDE.md",
            "features": [
                "Complex filtering with 15+ operators",
                "Multi-column sorting with flexible syntax",
                "High-performance cursor and offset-based pagination",
                "Nested relationship inclusion with dot notation",
                "Field selection for optimized responses",
                "JSON field querying support",
                "Security validation and SQL injection protection",
                "Enterprise-grade performance optimization"
            ],
            "real_world_examples": {
                "geographic_analysis": "filter[continent][eq]=North America&filter[population][gte]=10000000&sort=population:desc&include=provinces.cities",
                "user_management": "filter[status][eq]=active&filter[email_verified_at][is_not_null]=true&include=organizations.positions,roles.permissions",
                "organizational_hierarchy": "filter[type][in]=department,division&filter[level][between]=1,3&include=parent,children,users.roles",
                "geospatial_search": "filter[latitude][between]=40,50&filter[longitude][between]=-80,-70&filter[population][gte]=100000&include=province.country",
                "performance_optimized": "fields[users]=id,name,email&pagination_type=cursor&per_page=100&sort=-created_at"
            },
            "operators": [
                "eq", "ne", "gt", "gte", "lt", "lte",
                "like", "ilike", "contains", "starts_with", "ends_with",
                "in", "not_in", "is_null", "is_not_null", "between"
            ],
            "performance_tips": [
                "Use cursor pagination for datasets > 10,000 records",
                "Select only required fields to reduce bandwidth by 70-80%",
                "Limit relationship depth to 3 levels for optimal performance",
                "Filter early to reduce dataset size before sorting/pagination"
            ]
        }
    });

    (StatusCode::OK, ResponseJson(info))
}