use axum::{
    extract::{Query, Json, State, Form},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json as ResponseJson, Html},
};
use serde::{Deserialize, Serialize};
use crate::database::DbPool;
use utoipa::ToSchema;
use std::collections::HashMap;

use crate::app::services::oauth::{DeviceService, ClientService};
use crate::app::models::oauth::{CreateDeviceCode, DeviceAuthorizationResponse, DeviceCodeVerification};
use crate::app::utils::token_utils::TokenUtils;
use crate::app::services::auth_service::AuthService;
use crate::app::services::template_service::TemplateService;
use serde_json::json;

#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
    error_description: Option<String>,
}

#[derive(Deserialize, ToSchema)]
struct DeviceTokenRequest {
    grant_type: String,
    device_code: String,
    client_id: String,
}

#[derive(Serialize, ToSchema)]
struct DeviceVerificationPageData {
    user_code: Option<String>,
    client_name: Option<String>,
    scopes: Vec<String>,
    error: Option<String>,
}

/// RFC 8628: Device Authorization Endpoint
/// Initiates the device authorization flow
#[utoipa::path(
    post,
    path = "/oauth/device/authorize",
    tags = ["Device Authorization (RFC 8628)"],
    summary = "Device authorization request",
    description = "RFC 8628 Device Authorization Grant - Initiates device authorization flow for input-constrained devices",
    request_body = CreateDeviceCode,
    responses(
        (status = 200, description = "Device authorization initiated", body = DeviceAuthorizationResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Invalid client", body = ErrorResponse)
    )
)]
pub async fn device_authorize(
    State(pool): State<DbPool>,
    Json(request): Json<CreateDeviceCode>,
) -> impl IntoResponse {
    // Validate client exists
    let client = match ClientService::find_by_id(&pool, request.client_id.clone()) {
        Ok(Some(client)) => client,
        Ok(None) => {
            return (StatusCode::UNAUTHORIZED, ResponseJson(ErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Client not found".to_string()),
            })).into_response();
        },
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(ErrorResponse {
                error: "server_error".to_string(),
                error_description: Some("Internal server error".to_string()),
            })).into_response();
        }
    };

    if client.revoked {
        return (StatusCode::UNAUTHORIZED, ResponseJson(ErrorResponse {
            error: "invalid_client".to_string(),
            error_description: Some("Client has been revoked".to_string()),
        })).into_response();
    }

    // Create device authorization
    match DeviceService::create_device_authorization(&pool, request).await {
        Ok(device_response) => {
            tracing::info!("Device authorization created: user_code={}", device_response.user_code);
            (StatusCode::OK, ResponseJson(device_response)).into_response()
        },
        Err(e) => {
            tracing::error!("Device authorization failed: {}", e);
            (StatusCode::BAD_REQUEST, ResponseJson(ErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some(e.to_string()),
            })).into_response()
        }
    }
}

/// RFC 8628: Device Token Endpoint (Polling)
/// Used by devices to poll for access tokens after user authorization
#[utoipa::path(
    post,
    path = "/oauth/device/token",
    tags = ["Device Authorization (RFC 8628)"],
    summary = "Device token polling",
    description = "RFC 8628 Device Authorization Grant - Poll for access token after user authorization. Returns 'authorization_pending' until user authorizes.",
    request_body = DeviceTokenRequest,
    responses(
        (status = 200, description = "Access token granted", body = crate::app::docs::oauth::TokenResponse),
        (status = 400, description = "Authorization pending or expired", body = ErrorResponse),
        (status = 428, description = "Slow down - polling too frequently", body = ErrorResponse)
    )
)]
pub async fn device_token(
    State(pool): State<DbPool>,
    Form(request): Form<DeviceTokenRequest>,
) -> impl IntoResponse {
    // Validate grant type
    if request.grant_type != "urn:ietf:params:oauth:grant-type:device_code" {
        return (StatusCode::BAD_REQUEST, ResponseJson(ErrorResponse {
            error: "unsupported_grant_type".to_string(),
            error_description: Some("Only device_code grant type is supported".to_string()),
        })).into_response();
    }

    // Poll for device token
    match DeviceService::poll_device_token(&pool, request.device_code, request.client_id).await {
        Ok(token_response) => {
            tracing::info!("Device token granted successfully");
            (StatusCode::OK, ResponseJson(token_response)).into_response()
        },
        Err(e) => {
            let error_msg = e.to_string();

            let (error_code, description) = if error_msg.contains("Authorization pending") {
                ("authorization_pending", "The authorization request is still pending as the user hasn't yet completed the user interaction steps")
            } else if error_msg.contains("expired") {
                ("expired_token", "The device_code has expired and the device authorization session has concluded")
            } else if error_msg.contains("denied") {
                ("access_denied", "The user denied the authorization request")
            } else if error_msg.contains("Invalid device code") {
                ("invalid_grant", "The device_code is invalid")
            } else {
                ("invalid_request", "Invalid device authorization request")
            };

            // RFC 8628 specifies specific status codes for different errors
            let status_code = match error_code {
                "authorization_pending" => StatusCode::BAD_REQUEST, // 400 - keep polling
                "slow_down" => StatusCode::TOO_MANY_REQUESTS, // 428 - slow down polling
                "expired_token" | "access_denied" => StatusCode::BAD_REQUEST, // 400 - stop polling
                _ => StatusCode::BAD_REQUEST, // 400 - general error
            };

            tracing::warn!("Device token polling error: {} - {}", error_code, description);

            (status_code, ResponseJson(ErrorResponse {
                error: error_code.to_string(),
                error_description: Some(description.to_string()),
            })).into_response()
        }
    }
}

/// Device Verification Page (GET)
/// Shows the user code entry form
#[utoipa::path(
    get,
    path = "/oauth/device",
    tags = ["Device Authorization (RFC 8628)"],
    summary = "Device verification page",
    description = "User-facing page where users enter device codes to authorize devices",
    params(
        ("user_code" = Option<String>, Query, description = "Pre-filled user code from verification_uri_complete")
    ),
    responses(
        (status = 200, description = "Device verification page", content_type = "text/html")
    )
)]
pub async fn device_verification_page(
    State(pool): State<DbPool>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let user_code = params.get("user_code").cloned();
    let error = params.get("error").cloned();

    // If user_code is provided, try to get client info
    let (client_name, scopes) = if let Some(ref code) = user_code {
        match DeviceService::find_by_user_code(&pool, code) {
            Ok(Some(device_code)) => {
                if device_code.is_valid() {
                    let client = ClientService::find_by_id(&pool, device_code.client_id.clone())
                        .unwrap_or(None);

                    let client_name = client.map(|c| c.name);
                    let scopes = device_code.get_scopes();

                    (client_name, scopes)
                } else {
                    (None, Vec::new())
                }
            },
            _ => (None, Vec::new())
        }
    } else {
        (None, Vec::new())
    };

    let page_data = DeviceVerificationPageData {
        user_code,
        client_name,
        scopes,
        error,
    };

    let template_service = TemplateService::global();
    let context = json!({
        "user_code": page_data.user_code,
        "client_name": page_data.client_name,
        "scopes": page_data.scopes,
        "error": page_data.error
    });

    match template_service.render("oauth/device-verification", &context).await {
        Ok(html) => Html(html),
        Err(e) => {
            tracing::error!("Template rendering error: {}", e);
            Html("<h1>Template Error</h1><p>Unable to render device verification page</p>".to_string())
        }
    }
}

/// Device Verification (POST)
/// Processes user code submission and authorization
#[utoipa::path(
    post,
    path = "/oauth/device/verify",
    tags = ["Device Authorization (RFC 8628)"],
    summary = "Verify device code",
    description = "Process user code submission and authorize device if valid",
    request_body = DeviceCodeVerification,
    responses(
        (status = 302, description = "Redirect to success/error page"),
        (status = 200, description = "Authorization confirmation page", content_type = "text/html"),
        (status = 401, description = "Authentication required")
    )
)]
pub async fn device_verify(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Json(verification): Json<DeviceCodeVerification>,
) -> impl IntoResponse {
    // Extract authenticated user from JWT token
    let user_id = match crate::app::http::middleware::auth_middleware::validate_jwt_token(
        &crate::app::http::middleware::auth_middleware::extract_bearer_token(&headers)
            .unwrap_or_default()
    ) {
        Some(uid) => uid,
        None => {
            return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
                "error": "unauthorized",
                "error_description": "Valid authentication required to authorize device"
            }))).into_response();
        }
    };

    // Process the device verification
    match DeviceService::verify_device_code(&pool, &verification.user_code, &user_id).await {
        Ok(_) => {
            let template_service = TemplateService::global();
            let context = json!({
                "user_code": verification.user_code
            });

            match template_service.render("oauth/device-success", &context).await {
                Ok(html) => Html(html).into_response(),
                Err(e) => {
                    tracing::error!("Template rendering error: {}", e);
                    Html("<h1>Template Error</h1><p>Unable to render success page</p>".to_string()).into_response()
                }
            }
        },
        Err(e) => {
            tracing::warn!("Device verification failed: {}", e);
            let template_service = TemplateService::global();
            let context = json!({
                "user_code": verification.user_code,
                "error_message": e.to_string()
            });

            match template_service.render("oauth/device-error", &context).await {
                Ok(html) => (StatusCode::BAD_REQUEST, Html(html)).into_response(),
                Err(template_error) => {
                    tracing::error!("Template rendering error: {}", template_error);
                    (StatusCode::BAD_REQUEST, Html("<h1>Template Error</h1><p>Unable to render error page</p>".to_string())).into_response()
                }
            }
        }
    }
}

/// Admin endpoint to list active device codes
#[utoipa::path(
    get,
    path = "/oauth/device/admin/list",
    tags = ["Device Authorization Admin"],
    summary = "List active device codes",
    description = "Admin endpoint to list all active device authorization codes",
    security(
        ("Bearer" = ["admin"])
    ),
    responses(
        (status = 200, description = "List of active device codes"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn admin_list_device_codes(
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Check admin authentication
    if let Err(status) = crate::app::http::middleware::auth_middleware::verify_admin_access(&headers, &pool).await {
        return (status, ResponseJson(ErrorResponse {
            error: "access_denied".to_string(),
            error_description: Some("Admin privileges required".to_string()),
        })).into_response();
    }

    match DeviceService::list_active_codes(&pool, None) {
        Ok(device_codes) => {
            (StatusCode::OK, ResponseJson(device_codes)).into_response()
        },
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(ErrorResponse {
                error: "server_error".to_string(),
                error_description: Some(e.to_string()),
            })).into_response()
        }
    }
}

/// Admin endpoint to get device authorization statistics
#[utoipa::path(
    get,
    path = "/oauth/device/admin/stats",
    tags = ["Device Authorization Admin"],
    summary = "Device authorization statistics",
    description = "Admin endpoint to get device authorization flow statistics",
    security(
        ("Bearer" = ["admin"])
    ),
    responses(
        (status = 200, description = "Device authorization statistics"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn admin_device_stats(
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Check admin authentication
    if let Err(status) = crate::app::http::middleware::auth_middleware::verify_admin_access(&headers, &pool).await {
        return (status, ResponseJson(ErrorResponse {
            error: "access_denied".to_string(),
            error_description: Some("Admin privileges required".to_string()),
        })).into_response();
    }

    match DeviceService::get_device_stats(&pool) {
        Ok(stats) => {
            (StatusCode::OK, ResponseJson(stats)).into_response()
        },
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(ErrorResponse {
                error: "server_error".to_string(),
                error_description: Some(e.to_string()),
            })).into_response()
        }
    }
}

// Helper functions
async fn get_user_from_token(_pool: &DbPool, auth_header: Option<&str>) -> anyhow::Result<String> {
    let token = TokenUtils::extract_token_from_header(auth_header)?;
    let claims = AuthService::decode_token(token)?;

    let user_id = ulid::Ulid::from_string(&claims.sub)?;
    Ok(user_id.to_string())
}

