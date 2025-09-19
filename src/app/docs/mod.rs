use utoipa::OpenApi;

pub mod extractor;

// Import all the models and requests that will have schemas
use crate::app::models::country::{Country, CreateCountry, UpdateCountry, CountryResponse};
use crate::app::models::user::{User, CreateUser, UpdateUser, UserResponse, RefreshTokenRequest};
use crate::app::models::province::{Province, CreateProvince, UpdateProvince, ProvinceResponse};
use crate::app::models::city::{City, CreateCity, UpdateCity, CityResponse};
use crate::app::http::requests::country_requests::{CreateCountryRequest, UpdateCountryRequest};
use crate::app::http::requests::auth_requests::{RegisterRequest, LoginRequest, ForgotPasswordRequest, ResetPasswordRequest, ChangePasswordRequest};
use crate::app::http::requests::user_requests::{UpdateUserRequest, SearchUsersRequest, ContactFormRequest};
use crate::app::http::requests::province_requests::{CreateProvinceRequest, UpdateProvinceRequest};
use crate::app::http::requests::city_requests::{CreateCityRequest, UpdateCityRequest};

/// Main OpenAPI documentation structure
/// This generates the OpenAPI specification automatically from code annotations
#[derive(OpenApi)]
#[openapi(
    info(
        title = "RustAxum API",
        version = "1.0.0",
        description = "A Laravel-inspired Rust web framework built with Axum\n\nThis API follows REST conventions and provides comprehensive CRUD operations for all resources. All endpoints return JSON responses and follow consistent error handling patterns.",
        contact(
            name = "API Support",
            email = "support@rustaxum.dev"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Development server"),
        (url = "https://api.rustaxum.dev", description = "Production server")
    ),
    components(
        schemas(
            // Country models
            Country, CreateCountry, UpdateCountry, CountryResponse,
            CreateCountryRequest, UpdateCountryRequest,

            // User models
            User, CreateUser, UpdateUser, UserResponse, RefreshTokenRequest,
            UpdateUserRequest, SearchUsersRequest, ContactFormRequest,

            // Province models
            Province, CreateProvince, UpdateProvince, ProvinceResponse,
            CreateProvinceRequest, UpdateProvinceRequest,

            // City models
            City, CreateCity, UpdateCity, CityResponse,
            CreateCityRequest, UpdateCityRequest,

            // Authentication requests
            RegisterRequest, LoginRequest, ForgotPasswordRequest,
            ResetPasswordRequest, ChangePasswordRequest,

            // Common response types
            ErrorResponse,
            MessageResponse,
            ValidationErrorResponse,
            PaginatedResponse<CountryResponse>,
            PaginatedResponse<UserResponse>,
            PaginatedResponse<ProvinceResponse>,
            PaginatedResponse<CityResponse>,
        )
    ),
    tags(
        (name = "Authentication", description = "User authentication and authorization operations"),
        (name = "Users", description = "User management operations"),
        (name = "Countries", description = "Country management operations - full CRUD with filtering and pagination"),
        (name = "Provinces", description = "Province management operations - linked to countries"),
        (name = "Cities", description = "City management operations - linked to provinces"),
        (name = "Roles", description = "Role-based access control operations"),
        (name = "Permissions", description = "Permission management operations"),
    )
)]
pub struct ApiDoc;

// Common response schemas
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Standard error response returned by API endpoints
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    /// Error message describing what went wrong
    #[schema(example = "Invalid ID format")]
    pub error: String,
}

/// Standard success message response for operations that don't return data
#[derive(Serialize, Deserialize, ToSchema)]
pub struct MessageResponse {
    /// Success message
    #[schema(example = "Country deleted successfully")]
    pub message: String,
}

/// Validation error response with detailed field errors
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ValidationErrorResponse {
    /// Validation error message
    #[schema(example = "The given data was invalid.")]
    pub message: String,
    /// Field-specific validation errors
    #[schema(example = "Field validation errors map")]
    pub errors: std::collections::HashMap<String, Vec<String>>,
}

/// Generic paginated response wrapper for list endpoints
#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    /// Array of items for current page
    pub data: Vec<T>,
    /// Pagination metadata
    pub meta: PaginationMeta,
}

/// Pagination metadata included in paginated responses
#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginationMeta {
    /// Current page number (1-based)
    #[schema(example = 1)]
    pub current_page: u32,
    /// Total number of pages available
    #[schema(example = 5)]
    pub total_pages: u32,
    /// Total number of items across all pages
    #[schema(example = 50)]
    pub total_items: u64,
    /// Number of items per page
    #[schema(example = 10)]
    pub per_page: u32,
    /// Whether there is a next page available
    #[schema(example = true)]
    pub has_next: bool,
    /// Whether there is a previous page available
    #[schema(example = false)]
    pub has_prev: bool,
}

impl ApiDoc {
    /// Generate OpenAPI specification as JSON string
    pub fn openapi_json() -> String {
        Self::openapi().to_pretty_json().unwrap()
    }

    /// Generate OpenAPI specification as YAML string
    pub fn openapi_yaml() -> String {
        serde_yaml::to_string(&Self::openapi()).unwrap()
    }
}