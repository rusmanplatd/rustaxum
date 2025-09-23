use utoipa::OpenApi;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use crate::app::models::DieselUlid;

/// OAuth2/Passport API Documentation
///
/// This module contains comprehensive OpenAPI documentation for all OAuth2/Passport endpoints.
/// The implementation provides Laravel Passport-compatible OAuth2 functionality with enterprise-grade features.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "OAuth2/Passport API",
        version = "1.0.0",
        description = "Comprehensive OAuth2 authentication and authorization system inspired by Laravel Passport\n\n## Features\n\n- **Complete OAuth2 Implementation**: Authorization Code, Client Credentials, Password, and Refresh Token grants\n- **Personal Access Tokens**: API tokens for personal use\n- **Comprehensive Scope Management**: Fine-grained permission control\n- **Admin Dashboard**: Complete OAuth2 system monitoring and management\n- **Client Management**: Full CRUD operations for OAuth2 clients\n- **Token Management**: Advanced token lifecycle and analytics\n- **PKCE Support**: Secure authorization code flow\n- **Rate Limiting**: Built-in abuse protection\n\n## Authentication\n\nMost endpoints require Bearer token authentication. Include your access token in the Authorization header:\n\n```\nAuthorization: Bearer your_access_token_here\n```\n\n## Grant Types\n\n### Authorization Code Grant\nStandard OAuth2 flow for web applications with server-side code.\n\n### Client Credentials Grant\nFor machine-to-machine authentication where no user interaction is required.\n\n### Password Grant\nFor trusted first-party applications where the user provides credentials directly.\n\n### Refresh Token Grant\nTo obtain new access tokens when the current token expires.\n\n## Scopes\n\nScopes define the level of access granted to tokens:\n\n- `*` - Full access to all resources\n- `read` - Read access to user resources\n- `write` - Write access to user resources\n- `admin` - Administrative access\n- `user:read` - Read user profile information\n- `user:write` - Modify user profile information\n- `oauth:clients` - Manage OAuth clients\n- `oauth:tokens` - Manage OAuth tokens\n\nAnd many more fine-grained scopes for specific resource access.",
        contact(
            name = "OAuth API Support",
            email = "oauth@rustaxum.dev"
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
    paths(
        // OAuth Core Endpoints
        crate::app::http::controllers::oauth::oauth_controller::authorize,
        crate::app::http::controllers::oauth::oauth_controller::token,
        crate::app::http::controllers::oauth::oauth_controller::introspect,
        crate::app::http::controllers::oauth::oauth_controller::revoke,

        // Client Management
        crate::app::http::controllers::oauth::client_controller::create_client,
        crate::app::http::controllers::oauth::client_controller::list_clients,
        crate::app::http::controllers::oauth::client_controller::get_client,
        crate::app::http::controllers::oauth::client_controller::update_client,
        crate::app::http::controllers::oauth::client_controller::delete_client,
        crate::app::http::controllers::oauth::client_controller::regenerate_secret,

        // Personal Access Tokens
        crate::app::http::controllers::oauth::personal_access_token_controller::create_personal_access_token,
        crate::app::http::controllers::oauth::personal_access_token_controller::list_personal_access_tokens,
        crate::app::http::controllers::oauth::personal_access_token_controller::revoke_personal_access_token,

        // Scope Management
        crate::app::http::controllers::oauth::scope_controller::create_scope,
        crate::app::http::controllers::oauth::scope_controller::list_scopes,
        crate::app::http::controllers::oauth::scope_controller::get_scope,
        crate::app::http::controllers::oauth::scope_controller::get_scope_by_name,
        crate::app::http::controllers::oauth::scope_controller::update_scope,
        crate::app::http::controllers::oauth::scope_controller::delete_scope,
        crate::app::http::controllers::oauth::scope_controller::validate_scopes,

        // Authorization Management
        crate::app::http::controllers::oauth::authorization_controller::create_auth_code,
        crate::app::http::controllers::oauth::authorization_controller::list_auth_codes,
        crate::app::http::controllers::oauth::authorization_controller::get_auth_code,
        crate::app::http::controllers::oauth::authorization_controller::revoke_auth_code,
        crate::app::http::controllers::oauth::authorization_controller::list_authorized_clients,
        crate::app::http::controllers::oauth::authorization_controller::revoke_client_authorization,

        // Token Management
        crate::app::http::controllers::oauth::token_controller::list_tokens,
        crate::app::http::controllers::oauth::token_controller::get_token,
        crate::app::http::controllers::oauth::token_controller::revoke_token,
        crate::app::http::controllers::oauth::token_controller::extend_token,
        crate::app::http::controllers::oauth::token_controller::revoke_tokens,
        crate::app::http::controllers::oauth::token_controller::get_token_stats,
        crate::app::http::controllers::oauth::token_controller::get_my_tokens,

        // Admin Dashboard
        crate::app::http::controllers::oauth::admin_controller::get_dashboard_stats,
        crate::app::http::controllers::oauth::admin_controller::get_system_config,
        crate::app::http::controllers::oauth::admin_controller::system_cleanup,
        crate::app::http::controllers::oauth::admin_controller::get_audit_log,
        crate::app::http::controllers::oauth::admin_controller::export_data,
    ),
    components(
        schemas(
            // Core OAuth Models
            OAuthClient,
            CreateClientRequest,
            UpdateClientRequest,
            ClientResponse,

            OAuthScope,
            CreateScopeRequest,
            UpdateScopeRequest,
            ScopeResponse,

            OAuthToken,
            CreateTokenRequest,
            TokenResponse,
            AccessTokenResponse,

            PersonalAccessToken,
            CreatePersonalAccessTokenRequest,
            PersonalAccessTokenResponse,

            // Authorization Models
            AuthorizationCode,
            CreateAuthCodeRequest,
            AuthCodeResponse,

            // Request/Response Models
            TokenRequest,
            AuthorizeRequest,
            IntrospectRequest,
            RevokeRequest,

            // Query Parameters
            ListScopesQuery,
            ListTokensQuery,
            ListAuthCodesQuery,
            AuthorizedClientQuery,
            TokenStatsQuery,
            AdminStatsQuery,

            // Admin Dashboard Models
            OAuthDashboardStats,
            OverviewStats,
            ClientStats,
            TokenStats,
            ScopeStats,
            SystemHealth,
            ActivityItem,

            // Bulk Operations
            RevokeTokensRequest,
            ExtendTokenRequest,
            SystemCleanupRequest,
            CleanupResult,

            // Common Response Types
            OAuthErrorResponse,
            OAuthMessageResponse,
            ValidationErrorResponse,
        ),
    ),
    tags(
        (name = "OAuth Core", description = "Core OAuth2 authorization and token endpoints"),
        (name = "OAuth Clients", description = "OAuth2 client management - create, update, delete, and manage client applications"),
        (name = "OAuth Scopes", description = "OAuth2 scope management - define and manage permission scopes"),
        (name = "OAuth Tokens", description = "OAuth2 token management - monitor, revoke, and analyze access tokens"),
        (name = "Personal Access Tokens", description = "Personal access token management for API access"),
        (name = "OAuth Authorization", description = "Authorization management - track and manage user authorizations"),
        (name = "OAuth Admin", description = "Administrative dashboard and system management for OAuth2"),
    )
)]
pub struct OAuthApiDoc;

// ============================================================================
// CORE OAUTH MODELS
// ============================================================================

/// OAuth2 client representation
#[derive(Serialize, Deserialize, ToSchema, Clone)]
#[schema(description = "OAuth2 client application")]
pub struct OAuthClient {
    /// Unique client identifier
    #[schema(example = "01H8EXAMPLE123")]
    pub id: DieselUlid,
    /// User ID that owns this client (optional for system clients)
    #[schema(example = "01H8USER123")]
    pub user_id: Option<String>,
    /// Client application name
    #[schema(example = "My OAuth App")]
    pub name: String,
    /// Client secret (only shown on creation)
    #[schema(example = "abc123secretkey")]
    pub secret: Option<String>,
    /// OAuth provider (optional)
    pub provider: Option<String>,
    /// Comma-separated redirect URIs
    #[schema(example = "http://localhost:3000/callback,https://app.example.com/auth")]
    pub redirect_uris: String,
    /// Whether this is a personal access client
    #[schema(example = false)]
    pub personal_access_client: bool,
    /// Whether this client supports password grants
    #[schema(example = false)]
    pub password_client: bool,
    /// Whether the client has been revoked
    #[schema(example = false)]
    pub revoked: bool,
    /// When the client was created
    pub created_at: DateTime<Utc>,
    /// When the client was last updated
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new OAuth2 client
#[derive(Deserialize, ToSchema)]
#[schema(description = "Request payload for creating a new OAuth2 client")]
pub struct CreateClientRequest {
    /// Client application name
    #[schema(example = "My OAuth App")]
    pub name: String,
    /// List of allowed redirect URIs
    #[schema(example = "http://localhost:3000/callback")]
    pub redirect_uris: Vec<String>,
    /// Whether this should be a personal access client
    #[schema(example = false)]
    pub personal_access_client: Option<bool>,
    /// Whether this client should support password grants
    #[schema(example = false)]
    pub password_client: Option<bool>,
}

/// Request to update an OAuth2 client
#[derive(Deserialize, ToSchema)]
#[schema(description = "Request payload for updating an OAuth2 client")]
pub struct UpdateClientRequest {
    /// Updated client name
    #[schema(example = "Updated App Name")]
    pub name: Option<String>,
    /// Updated redirect URIs
    #[schema(example = "example_value")]
    // Original: #[schema(example = ["https://newdomain.com/callback"])]
    pub redirect_uris: Option<Vec<String>>,
    /// Whether to revoke the client
    #[schema(example = false)]
    pub revoked: Option<bool>,
}

/// OAuth2 client response
#[derive(Serialize, ToSchema)]
#[schema(description = "OAuth2 client response with public information")]
pub struct ClientResponse {
    /// Unique client identifier
    pub id: DieselUlid,
    /// Client application name
    #[schema(example = "My OAuth App")]
    pub name: String,
    /// Client secret (only included when creating or regenerating)
    pub secret: Option<String>,
    /// List of allowed redirect URIs
    pub redirect_uris: Vec<String>,
    /// Whether this is a personal access client
    pub personal_access_client: bool,
    /// Whether this client supports password grants
    pub password_client: bool,
    /// Whether the client has been revoked
    pub revoked: bool,
    /// When the client was created
    pub created_at: DateTime<Utc>,
    /// When the client was last updated
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// OAUTH SCOPE MODELS
// ============================================================================

/// OAuth2 scope definition
#[derive(Serialize, Deserialize, ToSchema, Clone)]
#[schema(description = "OAuth2 permission scope")]
pub struct OAuthScope {
    /// Unique scope identifier
    pub id: DieselUlid,
    /// Scope name (e.g., 'user:read', 'admin')
    #[schema(example = "user:read")]
    pub name: String,
    /// Human-readable description
    #[schema(example = "Read user profile information")]
    pub description: Option<String>,
    /// Whether this scope is included by default
    #[schema(example = false)]
    pub is_default: bool,
    /// When the scope was created
    pub created_at: DateTime<Utc>,
    /// When the scope was last updated
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new OAuth2 scope
#[derive(Deserialize, ToSchema)]
#[schema(description = "Request to create a new OAuth2 scope")]
pub struct CreateScopeRequest {
    /// Scope name (e.g., 'user:read', 'admin')
    #[schema(example = "user:read")]
    pub name: String,
    /// Human-readable description of the scope
    #[schema(example = "Read user profile information")]
    pub description: Option<String>,
    /// Whether this scope should be included by default
    #[schema(example = false)]
    pub is_default: Option<bool>,
}

/// Request to update an OAuth2 scope
#[derive(Deserialize, ToSchema)]
#[schema(description = "Request to update an OAuth2 scope")]
pub struct UpdateScopeRequest {
    /// Updated scope name
    #[schema(example = "user:write")]
    pub name: Option<String>,
    /// Updated description
    #[schema(example = "Write user profile information")]
    pub description: Option<String>,
    /// Updated default status
    #[schema(example = true)]
    pub is_default: Option<bool>,
}

/// OAuth2 scope response
#[derive(Serialize, ToSchema)]
#[schema(description = "OAuth2 scope response")]
pub struct ScopeResponse {
    /// Unique scope identifier
    pub id: DieselUlid,
    /// Scope name
    #[schema(example = "user:read")]
    pub name: String,
    /// Human-readable description
    #[schema(example = "Read user profile information")]
    pub description: Option<String>,
    /// Whether this scope is included by default
    pub is_default: bool,
    /// When the scope was created
    pub created_at: DateTime<Utc>,
    /// When the scope was last updated
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// TOKEN MODELS
// ============================================================================

/// OAuth2 access token
#[derive(Serialize, Deserialize, ToSchema, Clone)]
#[schema(description = "OAuth2 access token")]
pub struct OAuthToken {
    /// Unique token identifier
    pub id: DieselUlid,
    /// User ID that owns this token
    #[schema(example = "01H8USER123")]
    pub user_id: Option<String>,
    /// Client ID that created this token
    #[schema(example = "01H8CLIENT123")]
    pub client_id: String,
    /// Token name/description
    #[schema(example = "My API Token")]
    pub name: Option<String>,
    /// Comma-separated list of scopes
    #[schema(example = "read,write")]
    pub scopes: String,
    /// Whether the token has been revoked
    #[schema(example = false)]
    pub revoked: bool,
    /// When the token expires
    pub expires_at: Option<DateTime<Utc>>,
    /// When the token was created
    pub created_at: DateTime<Utc>,
    /// When the token was last updated
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new token
#[derive(Deserialize, ToSchema)]
#[schema(description = "Request to create a new access token")]
pub struct CreateTokenRequest {
    /// Token name/description
    #[schema(example = "My API Token")]
    pub name: String,
    /// List of requested scopes
    #[schema(example = "example_value")]
    // Original: #[schema(example = ["read", "write"])]
    pub scopes: Vec<String>,
    /// Token expiration in seconds (optional)
    #[schema(example = 3600)]
    pub expires_in_seconds: Option<i64>,
}

/// OAuth2 token response
#[derive(Serialize, ToSchema)]
#[schema(description = "OAuth2 token response from token endpoint")]
pub struct TokenResponse {
    /// The access token
    #[schema(example = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...")]
    pub access_token: String,
    /// Token type (always 'Bearer')
    #[schema(example = "Bearer")]
    pub token_type: String,
    /// Token lifetime in seconds
    #[schema(example = 3600)]
    pub expires_in: i64,
    /// Refresh token (if applicable)
    #[schema(example = "refresh_token_string")]
    pub refresh_token: Option<String>,
    /// Granted scopes (space-separated)
    #[schema(example = "read write")]
    pub scope: String,
}

/// Access token detailed response
#[derive(Serialize, ToSchema)]
#[schema(description = "Detailed access token information")]
pub struct AccessTokenResponse {
    /// Unique token identifier
    pub id: DieselUlid,
    /// User ID that owns this token
    pub user_id: Option<String>,
    /// Client ID that created this token
    pub client_id: String,
    /// Token name/description
    pub name: Option<String>,
    /// List of granted scopes
    pub scopes: Vec<String>,
    /// Whether the token has been revoked
    pub revoked: bool,
    /// When the token expires
    pub expires_at: Option<DateTime<Utc>>,
    /// When the token was created
    pub created_at: DateTime<Utc>,
    /// When the token was last updated
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// REQUEST/RESPONSE MODELS
// ============================================================================

/// OAuth2 token request (RFC 6749)
#[derive(Deserialize, ToSchema)]
#[schema(description = "OAuth2 token request following RFC 6749")]
pub struct TokenRequest {
    /// Grant type (authorization_code, client_credentials, password, refresh_token)
    #[schema(example = "authorization_code")]
    pub grant_type: String,
    /// Authorization code (for authorization_code grant)
    #[schema(example = "auth_code_123")]
    pub code: Option<String>,
    /// Redirect URI (for authorization_code grant)
    #[schema(example = "http://localhost:3000/callback")]
    pub redirect_uri: Option<String>,
    /// Client identifier
    #[schema(example = "01H8CLIENT123")]
    pub client_id: String,
    /// Client secret
    #[schema(example = "client_secret_abc123")]
    pub client_secret: Option<String>,
    /// PKCE code verifier
    #[schema(example = "code_verifier_xyz")]
    pub code_verifier: Option<String>,
    /// Refresh token (for refresh_token grant)
    #[schema(example = "refresh_token_def456")]
    pub refresh_token: Option<String>,
    /// Requested scopes (space-separated)
    #[schema(example = "read write")]
    pub scope: Option<String>,
    /// Username (for password grant)
    #[schema(example = "user@example.com")]
    pub username: Option<String>,
    /// Password (for password grant)
    #[schema(example = "password123")]
    pub password: Option<String>,
}

/// OAuth2 authorization request
#[derive(Deserialize, ToSchema)]
#[schema(description = "OAuth2 authorization request")]
pub struct AuthorizeRequest {
    /// Response type (always 'code')
    #[schema(example = "code")]
    pub response_type: String,
    /// Client identifier
    #[schema(example = "01H8CLIENT123")]
    pub client_id: String,
    /// Redirect URI
    #[schema(example = "http://localhost:3000/callback")]
    pub redirect_uri: String,
    /// Requested scopes (space-separated)
    #[schema(example = "read write")]
    pub scope: Option<String>,
    /// Anti-CSRF state parameter
    #[schema(example = "random_state_value")]
    pub state: Option<String>,
    /// PKCE code challenge
    #[schema(example = "code_challenge_abc")]
    pub code_challenge: Option<String>,
    /// PKCE code challenge method
    #[schema(example = "S256")]
    pub code_challenge_method: Option<String>,
}

/// Token introspection request
#[derive(Deserialize, ToSchema)]
#[schema(description = "Token introspection request (RFC 7662)")]
pub struct IntrospectRequest {
    /// The token to introspect
    #[schema(example = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...")]
    pub token: String,
}

/// Token revocation request
#[derive(Deserialize, ToSchema)]
#[schema(description = "Token revocation request (RFC 7009)")]
pub struct RevokeRequest {
    /// The token to revoke
    #[schema(example = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...")]
    pub token: String,
    /// Token type hint (access_token or refresh_token)
    #[schema(example = "access_token")]
    pub token_type_hint: Option<String>,
}

// ============================================================================
// QUERY PARAMETER MODELS
// ============================================================================

/// Query parameters for listing scopes
#[derive(Deserialize, ToSchema)]
#[schema(description = "Query parameters for scope listing")]
pub struct ListScopesQuery {
    /// Filter to only default scopes
    #[schema(example = false)]
    pub default_only: Option<bool>,
    /// Search term to filter scopes
    #[schema(example = "user")]
    pub search: Option<String>,
}

/// Query parameters for listing tokens
#[derive(Deserialize, ToSchema)]
#[schema(description = "Query parameters for token listing")]
pub struct ListTokensQuery {
    /// Filter by user ID
    #[schema(example = "01H8USER123")]
    pub user_id: Option<String>,
    /// Filter by client ID
    #[schema(example = "01H8CLIENT123")]
    pub client_id: Option<String>,
    /// Filter by scope
    #[schema(example = "read")]
    pub scope: Option<String>,
    /// Show only active tokens
    #[schema(example = true)]
    pub active_only: Option<bool>,
    /// Results limit
    #[schema(example = 50)]
    pub limit: Option<i64>,
    /// Results offset
    #[schema(example = 0)]
    pub offset: Option<i64>,
}

/// Query parameters for admin statistics
#[derive(Deserialize, ToSchema)]
#[schema(description = "Query parameters for admin dashboard statistics")]
pub struct AdminStatsQuery {
    /// Number of days to include in statistics
    #[schema(example = 30)]
    pub days: Option<i32>,
    /// Whether to include detailed breakdown
    #[schema(example = true)]
    pub include_details: Option<bool>,
}

// ============================================================================
// ERROR RESPONSE MODELS
// ============================================================================

/// Standard OAuth2 error response
#[derive(Serialize, ToSchema)]
#[schema(description = "OAuth2 error response following RFC 6749")]
pub struct OAuthErrorResponse {
    /// Error code
    #[schema(example = "invalid_client")]
    pub error: String,
    /// Human-readable error description
    #[schema(example = "Client authentication failed")]
    pub error_description: Option<String>,
    /// URI for more information about the error
    #[schema(example = "https://docs.example.com/oauth/errors#invalid_client")]
    pub error_uri: Option<String>,
}

/// Success message response
#[derive(Serialize, ToSchema)]
#[schema(description = "Success message response")]
pub struct OAuthMessageResponse {
    /// Success message
    #[schema(example = "Operation completed successfully")]
    pub message: String,
}

/// Validation error response
#[derive(Serialize, ToSchema)]
#[schema(description = "Validation error response with field-specific errors")]
pub struct ValidationErrorResponse {
    /// Error message
    #[schema(example = "The given data was invalid")]
    pub message: String,
    /// Field-specific errors
    #[schema(example = r#"{"name": ["The name field is required"], "scopes": ["At least one scope must be specified"]}"#)]
    pub errors: std::collections::HashMap<String, Vec<String>>,
}

// ============================================================================
// ADMIN DASHBOARD MODELS
// ============================================================================

/// OAuth2 dashboard statistics overview
#[derive(Serialize, ToSchema)]
#[schema(description = "Comprehensive OAuth2 dashboard statistics")]
pub struct OAuthDashboardStats {
    /// High-level overview statistics
    pub overview: OverviewStats,
    /// Client-related statistics
    pub clients: ClientStats,
    /// Token-related statistics
    pub tokens: TokenStats,
    /// Scope-related statistics
    pub scopes: ScopeStats,
    /// Recent system activity
    pub recent_activity: Vec<ActivityItem>,
    /// System health information
    pub system_health: SystemHealth,
}

/// Overview statistics
#[derive(Serialize, ToSchema)]
#[schema(description = "High-level OAuth2 system statistics")]
pub struct OverviewStats {
    /// Total number of OAuth2 clients
    #[schema(example = 25)]
    pub total_clients: i64,
    /// Number of active clients
    #[schema(example = 23)]
    pub active_clients: i64,
    /// Total number of access tokens
    #[schema(example = 1250)]
    pub total_tokens: i64,
    /// Number of currently valid tokens
    #[schema(example = 987)]
    pub active_tokens: i64,
    /// Total number of defined scopes
    #[schema(example = 20)]
    pub total_scopes: i64,
    /// Number of users with active tokens
    #[schema(example = 450)]
    pub total_users_with_tokens: i64,
    /// Tokens created today
    #[schema(example = 45)]
    pub tokens_created_today: i64,
    /// Tokens created this week
    #[schema(example = 312)]
    pub tokens_created_this_week: i64,
}

/// Client statistics
#[derive(Serialize, ToSchema)]
#[schema(description = "OAuth2 client statistics")]
pub struct ClientStats {
    /// Statistics by client type
    pub by_type: ClientsByType,
    /// Top clients by token usage
    pub top_clients_by_tokens: Vec<TopClientInfo>,
    /// Number of revoked clients
    #[schema(example = 2)]
    pub revoked_clients: i64,
    /// Clients created this month
    #[schema(example = 8)]
    pub clients_created_this_month: i64,
}

/// Client type breakdown
#[derive(Serialize, ToSchema)]
#[schema(description = "Client statistics by type")]
pub struct ClientsByType {
    /// Personal access clients
    #[schema(example = 5)]
    pub personal_access: i64,
    /// Password grant clients
    #[schema(example = 8)]
    pub password_grant: i64,
    /// Authorization code clients
    #[schema(example = 10)]
    pub authorization_code: i64,
    /// Client credentials clients
    #[schema(example = 2)]
    pub client_credentials: i64,
}

/// Top client information
#[derive(Serialize, ToSchema)]
#[schema(description = "Information about top clients by usage")]
pub struct TopClientInfo {
    /// Client identifier
    #[schema(example = "01H8CLIENT123")]
    pub client_id: String,
    /// Client name
    #[schema(example = "My OAuth App")]
    pub client_name: String,
    /// Total number of tokens created
    #[schema(example = 150)]
    pub token_count: i64,
    /// Currently active tokens
    #[schema(example = 120)]
    pub active_token_count: i64,
    /// Last time this client was used
    pub last_used: Option<DateTime<Utc>>,
}

/// Token statistics
#[derive(Serialize, ToSchema)]
#[schema(description = "OAuth2 token statistics")]
pub struct TokenStats {
    /// Token distribution by grant type
    pub by_grant_type: TokensByGrantType,
    /// Token expiry distribution
    pub expiry_distribution: ExpiryDistribution,
    /// Number of revoked tokens
    #[schema(example = 123)]
    pub revoked_tokens: i64,
    /// Number of expired tokens
    #[schema(example = 234)]
    pub expired_tokens: i64,
    /// Token usage by scope
    pub tokens_by_scope: Vec<TokenScopeStats>,
}

/// Token distribution by grant type
#[derive(Serialize, ToSchema)]
#[schema(description = "Token statistics by grant type")]
pub struct TokensByGrantType {
    /// Authorization code flow tokens
    #[schema(example = 456)]
    pub authorization_code: i64,
    /// Client credentials tokens
    #[schema(example = 234)]
    pub client_credentials: i64,
    /// Password grant tokens
    #[schema(example = 345)]
    pub password: i64,
    /// Personal access tokens
    #[schema(example = 156)]
    pub personal_access: i64,
    /// Refresh tokens
    #[schema(example = 59)]
    pub refresh_token: i64,
}

/// Token expiry distribution
#[derive(Serialize, ToSchema)]
#[schema(description = "Distribution of token expiry times")]
pub struct ExpiryDistribution {
    /// Tokens expiring within an hour
    #[schema(example = 45)]
    pub expires_within_hour: i64,
    /// Tokens expiring within a day
    #[schema(example = 234)]
    pub expires_within_day: i64,
    /// Tokens expiring within a week
    #[schema(example = 456)]
    pub expires_within_week: i64,
    /// Tokens expiring within a month
    #[schema(example = 245)]
    pub expires_within_month: i64,
    /// Tokens that never expire
    #[schema(example = 67)]
    pub never_expires: i64,
}

/// Token usage by scope
#[derive(Serialize, ToSchema)]
#[schema(description = "Token usage statistics by scope")]
pub struct TokenScopeStats {
    /// Scope name
    #[schema(example = "read")]
    pub scope: String,
    /// Number of tokens with this scope
    #[schema(example = 567)]
    pub token_count: i64,
    /// Percentage of total tokens
    #[schema(example = 45.6)]
    pub percentage: f64,
}

/// Scope statistics
#[derive(Serialize, ToSchema)]
#[schema(description = "OAuth2 scope statistics")]
pub struct ScopeStats {
    /// Total number of defined scopes
    #[schema(example = 20)]
    pub total_scopes: i64,
    /// Number of default scopes
    #[schema(example = 3)]
    pub default_scopes: i64,
    /// Most frequently used scopes
    pub most_used_scopes: Vec<ScopeUsageInfo>,
    /// Scopes that are never used
    #[schema(example = "example_value")]
    // Original: #[schema(example = ["analytics:write"])]
    pub unused_scopes: Vec<String>,
}

/// Scope usage information
#[derive(Serialize, ToSchema)]
#[schema(description = "Scope usage statistics")]
pub struct ScopeUsageInfo {
    /// Scope name
    #[schema(example = "read")]
    pub scope_name: String,
    /// Number of times this scope is used
    #[schema(example = 567)]
    pub usage_count: i64,
    /// Percentage of total usage
    #[schema(example = 45.6)]
    pub percentage: f64,
}

/// System activity item
#[derive(Serialize, ToSchema)]
#[schema(description = "System activity log entry")]
pub struct ActivityItem {
    /// When the activity occurred
    pub timestamp: DateTime<Utc>,
    /// Type of activity
    #[schema(example = "token_created")]
    pub activity_type: String,
    /// Human-readable description
    #[schema(example = "New access token created")]
    pub description: String,
    /// User ID involved in the activity
    #[schema(example = "01H8USER123")]
    pub user_id: Option<String>,
    /// Client ID involved in the activity
    #[schema(example = "01H8CLIENT123")]
    pub client_id: Option<String>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// System health information
#[derive(Serialize, ToSchema)]
#[schema(description = "OAuth2 system health status")]
pub struct SystemHealth {
    /// Overall system status
    #[schema(example = "healthy")]
    pub status: String,
    /// System uptime in hours
    #[schema(example = 72.5)]
    pub uptime_hours: f64,
    /// Memory usage percentage
    #[schema(example = "45%")]
    pub memory_usage: Option<String>,
    /// Database connection status
    #[schema(example = "connected")]
    pub database_status: String,
    /// Current system issues
    pub issues: Vec<HealthIssue>,
    /// System recommendations
    #[schema(example = "example_value")]
    // Original: #[schema(example = ["Consider cleaning up expired tokens older than 30 days"])]
    pub recommendations: Vec<String>,
}

/// System health issue
#[derive(Serialize, ToSchema)]
#[schema(description = "System health issue")]
pub struct HealthIssue {
    /// Issue severity level
    #[schema(example = "warning")]
    pub severity: String,
    /// Issue description
    #[schema(example = "High number of expired tokens")]
    pub issue: String,
    /// Recommended action
    #[schema(example = "Run cleanup operation")]
    pub recommendation: String,
}

// ============================================================================
// BULK OPERATION MODELS
// ============================================================================

/// Request to revoke multiple tokens
#[derive(Deserialize, ToSchema)]
#[schema(description = "Request to revoke multiple tokens at once")]
pub struct RevokeTokensRequest {
    /// List of token IDs to revoke
    #[schema(example = "example_value")]
    // Original: #[schema(example = ["01H8TOKEN1", "01H8TOKEN2"])]
    pub token_ids: Vec<String>,
}

/// Request to extend token expiration
#[derive(Deserialize, ToSchema)]
#[schema(description = "Request to extend token expiration")]
pub struct ExtendTokenRequest {
    /// Additional expiration time in seconds
    #[schema(example = 3600)]
    pub expires_in_seconds: i64,
}

/// System cleanup request
#[derive(Deserialize, ToSchema)]
#[schema(description = "Request to perform system cleanup operations")]
pub struct SystemCleanupRequest {
    /// Whether to remove expired tokens
    #[schema(example = true)]
    pub remove_expired_tokens: Option<bool>,
    /// Whether to remove revoked tokens
    #[schema(example = true)]
    pub remove_revoked_tokens: Option<bool>,
    /// Whether to remove expired authorization codes
    #[schema(example = true)]
    pub remove_expired_auth_codes: Option<bool>,
    /// Remove items older than this many days
    #[schema(example = 30)]
    pub older_than_days: Option<i32>,
}

/// System cleanup result
#[derive(Serialize, ToSchema)]
#[schema(description = "Result of system cleanup operation")]
pub struct CleanupResult {
    /// Number of tokens removed
    #[schema(example = 75)]
    pub tokens_removed: i64,
    /// Number of authorization codes removed
    #[schema(example = 10)]
    pub auth_codes_removed: i64,
    /// Cleanup operation summary
    #[schema(example = "Cleanup completed: remove expired tokens, remove revoked tokens. Removed 75 tokens and 10 auth codes.")]
    pub message: String,
}

// ============================================================================
// PERSONAL ACCESS TOKEN MODELS
// ============================================================================

/// Personal access token
#[derive(Serialize, Deserialize, ToSchema, Clone)]
#[schema(description = "Personal access token for API access")]
pub struct PersonalAccessToken {
    /// Unique token identifier
    pub id: DieselUlid,
    /// User ID that owns this token
    pub user_id: String,
    /// Client ID (personal access client)
    pub client_id: String,
    /// Token name/description
    #[schema(example = "My API Token")]
    pub name: String,
    /// List of granted scopes
    pub scopes: Vec<String>,
    /// Whether the token has been revoked
    pub revoked: bool,
    /// When the token expires
    pub expires_at: Option<DateTime<Utc>>,
    /// When the token was created
    pub created_at: DateTime<Utc>,
    /// When the token was last updated
    pub updated_at: DateTime<Utc>,
}

/// Request to create a personal access token
#[derive(Deserialize, ToSchema)]
#[schema(description = "Request to create a personal access token")]
pub struct CreatePersonalAccessTokenRequest {
    /// Token name/description
    #[schema(example = "My API Token")]
    pub name: String,
    /// List of requested scopes
    #[schema(example = "example_value")]
    // Original: #[schema(example = ["read", "write"])]
    pub scopes: Vec<String>,
    /// Token expiration in seconds (optional)
    #[schema(example = 31536000)]
    pub expires_in_seconds: Option<i64>,
}

/// Personal access token response
#[derive(Serialize, ToSchema)]
#[schema(description = "Personal access token creation response")]
pub struct PersonalAccessTokenResponse {
    /// Unique token identifier
    pub id: DieselUlid,
    /// Token name
    #[schema(example = "My API Token")]
    pub name: String,
    /// The actual access token (only shown on creation)
    #[schema(example = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...")]
    pub access_token: Option<String>,
    /// List of granted scopes
    pub scopes: Vec<String>,
    /// When the token expires
    pub expires_at: Option<DateTime<Utc>>,
    /// When the token was created
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// AUTHORIZATION CODE MODELS
// ============================================================================

/// OAuth2 authorization code
#[derive(Serialize, Deserialize, ToSchema, Clone)]
#[schema(description = "OAuth2 authorization code for authorization flow")]
pub struct AuthorizationCode {
    /// Unique authorization code identifier
    pub id: DieselUlid,
    /// User ID that authorized the request
    pub user_id: String,
    /// Client ID requesting authorization
    pub client_id: String,
    /// Comma-separated list of requested scopes
    pub scopes: String,
    /// Redirect URI for the authorization
    pub redirect_uri: String,
    /// PKCE code challenge (optional)
    pub challenge: Option<String>,
    /// PKCE code challenge method (optional)
    pub challenge_method: Option<String>,
    /// Whether the code has been revoked
    pub revoked: bool,
    /// When the code expires
    pub expires_at: Option<DateTime<Utc>>,
    /// When the code was created
    pub created_at: DateTime<Utc>,
    /// When the code was last updated
    pub updated_at: DateTime<Utc>,
}

/// Request to create an authorization code
#[derive(Deserialize, ToSchema)]
#[schema(description = "Request to create an authorization code (admin only)")]
pub struct CreateAuthCodeRequest {
    /// User ID granting authorization
    #[schema(example = "01H8USER123")]
    pub user_id: String,
    /// Client ID requesting authorization
    #[schema(example = "01H8CLIENT123")]
    pub client_id: String,
    /// List of requested scopes
    #[schema(example = "example_value")]
    // Original: #[schema(example = ["read", "write"])]
    pub scopes: Vec<String>,
    /// Redirect URI for authorization response
    #[schema(example = "http://localhost:3000/callback")]
    pub redirect_uri: String,
    /// PKCE code challenge
    #[schema(example = "code_challenge_abc")]
    pub challenge: Option<String>,
    /// PKCE code challenge method
    #[schema(example = "S256")]
    pub challenge_method: Option<String>,
    /// Code expiration in minutes
    #[schema(example = 10)]
    pub expires_in_minutes: Option<i64>,
}

/// Authorization code response
#[derive(Serialize, ToSchema)]
#[schema(description = "Authorization code information")]
pub struct AuthCodeResponse {
    /// Unique authorization code identifier
    pub id: String,
    /// User ID that granted authorization
    pub user_id: String,
    /// Client ID that requested authorization
    pub client_id: String,
    /// List of granted scopes
    pub scopes: Vec<String>,
    /// Redirect URI for the authorization
    pub redirect_uri: String,
    /// When the code expires
    pub expires_at: DateTime<Utc>,
    /// Whether the code has been revoked
    pub revoked: bool,
    /// When the code was created
    pub created_at: DateTime<Utc>,
}

/// Query parameters for listing authorization codes
#[derive(Deserialize, ToSchema)]
#[schema(description = "Query parameters for authorization code listing")]
pub struct ListAuthCodesQuery {
    /// Filter by user ID
    #[schema(example = "01H8USER123")]
    pub user_id: Option<String>,
    /// Filter by client ID
    #[schema(example = "01H8CLIENT123")]
    pub client_id: Option<String>,
    /// Show only expired codes
    #[schema(example = false)]
    pub expired: Option<bool>,
}

/// Query parameters for authorized clients
#[derive(Deserialize, ToSchema)]
#[schema(description = "Query parameters for authorized client listing")]
pub struct AuthorizedClientQuery {
    /// Filter by specific client ID
    #[schema(example = "01H8CLIENT123")]
    pub client_id: Option<String>,
    /// Filter by scope
    #[schema(example = "read")]
    pub scope: Option<String>,
}

/// Token statistics query parameters
#[derive(Deserialize, ToSchema)]
#[schema(description = "Query parameters for token statistics")]
pub struct TokenStatsQuery {
    /// Number of days to include in statistics
    #[schema(example = 30)]
    pub days: Option<i32>,
}

impl OAuthApiDoc {
    /// Generate OpenAPI specification as JSON
    pub fn openapi_json() -> String {
        Self::openapi().to_pretty_json().unwrap()
    }

    /// Generate OpenAPI specification as YAML
    pub fn openapi_yaml() -> String {
        serde_yaml::to_string(&Self::openapi()).unwrap()
    }
}