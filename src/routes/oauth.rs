use axum::{
    routing::{get, post, put, delete},
    Router,
};
use crate::database::DbPool;

use crate::app::http::controllers::oauth::{
    oauth_controller, client_controller, personal_access_token_controller,
    scope_controller, authorization_controller, token_controller, admin_controller,
    device_controller, par_controller, token_exchange_controller,
    ciba_controller, mtls_controller
};

/// OAuth2/Passport routes
pub fn oauth_routes() -> Router<DbPool> {
    tracing::debug!("Creating OAuth2/Passport routes...");
    let router = Router::new()
        // OAuth2 Core Endpoints
        .route("/oauth/authorize", get(oauth_controller::authorize))
        .route("/oauth/token", post(oauth_controller::token))
        .route("/oauth/introspect", post(oauth_controller::introspect))
        .route("/oauth/revoke", post(oauth_controller::revoke))

        // Client Management API (requires authentication)
        .route("/oauth/clients", post(client_controller::create_client))
        .route("/oauth/clients", get(client_controller::list_clients))
        .route("/oauth/clients/{id}", get(client_controller::get_client))
        .route("/oauth/clients/{id}", put(client_controller::update_client))
        .route("/oauth/clients/{id}", delete(client_controller::delete_client))
        .route("/oauth/clients/{id}/regenerate-secret", post(client_controller::regenerate_secret))

        // Personal Access Tokens API (requires authentication)
        .route("/oauth/personal-access-tokens", post(personal_access_token_controller::create_personal_access_token))
        .route("/oauth/personal-access-tokens", get(personal_access_token_controller::list_personal_access_tokens))
        .route("/oauth/personal-access-tokens/{id}", delete(personal_access_token_controller::revoke_personal_access_token))

        // Scope Management API (admin required for create/update/delete)
        .route("/oauth/scopes", post(scope_controller::create_scope))
        .route("/oauth/scopes", get(scope_controller::list_scopes))
        .route("/oauth/scopes/{id}", get(scope_controller::get_scope))
        .route("/oauth/scopes/{id}", put(scope_controller::update_scope))
        .route("/oauth/scopes/{id}", delete(scope_controller::delete_scope))
        .route("/oauth/scopes/name/{name}", get(scope_controller::get_scope_by_name))
        .route("/oauth/scopes/validate", post(scope_controller::validate_scopes))

        // Authorization Management API
        .route("/oauth/auth-codes", post(authorization_controller::create_auth_code))
        .route("/oauth/auth-codes", get(authorization_controller::list_auth_codes))
        .route("/oauth/auth-codes/{id}", get(authorization_controller::get_auth_code))
        .route("/oauth/auth-codes/{id}/revoke", post(authorization_controller::revoke_auth_code))
        .route("/oauth/authorized-clients", get(authorization_controller::list_authorized_clients))
        .route("/oauth/authorized-clients/{client_id}/revoke", post(authorization_controller::revoke_client_authorization))

        // Token Management API
        .route("/oauth/tokens", get(token_controller::list_tokens))
        .route("/oauth/tokens/{id}", get(token_controller::get_token))
        .route("/oauth/tokens/{id}", delete(token_controller::revoke_token))
        .route("/oauth/tokens/{id}/extend", post(token_controller::extend_token))
        .route("/oauth/tokens/revoke", post(token_controller::revoke_tokens))
        .route("/oauth/tokens/stats", get(token_controller::get_token_stats))
        .route("/oauth/tokens/me", get(token_controller::get_my_tokens))

        // Admin Dashboard API (admin only)
        .route("/oauth/admin/dashboard", get(admin_controller::get_dashboard_stats))
        .route("/oauth/admin/config", get(admin_controller::get_system_config))
        .route("/oauth/admin/cleanup", post(admin_controller::system_cleanup))
        .route("/oauth/admin/audit", get(admin_controller::get_audit_log))
        .route("/oauth/admin/export", get(admin_controller::export_data))

        // RFC 8628: Device Authorization Grant endpoints
        .route("/oauth/device/code", post(device_controller::device_authorize))
        .route("/oauth/device/verify", get(device_controller::device_verification_page))
        .route("/oauth/device/authorize", post(device_controller::device_verify))

        // RFC 9126: Pushed Authorization Requests endpoints
        .route("/oauth/par", post(par_controller::create_pushed_request))
        .route("/oauth/par/authorize", get(par_controller::create_authorization_url))
        .route("/oauth/par/required/{client_id}", get(par_controller::check_par_requirement))
        .route("/oauth/par/cleanup", post(par_controller::cleanup_expired_requests))

        // RFC 8693: Token Exchange endpoints
        .route("/oauth/token-exchange", post(token_exchange_controller::exchange_token))
        .route("/oauth/token-exchange/supported-types", get(token_exchange_controller::get_supported_token_types))
        .route("/oauth/token-exchange/validate", post(token_exchange_controller::validate_exchange_request))

        // RFC 8955: CIBA endpoints
        .route("/oauth/ciba/auth", post(ciba_controller::create_backchannel_auth_request))
        .route("/oauth/ciba/complete/{auth_req_id}", post(ciba_controller::complete_user_authentication))
        .route("/oauth/ciba/status/{auth_req_id}", get(ciba_controller::get_auth_request_status))
        .route("/oauth/ciba/cleanup", post(ciba_controller::cleanup_expired_requests))

        // RFC 8705: mTLS endpoints
        .route("/oauth/mtls/validate", post(mtls_controller::validate_client_certificate))
        .route("/oauth/mtls/certificate-info", get(mtls_controller::get_certificate_info))
        .route("/oauth/mtls/validate-bound-token", post(mtls_controller::validate_certificate_bound_token))
        .route("/oauth/mtls/create-bound-claims", post(mtls_controller::create_certificate_bound_claims))
        .route("/oauth/mtls/validate-endpoint", post(mtls_controller::validate_mtls_endpoint));

    tracing::info!("OAuth2/Passport routes created successfully with {} endpoints", 56);
    router
}