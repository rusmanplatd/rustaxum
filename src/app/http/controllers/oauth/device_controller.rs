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

    // Render HTML page (simplified - in real app you'd use a template engine)
    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Device Authorization</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        .error {{
            color: #d32f2f;
            background: #ffebee;
            padding: 12px;
            border-radius: 4px;
            margin-bottom: 20px;
            border-left: 4px solid #d32f2f;
        }}
        .success {{
            color: #2e7d32;
            background: #e8f5e8;
            padding: 12px;
            border-radius: 4px;
            margin-bottom: 20px;
            border-left: 4px solid #2e7d32;
        }}
        .form-group {{
            margin-bottom: 20px;
        }}
        label {{
            display: block;
            margin-bottom: 8px;
            font-weight: bold;
        }}
        input[type="text"] {{
            width: 100%;
            padding: 12px;
            border: 1px solid #ddd;
            border-radius: 4px;
            font-size: 16px;
            text-transform: uppercase;
            letter-spacing: 2px;
            text-align: center;
        }}
        button {{
            background: #1976d2;
            color: white;
            padding: 12px 24px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 16px;
            width: 100%;
        }}
        button:hover {{
            background: #1565c0;
        }}
        .client-info {{
            background: #f8f9fa;
            padding: 15px;
            border-radius: 4px;
            margin-bottom: 20px;
        }}
        .scope-list {{
            list-style: none;
            padding: 0;
        }}
        .scope-list li {{
            background: #e3f2fd;
            padding: 8px 12px;
            margin: 4px 0;
            border-radius: 4px;
            border-left: 3px solid #1976d2;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🔐 Device Authorization</h1>
        <p>Enter the code displayed on your device to authorize access.</p>

        {}

        {}

        <form method="post" action="/oauth/device/verify">
            <div class="form-group">
                <label for="user_code">Device Code:</label>
                <input type="text"
                       id="user_code"
                       name="user_code"
                       placeholder="ABCD-EFGH"
                       value="{}"
                       pattern="[A-Z0-9]{{4}}-[A-Z0-9]{{4}}"
                       maxlength="9"
                       required
                       autofocus>
                <small style="color: #666; display: block; margin-top: 5px;">
                    Enter the 8-character code shown on your device (format: ABCD-EFGH)
                </small>
            </div>
            <button type="submit">Authorize Device</button>
        </form>

        <div style="margin-top: 30px; padding-top: 20px; border-top: 1px solid #eee; font-size: 14px; color: #666;">
            <p><strong>Having trouble?</strong></p>
            <ul>
                <li>Make sure you entered the code exactly as shown on your device</li>
                <li>The code is case-insensitive but must include the hyphen</li>
                <li>Device codes expire after 30 minutes</li>
            </ul>
        </div>
    </div>

    <script>
        // Auto-format user code input
        document.getElementById('user_code').addEventListener('input', function(e) {{
            let value = e.target.value.replace(/[^A-Z0-9]/g, '').toUpperCase();
            if (value.length > 4) {{
                value = value.substring(0, 4) + '-' + value.substring(4, 8);
            }}
            e.target.value = value;
        }});
    </script>
</body>
</html>"#,
        // Error message
        if let Some(error) = page_data.error {
            format!(r#"<div class="error">❌ {}</div>"#, html_escape(&error))
        } else {
            String::new()
        },

        // Client info
        if let Some(client_name) = page_data.client_name {
            format!(
                r#"<div class="client_info">
                    <h3>📱 Application: {}</h3>
                    {}
                </div>"#,
                html_escape(&client_name),
                if !page_data.scopes.is_empty() {
                    format!(
                        "<p><strong>Permissions requested:</strong></p><ul class=\"scope-list\">{}</ul>",
                        page_data.scopes.iter()
                            .map(|scope| format!("<li>🔑 {}</li>", html_escape(scope)))
                            .collect::<Vec<_>>()
                            .join("")
                    )
                } else {
                    "<p><em>No specific permissions requested</em></p>".to_string()
                }
            )
        } else {
            String::new()
        },

        // Pre-filled user code
        page_data.user_code.as_deref().unwrap_or("")
    );

    Html(html_content)
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
            // Return success HTML page
            Html(format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>Device Authorization Success</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body {{ font-family: Arial, sans-serif; text-align: center; padding: 50px; }}
        .success {{ color: green; }}
        .container {{ max-width: 500px; margin: 0 auto; }}
    </style>
</head>
<body>
    <div class="container">
        <h1 class="success">Device Authorization Successful</h1>
        <p>Your device with code <strong>{}</strong> has been successfully authorized.</p>
        <p>You may now return to your device and continue using the application.</p>
    </div>
</body>
</html>"#,
                verification.user_code
            )).into_response()
        },
        Err(e) => {
            tracing::warn!("Device verification failed: {}", e);
            (StatusCode::BAD_REQUEST, Html(format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>Device Authorization Failed</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body {{ font-family: Arial, sans-serif; text-align: center; padding: 50px; }}
        .error {{ color: red; }}
        .container {{ max-width: 500px; margin: 0 auto; }}
    </style>
</head>
<body>
    <div class="container">
        <h1 class="error">Device Authorization Failed</h1>
        <p>The device code <strong>{}</strong> could not be verified.</p>
        <p>Error: {}</p>
        <p>Please check the code and try again.</p>
    </div>
</body>
</html>"#,
                verification.user_code, e
            ))).into_response()
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

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}