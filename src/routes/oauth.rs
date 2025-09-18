use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::PgPool;

use crate::app::controllers::oauth::{
    oauth_controller, client_controller, personal_access_token_controller
};

/// OAuth2/Passport routes
pub fn oauth_routes() -> Router<PgPool> {
    Router::new()
        // OAuth2 Core Endpoints
        .route("/oauth/authorize", get(oauth_controller::authorize))
        .route("/oauth/token", post(oauth_controller::token))
        .route("/oauth/introspect", post(oauth_controller::introspect))
        .route("/oauth/revoke", post(oauth_controller::revoke))

        // Client Management API (requires authentication)
        .route("/oauth/clients", post(client_controller::create_client))
        .route("/oauth/clients", get(client_controller::list_clients))
        .route("/oauth/clients/:id", get(client_controller::get_client))
        .route("/oauth/clients/:id", put(client_controller::update_client))
        .route("/oauth/clients/:id", delete(client_controller::delete_client))
        .route("/oauth/clients/:id/regenerate-secret", post(client_controller::regenerate_secret))

        // Personal Access Tokens API (requires authentication)
        .route("/oauth/personal-access-tokens", post(personal_access_token_controller::create_personal_access_token))
        .route("/oauth/personal-access-tokens", get(personal_access_token_controller::list_personal_access_tokens))
        .route("/oauth/personal-access-tokens/:id", delete(personal_access_token_controller::revoke_personal_access_token))
}