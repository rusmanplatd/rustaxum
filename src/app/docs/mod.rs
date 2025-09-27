use utoipa::OpenApi;
use utoipa_auto_discovery::utoipa_auto_discovery;

pub mod oauth;

// Import only basic models to prevent circular dependencies
use crate::app::models::country::{Country, CreateCountry, UpdateCountry, CountryResponse};
use crate::app::models::user::{User, CreateUser, UpdateUser, UserResponse, RefreshTokenRequest};
use crate::app::models::province::{Province, CreateProvince, UpdateProvince, ProvinceResponse};
use crate::app::models::city::{City, CreateCity, UpdateCity, CityResponse};
use crate::app::http::requests::country_requests::{CreateCountryRequest, UpdateCountryRequest};
use crate::app::http::requests::auth_requests::{RegisterRequest, LoginRequest, ForgotPasswordRequest, ResetPasswordRequest, ChangePasswordRequest};
use crate::app::http::requests::user_requests::{UpdateUserRequest, SearchUsersRequest, ContactFormRequest};
use crate::app::http::requests::province_requests::{CreateProvinceRequest, UpdateProvinceRequest};
use crate::app::http::requests::city_requests::{CreateCityRequest, UpdateCityRequest};

// Adding back simple models that don't have circular dependencies
use crate::app::models::organization_position_level::{OrganizationPositionLevel, CreateOrganizationPositionLevel, UpdateOrganizationPositionLevel, OrganizationPositionLevelResponse};
use crate::app::models::organization_position::{OrganizationPosition, CreateOrganizationPosition, UpdateOrganizationPosition, OrganizationPositionResponse};
use crate::app::http::requests::organization_position_level_requests::{CreateOrganizationPositionLevelRequest, UpdateOrganizationPositionLevelRequest, IndexOrganizationPositionLevelRequest};
use crate::app::http::requests::organization_position_requests::{CreateOrganizationPositionRequest, UpdateOrganizationPositionRequest, IndexOrganizationPositionRequest, OrganizationPositionsByLevelRequest};

// Query Builder response structures
use crate::app::query_builder::response::{QueryResponse, QueryMeta, DataResponse, QueryErrorResponse, ResponseLinks, Link, CacheStatus};
use crate::app::query_builder::{Pagination, PaginationResult, PaginationType};
use crate::app::query_builder::pagination::{PaginationInfo, CursorData};

// Auth controller models
use crate::app::http::controllers::auth_controller::MfaLoginRequest;

// Role and permission models need ToSchema trait implementation - commented out for now
// use crate::app::models::role::{Role, CreateRole, UpdateRole, RoleResponse};
// use crate::app::models::permission::{Permission, CreatePermission, UpdatePermission, PermissionResponse};

// Adding organization model - should be safe as it has ToSchema implemented
use crate::app::models::organization::{Organization, CreateOrganization, UpdateOrganization, OrganizationResponse};

// More complex models with potential circular dependencies - kept commented for now
// use crate::app::models::user_organization::{UserOrganization, CreateUserOrganization, UpdateUserOrganization, UserOrganizationResponse};
// use crate::app::models::sys_model_has_permission::{SysModelHasPermission, CreateSysModelHasPermission, UpdateSysModelHasPermission, SysModelHasPermissionResponse};
// use crate::app::models::sys_model_has_role::{SysModelHasRole, CreateSysModelHasRole, UpdateSysModelHasRole, SysModelHasRoleResponse};

/// Main OpenAPI documentation structure with auto-discovery
/// This automatically discovers all endpoints with utoipa path annotations
#[utoipa_auto_discovery(
    paths = "(crate::app::http::controllers::auth_controller => ./src/app/http/controllers/auth_controller.rs);
             (crate::app::http::controllers::country_controller => ./src/app/http/controllers/country_controller.rs);
             (crate::app::http::controllers::user_controller => ./src/app/http/controllers/user_controller.rs);
             (crate::app::http::controllers::province_controller => ./src/app/http/controllers/province_controller.rs);
             (crate::app::http::controllers::city_controller => ./src/app/http/controllers/city_controller.rs);
             (crate::app::http::controllers::village_controller => ./src/app/http/controllers/village_controller.rs);
             (crate::app::http::controllers::district_controller => ./src/app/http/controllers/district_controller.rs);
             (crate::app::http::controllers::organization_controller => ./src/app/http/controllers/organization_controller.rs);
             (crate::app::http::controllers::organization_position_level_controller => ./src/app/http/controllers/organization_position_level_controller.rs);
             (crate::app::http::controllers::organization_position_controller => ./src/app/http/controllers/organization_position_controller.rs);
             (crate::app::http::controllers::role_controller => ./src/app/http/controllers/role_controller.rs);
             (crate::app::http::controllers::permission_controller => ./src/app/http/controllers/permission_controller.rs);
             (crate::app::http::controllers::user_organization_controller => ./src/app/http/controllers/user_organization_controller.rs);
             (crate::app::http::controllers::sys_model_has_permission_controller => ./src/app/http/controllers/sys_model_has_permission_controller.rs);
             (crate::app::http::controllers::sys_model_has_role_controller => ./src/app/http/controllers/sys_model_has_role_controller.rs);
             (crate::app::http::controllers::activity_log_controller => ./src/app/http/controllers/activity_log_controller.rs);
             (crate::app::http::controllers::oauth::oauth_controller => ./src/app/http/controllers/oauth/oauth_controller.rs);
             (crate::app::http::controllers::oauth::client_controller => ./src/app/http/controllers/oauth/client_controller.rs);
             (crate::app::http::controllers::oauth::personal_access_token_controller => ./src/app/http/controllers/oauth/personal_access_token_controller.rs);
             (crate::app::http::controllers::oauth::scope_controller => ./src/app/http/controllers/oauth/scope_controller.rs);
             (crate::app::http::controllers::oauth::authorization_controller => ./src/app/http/controllers/oauth/authorization_controller.rs);
             (crate::app::http::controllers::oauth::token_controller => ./src/app/http/controllers/oauth/token_controller.rs);
             (crate::app::http::controllers::oauth::admin_controller => ./src/app/http/controllers/oauth/admin_controller.rs);
             (crate::app::http::controllers::oauth::device_controller => ./src/app/http/controllers/oauth/device_controller.rs)"
)]
#[derive(OpenApi)]
#[openapi(
    info(
        title = "RustAxum API",
        version = "1.0.0",
        description = "A Laravel-inspired Rust web framework built with Axum\n\nThis API follows REST conventions and provides comprehensive CRUD operations for all resources. All endpoints return JSON responses and follow consistent error handling patterns.\n\n## 🚀 Auto-Discovery\n\nThis API documentation is automatically generated using utoipa_auto_discovery, which scans for all endpoints with `#[utoipa::path]` annotations and includes them in the OpenAPI specification.\n\n## 🔍 Advanced Query Builder\n\nThis API features a powerful Laravel-style query builder with support for:\n\n### Complex Filtering\n- **Comparison operators**: `eq`, `ne`, `gt`, `gte`, `lt`, `lte`\n- **Pattern matching**: `like`, `ilike`, `contains`, `starts_with`, `ends_with`\n- **List operations**: `in`, `not_in`\n- **Null checks**: `is_null`, `is_not_null`\n- **Range queries**: `between`\n- **JSON operations**: Query JSONB fields and nested data\n\n### Multi-Column Sorting\n- Sort by multiple fields with different directions\n- Support for both `-field` and `field:desc` syntax\n- Automatic validation against allowed sort fields\n\n### Relationship Inclusion\n- Eager load relationships using dot notation\n- Nested relationship support: `organization.positions.level`\n- Relationship-specific field selection and filtering\n\n### Flexible Pagination\n- **Cursor-based**: High performance for large datasets\n- **Offset-based**: Traditional page/per_page pagination\n- Automatic pagination type detection and conversion\n\n### Field Selection\n- Select only needed fields to optimize response size\n- Relationship-specific field selection\n- Automatic validation against allowed fields\n\n### Usage Examples\n```\nGET /api/users?\n  filter[name][contains]=john&\n  filter[status][in]=active,verified&\n  filter[created_at][gte]=2023-01-01&\n  sort=name,-created_at&\n  include=organization.positions&\n  fields[users]=id,name,email&\n  page=1&per_page=20\n```",
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
            // Basic models only to prevent circular dependencies
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

            // Basic auth requests
            RegisterRequest, LoginRequest, ForgotPasswordRequest, ResetPasswordRequest, ChangePasswordRequest,
            MfaLoginRequest,

            // Note: Complex schemas with potential circular dependencies are commented out
            // to prevent stack overflow during OpenAPI generation

            // Organization models - safe to include with ToSchema implemented
            Organization, CreateOrganization, UpdateOrganization, OrganizationResponse,

            // User Organization models - commented out due to complex relationships
            // UserOrganization, CreateUserOrganization, UpdateUserOrganization, UserOrganizationResponse,
            // CreateUserOrganizationRequest, UpdateUserOrganizationRequest, IndexUserOrganizationRequest,
            // TransferUserOrganizationRequest, AssignRoleRequest, RemoveRoleRequest,

            // Job models - safe to include as they have minimal dependencies
            OrganizationPositionLevel, CreateOrganizationPositionLevel, UpdateOrganizationPositionLevel, OrganizationPositionLevelResponse,
            CreateOrganizationPositionLevelRequest, UpdateOrganizationPositionLevelRequest, IndexOrganizationPositionLevelRequest,
            OrganizationPosition, CreateOrganizationPosition, UpdateOrganizationPosition, OrganizationPositionResponse,
            CreateOrganizationPositionRequest, UpdateOrganizationPositionRequest, IndexOrganizationPositionRequest, OrganizationPositionsByLevelRequest,

            // Role and Permission models - commented out until ToSchema is implemented
            // Role, CreateRole, UpdateRole, RoleResponse,
            // Permission, CreatePermission, UpdatePermission, PermissionResponse,

            // Polymorphic permission models - commented out due to potential circular references
            // SysModelHasPermission, CreateSysModelHasPermission, UpdateSysModelHasPermission, SysModelHasPermissionResponse,
            // Role and Permission models - commented out due to potential circular references
            // CreateSysModelHasPermissionRequest, UpdateSysModelHasPermissionRequest,
            // SysModelHasRole, CreateSysModelHasRole, UpdateSysModelHasRole, SysModelHasRoleResponse,
            // CreateSysModelHasRoleRequest, UpdateSysModelHasRoleRequest,

            // Complex resource models - commented out due to circular dependencies
            // UserOrganizationResource, UserOrganizationResourceWithRelations, UserOrganizationCollection,
            // UserOrganizationSummaryResource, UserOrganizationActivityResource, OrganizationHierarchyResource,
            // UserBasicInfo, OrganizationBasicInfo, OrganizationPositionBasicInfo, OrganizationPositionLevelBasicInfo, RoleBasicInfo,
            // UserOrgPaginationMeta, OrganizationTypeCount,

            // Common response types - basic ones only
            ErrorResponse,
            MessageResponse,
            ValidationErrorResponse,

            // Query Builder schemas
            QueryFilterSchema,
            QuerySortSchema,
            QueryIncludeSchema,
            QueryPaginationSchema,
            FilterOperatorSchema,

            // Query Builder response structures
            QueryResponse<serde_json::Value>,
            QueryMeta,
            DataResponse<serde_json::Value>,
            QueryErrorResponse,
            ResponseLinks,
            Link,
            CacheStatus,
            Pagination,
            PaginationResult<serde_json::Value>,
            PaginationType,
            PaginationInfo,
            CursorData
        )
    ),
    tags(
        (name = "Authentication", description = "User authentication and authorization operations"),
        (name = "Users", description = "User management operations"),
        (name = "Countries", description = "Country management operations - full CRUD with advanced filtering, sorting, pagination, and relationship inclusion\n\n## Query Parameters\n\n### Advanced Filtering (15+ operators)\n- `filter[field][operator]=value` - Apply filters using various operators\n- **Comparison**: `eq`, `ne`, `gt`, `gte`, `lt`, `lte`\n- **Text search**: `like`, `ilike`, `contains`, `starts_with`, `ends_with`\n- **List operations**: `in`, `not_in`\n- **Null checks**: `is_null`, `is_not_null`\n- **Range queries**: `between`\n\n**Examples:**\n```\n# Exact match\nGET /api/countries?filter[name][eq]=Canada\n\n# Text search (case-insensitive)\nGET /api/countries?filter[name][contains]=united\n\n# Multiple filters with different operators\nGET /api/countries?filter[name][starts_with]=A&filter[iso_code][in]=US,CA,GB&filter[created_at][gte]=2023-01-01\n\n# Range queries\nGET /api/countries?filter[population][between]=1000000,50000000\n```\n\n### Multi-Column Sorting\n- `sort=field1,-field2,field3:desc` - Flexible syntax support\n- Use `-` prefix or `:desc` suffix for descending order\n\n**Examples:**\n```\n# Single field\nGET /api/countries?sort=name\n\n# Multiple fields with mixed syntax\nGET /api/countries?sort=region:asc,-population,name\n\n# Complex sorting\nGET /api/countries?sort=continent:asc,region:asc,-population,name:asc\n```\n\n### High-Performance Pagination\n- **Cursor-based** (default): `cursor=...&per_page=20` - Best for large datasets\n- **Offset-based**: `page=1&per_page=15` - Traditional pagination\n- `pagination_type=cursor|offset` - Force pagination type\n\n**Examples:**\n```\n# Cursor pagination (recommended)\nGET /api/countries?per_page=20&cursor=eyJpZCI6MTAwfQ==\n\n# Offset pagination\nGET /api/countries?page=2&per_page=25&pagination_type=offset\n```\n\n### Field Selection & Performance\n- `fields[countries]=id,name,iso_code` - Select specific fields\n- Reduces bandwidth and improves response time\n- Supports relationship field selection\n\n**Examples:**\n```\n# Optimize for minimal payload\nGET /api/countries?fields[countries]=id,name,iso_code\n\n# Select fields for multiple resources\nGET /api/countries?fields[countries]=id,name&fields[provinces]=id,name&include=provinces\n```\n\n### Relationship Inclusion\n- `include=provinces,provinces.cities` - Eager load relationships\n- Supports nested relationships with dot notation\n- Prevents N+1 query problems\n\n**Examples:**\n```\n# Include direct relationships\nGET /api/countries?include=provinces\n\n# Include nested relationships\nGET /api/countries?include=provinces.cities,provinces.cities.districts\n\n# Combined with field selection\nGET /api/countries?include=provinces&fields[countries]=id,name&fields[provinces]=id,name,population\n```\n\n### Complete Examples\n```\n# Advanced filtering with relationships\nGET /api/countries?\n  filter[name][contains]=united&\n  filter[population][gte]=10000000&\n  filter[continent][eq]=North America&\n  sort=population:desc,name:asc&\n  include=provinces.cities&\n  fields[countries]=id,name,population,continent&\n  per_page=10\n\n# Search with cursor pagination\nGET /api/countries?\n  filter[name][starts_with]=A&\n  sort=-created_at&\n  cursor=eyJjcmVhdGVkX2F0IjoxNjc4ODg2NDAwfQ==&\n  per_page=20\n```"),
        (name = "Provinces", description = "Province management operations - linked to countries"),
        (name = "Cities", description = "City management operations - linked to provinces"),
        (name = "User Organizations", description = "User-Organization relationship management with hierarchical access control, RBAC/ABAC authorization, and transfer operations"),
        (name = "Organizations", description = "Hierarchical organization structure management (holding, subsidiary, divisions, departments, branches, etc.)"),
        (name = "Job Levels", description = "Job level hierarchy management for career progression"),
        (name = "Organization Positions", description = "Organization position management linked to organization position levels"),
        (name = "Roles", description = "Role-based access control operations"),
        (name = "Permissions", description = "Permission management operations"),
        (name = "Model Permissions", description = "Polymorphic model permission assignments - assign permissions to any model type"),
        (name = "Model Roles", description = "Polymorphic model role assignments - assign roles to any model type"),
        (name = "OAuth Core", description = "OAuth2 authentication and authorization core endpoints"),
        (name = "OAuth Clients", description = "OAuth2 client management operations"),
        (name = "OAuth Scopes", description = "OAuth2 scope management and validation"),
        (name = "OAuth Tokens", description = "OAuth2 token management and analytics"),
        (name = "OAuth Authorization", description = "OAuth2 authorization management"),
        (name = "OAuth Admin", description = "OAuth2 administrative dashboard and system management"),
        (name = "Personal Access Tokens", description = "Personal access token management"),
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

// Query Builder Documentation Schemas

/// Query filter parameter schema for API documentation
#[derive(Serialize, Deserialize, ToSchema)]
pub struct QueryFilterSchema {
    /// Field name to filter on
    #[schema(example = "name")]
    pub field: String,

    /// Filter operator (see FilterOperatorSchema for valid values)
    #[schema(example = "eq")]
    pub operator: String,

    /// Filter value (can be single value, array, or range)
    #[schema(example = "John")]
    pub value: serde_json::Value,

    /// Whether to combine with AND (true) or OR (false)
    #[schema(example = true)]
    pub and: bool,
}

/// Supported filter operators
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum FilterOperatorSchema {
    /// Equals (=)
    Eq,
    /// Not equals (!=)
    Ne,
    /// Greater than (>)
    Gt,
    /// Greater than or equal (>=)
    Gte,
    /// Less than (<)
    Lt,
    /// Less than or equal (<=)
    Lte,
    /// LIKE pattern matching
    Like,
    /// ILIKE case-insensitive pattern matching
    Ilike,
    /// IN (value1, value2, ...)
    In,
    /// NOT IN (value1, value2, ...)
    NotIn,
    /// IS NULL
    IsNull,
    /// IS NOT NULL
    IsNotNull,
    /// BETWEEN value1 AND value2
    Between,
    /// Contains pattern (ILIKE %pattern%)
    Contains,
    /// Starts with pattern (LIKE pattern%)
    StartsWith,
    /// Ends with pattern (LIKE %pattern)
    EndsWith,
}

/// Query sort parameter schema for API documentation
#[derive(Serialize, Deserialize, ToSchema)]
pub struct QuerySortSchema {
    /// Field name to sort by
    #[schema(example = "created_at")]
    pub field: String,

    /// Sort direction
    #[schema(example = "desc")]
    pub direction: String, // "asc" or "desc"
}

/// Query include parameter schema for API documentation
#[derive(Serialize, Deserialize, ToSchema)]
pub struct QueryIncludeSchema {
    /// Relationship name to include
    #[schema(example = "organization")]
    pub relation: String,

    /// Nested includes for the relationship
    #[schema(example = "organization.positions")]
    pub nested: Option<Vec<String>>,

    /// Fields to select from the included relationship
    #[schema(example = "id,name,type")]
    pub fields: Option<Vec<String>>,
}

/// Query pagination parameter schema for API documentation
#[derive(Serialize, Deserialize, ToSchema)]
pub struct QueryPaginationSchema {
    /// Page number (1-based) for offset pagination
    #[schema(example = 1)]
    pub page: Option<u32>,

    /// Number of items per page
    #[schema(example = 15, minimum = 1, maximum = 100)]
    pub per_page: Option<u32>,

    /// Cursor for cursor-based pagination
    #[schema(example = "eyJpZCI6MTAwfQ==")]
    pub cursor: Option<String>,

    /// Pagination type preference
    #[schema(example = "cursor")]
    pub pagination_type: Option<String>, // "cursor" or "offset"
}