use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::PgPool;

use crate::app::controllers::{auth_controller, user_controller, country_controller, province_controller, city_controller};

pub fn routes() -> Router<PgPool> {
    Router::new()
        // Authentication routes
        .route("/api/auth/register", post(auth_controller::register))
        .route("/api/auth/login", post(auth_controller::login))
        .route("/api/auth/forgot-password", post(auth_controller::forgot_password))
        .route("/api/auth/reset-password", post(auth_controller::reset_password))
        .route("/api/auth/change-password", put(auth_controller::change_password))
        .route("/api/auth/refresh-token", post(auth_controller::refresh_token))
        .route("/api/auth/logout", post(auth_controller::logout))
        .route("/api/auth/revoke-token", delete(auth_controller::revoke_token))
        .route("/api/auth/revoke-all-tokens", delete(auth_controller::revoke_all_tokens))
        // User routes
        .route("/api/users", get(user_controller::index))
        .route("/api/users/{id}", get(user_controller::show))
        // Country routes
        .route("/api/countries", get(country_controller::index))
        .route("/api/countries", post(country_controller::store))
        .route("/api/countries/{id}", get(country_controller::show))
        .route("/api/countries/{id}", put(country_controller::update))
        .route("/api/countries/{id}", delete(country_controller::destroy))
        // Province routes
        .route("/api/provinces", get(province_controller::index))
        .route("/api/provinces", post(province_controller::store))
        .route("/api/provinces/{id}", get(province_controller::show))
        .route("/api/provinces/{id}", put(province_controller::update))
        .route("/api/provinces/{id}", delete(province_controller::destroy))
        .route("/api/countries/{country_id}/provinces", get(province_controller::by_country))
        // City routes
        .route("/api/cities", get(city_controller::index))
        .route("/api/cities", post(city_controller::store))
        .route("/api/cities/{id}", get(city_controller::show))
        .route("/api/cities/{id}", put(city_controller::update))
        .route("/api/cities/{id}", delete(city_controller::destroy))
        .route("/api/provinces/{province_id}/cities", get(city_controller::by_province))
        .route("/api/cities/nearby", get(city_controller::nearby))
}