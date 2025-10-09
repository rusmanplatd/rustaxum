use axum::{
    routing::{get, post, put, delete},
    Router,
    middleware,
};
use crate::database::DbPool;
use crate::app::http::middleware::auth_guard::auth_guard;

use crate::app::http::controllers::{auth_controller, user_controller, country_controller, province_controller, city_controller, district_controller, village_controller, role_controller, permission_controller, docs_controller, organization_domain_controller, organization_type_controller, user_organization_controller, organization_position_level_controller, organization_position_controller, sys_model_has_permission_controller, sys_model_has_role_controller, activity_log_controller, session_controller};
use crate::app::http::controllers::web_push_controller::WebPushController;

pub fn routes() -> Router<DbPool> {
    tracing::debug!("Creating API routes...");

    // Public authentication routes (no middleware needed for these)
    let auth_routes = Router::new()
        // JWT-based authentication
        .route("/api/auth/register", post(auth_controller::register))
        .route("/api/auth/login", post(auth_controller::login))
        .route("/api/auth/mfa-login", post(auth_controller::complete_mfa_login))
        .route("/api/auth/forgot-password", post(auth_controller::forgot_password))
        .route("/api/auth/reset-password", post(auth_controller::reset_password))
        .route("/api/auth/refresh-token", post(auth_controller::refresh_token))
        // Session-based authentication
        .route("/api/auth/session/register", post(auth_controller::register_session))
        .route("/api/auth/session/login", post(auth_controller::login_session));

    // Protected authentication routes (require auth)
    let protected_auth_routes = Router::new()
        .route("/api/auth/change-password", put(auth_controller::change_password))
        .route("/api/auth/session/logout", post(auth_controller::logout_session))
        .route("/api/auth/session/user", get(auth_controller::user_session))
        .route("/api/me", get(auth_controller::me))
        .route_layer(middleware::from_fn(auth_guard));

    // Protected routes (require authentication)
    let protected_routes = Router::new()
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
        // District routes
        .route("/api/districts", get(district_controller::index))
        .route("/api/districts", post(district_controller::store))
        .route("/api/districts/{id}", get(district_controller::show))
        .route("/api/districts/{id}", put(district_controller::update))
        .route("/api/districts/{id}", delete(district_controller::destroy))
        // Village routes
        .route("/api/villages", get(village_controller::index))
        .route("/api/villages", post(village_controller::store))
        .route("/api/villages/{id}", get(village_controller::show))
        .route("/api/villages/{id}", put(village_controller::update))
        .route("/api/villages/{id}", delete(village_controller::destroy))
        // RBAC Role routes
        .route("/api/roles", get(role_controller::index))
        .route("/api/roles", post(role_controller::store))
        .route("/api/roles/{id}", get(role_controller::show))
        .route("/api/roles/{id}", put(role_controller::update))
        .route("/api/roles/{id}", delete(role_controller::destroy))
        .route("/api/roles/{id}/assign", post(role_controller::assign_to_user))
        .route("/api/roles/{id}/users/{user_id}", delete(role_controller::remove_from_user))
        .route("/api/users/{user_id}/roles", get(role_controller::get_user_roles))
        // RBAC Permission routes
        .route("/api/permissions", get(permission_controller::index))
        .route("/api/permissions", post(permission_controller::store))
        .route("/api/permissions/{id}", get(permission_controller::show))
        .route("/api/permissions/{id}", put(permission_controller::update))
        .route("/api/permissions/{id}", delete(permission_controller::destroy))
        .route("/api/permissions/{id}/assign", post(permission_controller::assign_to_role))
        .route("/api/permissions/{id}/roles/{role_id}", delete(permission_controller::remove_from_role))
        .route("/api/roles/{role_id}/permissions", get(permission_controller::get_role_permissions))
        .route("/api/users/{user_id}/permissions", get(permission_controller::get_user_permissions))
        // User Organization routes
        .route("/api/user-organizations", get(user_organization_controller::index))
        .route("/api/user-organizations", post(user_organization_controller::store))
        .route("/api/user-organizations/{id}", get(user_organization_controller::show))
        .route("/api/user-organizations/{id}", put(user_organization_controller::update))
        .route("/api/user-organizations/{id}", delete(user_organization_controller::destroy))
        .route("/api/user-organizations/{id}/transfer", post(user_organization_controller::transfer))
        .route("/api/user-organizations/{id}/activate", post(user_organization_controller::activate))
        .route("/api/user-organizations/{id}/deactivate", post(user_organization_controller::deactivate))
        // Organization Domain routes
        .route("/api/organization-domains", get(organization_domain_controller::index))
        .route("/api/organization-domains", post(organization_domain_controller::store))
        .route("/api/organization-domains/{id}", get(organization_domain_controller::show))
        .route("/api/organization-domains/{id}", put(organization_domain_controller::update))
        .route("/api/organization-domains/{id}", delete(organization_domain_controller::destroy))
        // Organization Type routes
        .route("/api/organization-types", get(organization_type_controller::index))
        .route("/api/organization-types", post(organization_type_controller::store))
        .route("/api/organization-types/{id}", get(organization_type_controller::show))
        .route("/api/organization-types/{id}", put(organization_type_controller::update))
        .route("/api/organization-types/{id}", delete(organization_type_controller::destroy))
        // Job Level routes
        .route("/api/organization-position-levels", get(organization_position_level_controller::index))
        .route("/api/organization-position-levels", post(organization_position_level_controller::store))
        .route("/api/organization-position-levels/{id}", get(organization_position_level_controller::show))
        .route("/api/organization-position-levels/{id}", put(organization_position_level_controller::update))
        .route("/api/organization-position-levels/{id}", delete(organization_position_level_controller::destroy))
        .route("/api/organization-position-levels/{id}/activate", post(organization_position_level_controller::activate))
        .route("/api/organization-position-levels/{id}/deactivate", post(organization_position_level_controller::deactivate))
        // Job Position routes
        .route("/api/organization-positions", get(organization_position_controller::index))
        .route("/api/organization-positions", post(organization_position_controller::store))
        .route("/api/organization-positions/{id}", get(organization_position_controller::show))
        .route("/api/organization-positions/{id}", put(organization_position_controller::update))
        .route("/api/organization-positions/{id}", delete(organization_position_controller::destroy))
        .route("/api/organization-positions/{id}/activate", post(organization_position_controller::activate))
        .route("/api/organization-positions/{id}/deactivate", post(organization_position_controller::deactivate))
        .route("/api/organization-levels/{organization_position_level_id}/positions", get(organization_position_controller::by_level))
        // Web Push routes
        .route("/api/web-push/vapid-public-key", get(WebPushController::get_vapid_public_key))
        .route("/api/web-push/subscribe", post(WebPushController::subscribe))
        .route("/api/web-push/unsubscribe", delete(WebPushController::unsubscribe))
        .route("/api/web-push/subscriptions", get(WebPushController::get_subscriptions))
        .route("/api/web-push/test", post(WebPushController::send_test_notification))
        .route("/api/web-push/status", get(WebPushController::get_status))
        .route("/api/web-push/cleanup", post(WebPushController::cleanup_subscriptions))
        // Sys Model Has Permission routes
        .route("/api/sys-model-has-permissions", get(sys_model_has_permission_controller::index))
        .route("/api/sys-model-has-permissions", post(sys_model_has_permission_controller::store))
        .route("/api/sys-model-has-permissions/{id}", get(sys_model_has_permission_controller::show))
        .route("/api/sys-model-has-permissions/{id}", put(sys_model_has_permission_controller::update))
        .route("/api/sys-model-has-permissions/{id}", delete(sys_model_has_permission_controller::destroy))
        .route("/api/models/{model_type}/{model_id}/permissions", get(sys_model_has_permission_controller::by_model))
        // Sys Model Has Role routes
        .route("/api/sys-model-has-roles", get(sys_model_has_role_controller::index))
        .route("/api/sys-model-has-roles", post(sys_model_has_role_controller::store))
        .route("/api/sys-model-has-roles/{id}", get(sys_model_has_role_controller::show))
        .route("/api/sys-model-has-roles/{id}", put(sys_model_has_role_controller::update))
        .route("/api/sys-model-has-roles/{id}", delete(sys_model_has_role_controller::destroy))
        .route("/api/models/{model_type}/{model_id}/roles", get(sys_model_has_role_controller::by_model))
        // Activity Log routes
        .route("/api/activity-logs", get(activity_log_controller::list_activity_logs))
        .route("/api/activity-logs", post(activity_log_controller::create_activity_log))
        .route("/api/activity-logs/stats", get(activity_log_controller::get_activity_stats))
        .route("/api/activity-logs/{id}", get(activity_log_controller::get_activity_log))
        .route("/api/activity-logs/correlation/{correlation_id}", get(activity_log_controller::get_activities_by_correlation))
        .route("/api/activity-logs/batch/{batch_uuid}", get(activity_log_controller::get_activities_by_batch))
        .route("/api/activity-logs/subject/{subject_type}/{subject_id}", get(activity_log_controller::get_activities_by_subject))
        .route("/api/activity-logs/causer/{causer_type}/{causer_id}", get(activity_log_controller::get_activities_by_causer))
        // Session routes
        .route("/api/session", get(session_controller::get_session))
        .route("/api/session", post(session_controller::put_session))
        .route("/api/session/{key}", get(session_controller::get_session_value))
        .route("/api/session/{key}", delete(session_controller::forget_session_value))
        .route("/api/session/flush", post(session_controller::flush_session))
        .route("/api/session/regenerate", post(session_controller::regenerate_session))
        .route("/api/session/flash", post(session_controller::flash_session))
        .route("/api/session/token", post(session_controller::regenerate_token))
        .route_layer(middleware::from_fn(auth_guard));

    // Public routes (no authentication required)
    let public_routes = Router::new()
        // Documentation routes
        .route("/api/docs", get(docs_controller::docs_info))
        .route("/api/docs/openapi.json", get(docs_controller::openapi_json))
        .route("/api/docs/openapi.yaml", get(docs_controller::openapi_yaml));

    // Combine all routes
    let router = Router::new()
        .merge(auth_routes)
        .merge(protected_auth_routes)
        .merge(protected_routes)
        .merge(public_routes);

    tracing::info!("API routes created successfully with authentication middleware applied");
    router
}