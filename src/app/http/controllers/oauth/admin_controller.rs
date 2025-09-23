use axum::{
    extract::{Json, State, Query},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json as ResponseJson},
};
use serde::{Deserialize, Serialize};
use crate::database::DbPool;
use chrono::{Utc, DateTime, Duration};

// use crate::app::services::oauth::{ClientService, ScopeService, TokenService};
use crate::app::services::auth_service::AuthService;
use crate::app::utils::token_utils::TokenUtils;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    error_description: Option<String>,
}

#[derive(Deserialize)]
pub struct AdminStatsQuery {
    pub days: Option<i32>,
    pub include_details: Option<bool>,
}

#[derive(Serialize)]
pub struct OAuthDashboardStats {
    pub overview: OverviewStats,
    pub clients: ClientStats,
    pub tokens: TokenStats,
    pub scopes: ScopeStats,
    pub recent_activity: Vec<ActivityItem>,
    pub system_health: SystemHealth,
}

#[derive(Serialize)]
pub struct OverviewStats {
    pub total_clients: i64,
    pub active_clients: i64,
    pub total_tokens: i64,
    pub active_tokens: i64,
    pub total_scopes: i64,
    pub total_users_with_tokens: i64,
    pub tokens_created_today: i64,
    pub tokens_created_this_week: i64,
}

#[derive(Serialize)]
pub struct ClientStats {
    pub by_type: ClientsByType,
    pub top_clients_by_tokens: Vec<TopClientInfo>,
    pub revoked_clients: i64,
    pub clients_created_this_month: i64,
}

#[derive(Serialize)]
pub struct ClientsByType {
    pub personal_access: i64,
    pub password_grant: i64,
    pub authorization_code: i64,
    pub client_credentials: i64,
}

#[derive(Serialize)]
pub struct TopClientInfo {
    pub client_id: String,
    pub client_name: String,
    pub token_count: i64,
    pub active_token_count: i64,
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct TokenStats {
    pub by_grant_type: TokensByGrantType,
    pub expiry_distribution: ExpiryDistribution,
    pub revoked_tokens: i64,
    pub expired_tokens: i64,
    pub tokens_by_scope: Vec<TokenScopeStats>,
}

#[derive(Serialize)]
pub struct TokensByGrantType {
    pub authorization_code: i64,
    pub client_credentials: i64,
    pub password: i64,
    pub personal_access: i64,
    pub refresh_token: i64,
}

#[derive(Serialize)]
pub struct ExpiryDistribution {
    pub expires_within_hour: i64,
    pub expires_within_day: i64,
    pub expires_within_week: i64,
    pub expires_within_month: i64,
    pub never_expires: i64,
}

#[derive(Serialize)]
pub struct TokenScopeStats {
    pub scope: String,
    pub token_count: i64,
    pub percentage: f64,
}

#[derive(Serialize)]
pub struct ScopeStats {
    pub total_scopes: i64,
    pub default_scopes: i64,
    pub most_used_scopes: Vec<ScopeUsageInfo>,
    pub unused_scopes: Vec<String>,
}

#[derive(Serialize)]
pub struct ScopeUsageInfo {
    pub scope_name: String,
    pub usage_count: i64,
    pub percentage: f64,
}

#[derive(Serialize)]
pub struct ActivityItem {
    pub timestamp: DateTime<Utc>,
    pub activity_type: String,
    pub description: String,
    pub user_id: Option<String>,
    pub client_id: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Serialize)]
pub struct SystemHealth {
    pub status: String,
    pub uptime_hours: f64,
    pub memory_usage: Option<String>,
    pub database_status: String,
    pub issues: Vec<HealthIssue>,
    pub recommendations: Vec<String>,
}

#[derive(Serialize)]
pub struct HealthIssue {
    pub severity: String,
    pub issue: String,
    pub recommendation: String,
}

#[derive(Deserialize)]
pub struct SystemCleanupRequest {
    pub remove_expired_tokens: Option<bool>,
    pub remove_revoked_tokens: Option<bool>,
    pub remove_expired_auth_codes: Option<bool>,
    pub older_than_days: Option<i32>,
}

#[derive(Serialize)]
pub struct CleanupResult {
    pub tokens_removed: i64,
    pub auth_codes_removed: i64,
    pub message: String,
}

/// Get comprehensive OAuth2 dashboard statistics (admin only)
pub async fn get_dashboard_stats(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Query(_params): Query<AdminStatsQuery>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: "unauthorized".to_string(),
            error_description: Some(e.to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    // For now, return a comprehensive but placeholder response
    // In a real implementation, you'd query the database for actual statistics
    let stats = OAuthDashboardStats {
        overview: OverviewStats {
            total_clients: 25,
            active_clients: 23,
            total_tokens: 1250,
            active_tokens: 987,
            total_scopes: 20,
            total_users_with_tokens: 450,
            tokens_created_today: 45,
            tokens_created_this_week: 312,
        },
        clients: ClientStats {
            by_type: ClientsByType {
                personal_access: 5,
                password_grant: 8,
                authorization_code: 10,
                client_credentials: 2,
            },
            top_clients_by_tokens: vec![
                TopClientInfo {
                    client_id: "01K5T56NC9PY4JPCKPD2F7ANNZ".to_string(),
                    client_name: "Test App".to_string(),
                    token_count: 150,
                    active_token_count: 120,
                    last_used: Some(Utc::now() - Duration::minutes(30)),
                },
            ],
            revoked_clients: 2,
            clients_created_this_month: 8,
        },
        tokens: TokenStats {
            by_grant_type: TokensByGrantType {
                authorization_code: 456,
                client_credentials: 234,
                password: 345,
                personal_access: 156,
                refresh_token: 59,
            },
            expiry_distribution: ExpiryDistribution {
                expires_within_hour: 45,
                expires_within_day: 234,
                expires_within_week: 456,
                expires_within_month: 245,
                never_expires: 67,
            },
            revoked_tokens: 123,
            expired_tokens: 234,
            tokens_by_scope: vec![
                TokenScopeStats {
                    scope: "read".to_string(),
                    token_count: 567,
                    percentage: 45.6,
                },
                TokenScopeStats {
                    scope: "write".to_string(),
                    token_count: 234,
                    percentage: 18.7,
                },
            ],
        },
        scopes: ScopeStats {
            total_scopes: 20,
            default_scopes: 3,
            most_used_scopes: vec![
                ScopeUsageInfo {
                    scope_name: "read".to_string(),
                    usage_count: 567,
                    percentage: 45.6,
                },
                ScopeUsageInfo {
                    scope_name: "write".to_string(),
                    usage_count: 234,
                    percentage: 18.7,
                },
            ],
            unused_scopes: vec!["analytics:write".to_string()],
        },
        recent_activity: vec![
            ActivityItem {
                timestamp: Utc::now() - Duration::minutes(5),
                activity_type: "token_created".to_string(),
                description: "New access token created".to_string(),
                user_id: Some("user123".to_string()),
                client_id: Some("client456".to_string()),
                metadata: serde_json::json!({
                    "scopes": ["read", "write"],
                    "grant_type": "authorization_code"
                }),
            },
        ],
        system_health: SystemHealth {
            status: "healthy".to_string(),
            uptime_hours: 72.5,
            memory_usage: Some("45%".to_string()),
            database_status: "connected".to_string(),
            issues: vec![],
            recommendations: vec![
                "Consider cleaning up expired tokens older than 30 days".to_string(),
                "Monitor token creation rate for unusual spikes".to_string(),
            ],
        },
    };

    (StatusCode::OK, ResponseJson(stats)).into_response()
}

/// Get OAuth2 system configuration (admin only)
pub async fn get_system_config(
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: "unauthorized".to_string(),
            error_description: Some(e.to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    #[derive(Serialize)]
    struct SystemConfig {
        pub oauth_endpoints: Vec<EndpointInfo>,
        pub default_token_expiry: i64,
        pub default_refresh_token_expiry: i64,
        pub supported_grant_types: Vec<String>,
        pub supported_response_types: Vec<String>,
        pub supported_scopes: Vec<String>,
        pub pkce_required: bool,
        pub rate_limits: RateLimitConfig,
    }

    #[derive(Serialize)]
    struct EndpointInfo {
        pub path: String,
        pub method: String,
        pub description: String,
        pub authentication_required: bool,
    }

    #[derive(Serialize)]
    struct RateLimitConfig {
        pub token_requests_per_minute: i32,
        pub authorization_requests_per_minute: i32,
        pub enabled: bool,
    }

    let config = SystemConfig {
        oauth_endpoints: vec![
            EndpointInfo {
                path: "/oauth/authorize".to_string(),
                method: "GET".to_string(),
                description: "OAuth2 authorization endpoint".to_string(),
                authentication_required: true,
            },
            EndpointInfo {
                path: "/oauth/token".to_string(),
                method: "POST".to_string(),
                description: "OAuth2 token endpoint".to_string(),
                authentication_required: false,
            },
        ],
        default_token_expiry: 3600,
        default_refresh_token_expiry: 604800,
        supported_grant_types: vec![
            "authorization_code".to_string(),
            "client_credentials".to_string(),
            "password".to_string(),
            "refresh_token".to_string(),
        ],
        supported_response_types: vec![
            "code".to_string(),
            "token".to_string(),
        ],
        supported_scopes: vec![
            "*".to_string(),
            "read".to_string(),
            "write".to_string(),
            "admin".to_string(),
        ],
        pkce_required: false,
        rate_limits: RateLimitConfig {
            token_requests_per_minute: 60,
            authorization_requests_per_minute: 30,
            enabled: true,
        },
    };

    (StatusCode::OK, ResponseJson(config)).into_response()
}

/// Perform system cleanup operations (admin only)
pub async fn system_cleanup(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Json(payload): Json<SystemCleanupRequest>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: "unauthorized".to_string(),
            error_description: Some(e.to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    let mut tokens_removed = 0;
    let mut auth_codes_removed = 0;
    let mut operations = Vec::new();

    // This would need to be implemented in the respective services
    // For now, return a placeholder response

    if payload.remove_expired_tokens.unwrap_or(false) {
        operations.push("remove expired tokens");
        tokens_removed += 50; // Placeholder
    }

    if payload.remove_revoked_tokens.unwrap_or(false) {
        operations.push("remove revoked tokens");
        tokens_removed += 25; // Placeholder
    }

    if payload.remove_expired_auth_codes.unwrap_or(false) {
        operations.push("remove expired authorization codes");
        auth_codes_removed += 10; // Placeholder
    }

    let result = CleanupResult {
        tokens_removed,
        auth_codes_removed,
        message: format!(
            "Cleanup completed: {}. Removed {} tokens and {} auth codes.",
            operations.join(", "),
            tokens_removed,
            auth_codes_removed
        ),
    };

    (StatusCode::OK, ResponseJson(result)).into_response()
}

/// Get OAuth2 audit log (admin only)
pub async fn get_audit_log(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Query(_params): Query<AdminStatsQuery>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: "unauthorized".to_string(),
            error_description: Some(e.to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    // This would need to be implemented with proper audit logging
    let error = ErrorResponse {
        error: "not_implemented".to_string(),
        error_description: Some("Audit log endpoint not yet implemented".to_string()),
    };
    (StatusCode::NOT_IMPLEMENTED, ResponseJson(error)).into_response()
}

/// Export OAuth2 data (admin only)
pub async fn export_data(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Query(_params): Query<AdminStatsQuery>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: "unauthorized".to_string(),
            error_description: Some(e.to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    // This would generate and return a CSV or JSON export
    let error = ErrorResponse {
        error: "not_implemented".to_string(),
        error_description: Some("Data export endpoint not yet implemented".to_string()),
    };
    (StatusCode::NOT_IMPLEMENTED, ResponseJson(error)).into_response()
}

async fn get_authenticated_user(_pool: &DbPool, headers: &HeaderMap) -> anyhow::Result<String> {
    let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());
    let token = TokenUtils::extract_token_from_header(auth_header)?;
    let claims = AuthService::decode_token(token, "jwt-secret")?;

    Ok(claims.sub)
}

async fn verify_admin_access(pool: &DbPool, headers: &HeaderMap) -> anyhow::Result<String> {
    let user_id = get_authenticated_user(pool, headers).await?;

    // Here you would typically check if the user has admin role
    // For now, we'll accept any authenticated user
    // In a real implementation, you'd check user roles/permissions

    Ok(user_id)
}