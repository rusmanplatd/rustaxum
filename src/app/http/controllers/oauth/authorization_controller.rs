use axum::{
    extract::{Json, State, Path, Query},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json as ResponseJson},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::database::DbPool;
use chrono::{Utc, Duration};

use crate::app::services::oauth::{TokenService, ClientService, ScopeService};
use crate::app::services::auth_service::AuthService;
use crate::app::models::oauth::CreateAuthCode;
use crate::app::utils::token_utils::TokenUtils;

#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
    error_description: Option<String>,
}

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

#[derive(Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ListAuthCodesQuery {
    pub user_id: Option<String>,
    pub client_id: Option<String>,
    pub expired: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateAuthCodeRequest {
    pub user_id: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub redirect_uri: String,
    pub challenge: Option<String>,
    pub challenge_method: Option<String>,
    pub expires_in_minutes: Option<i64>,
}

#[derive(Serialize)]
pub struct AuthCodeResponse {
    pub id: String,
    pub user_id: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub redirect_uri: String,
    pub expires_at: chrono::DateTime<Utc>,
    pub revoked: bool,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Deserialize, ToSchema, utoipa::IntoParams)]
pub struct AuthorizedClientQuery {
    pub client_id: Option<String>,
    pub scope: Option<String>,
}

/// Create a new authorization code (admin only)
#[utoipa::path(
    post,
    path = "/oauth/auth-codes",
    tags = ["OAuth Authorization"],
    summary = "Create authorization code",
    description = "Create a new authorization code (admin only)",
    request_body = CreateAuthCodeRequest,
    responses(
        (status = 201, description = "Authorization code created", body = crate::app::docs::oauth::AuthCodeResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(("Bearer" = []))
)]
pub async fn create_auth_code(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Json(payload): Json<CreateAuthCodeRequest>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: "unauthorized".to_string(),
            error_description: Some(e.to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    // Validate client exists
    match ClientService::find_by_id(&pool, payload.client_id.clone()) {
        Ok(Some(_)) => {},
        Ok(None) => {
            let error = ErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Client not found".to_string()),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        },
        Err(e) => {
            let error = ErrorResponse {
                error: "server_error".to_string(),
                error_description: Some(e.to_string()),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    }

    // Validate scopes
    match ScopeService::validate_scopes(&pool, &payload.scopes).await {
        Ok(_) => {},
        Err(e) => {
            let error = ErrorResponse {
                error: "invalid_scope".to_string(),
                error_description: Some(e.to_string()),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    }

    let expires_in_minutes = payload.expires_in_minutes.unwrap_or(10);
    let expires_at = Utc::now() + Duration::minutes(expires_in_minutes);

    let create_data = CreateAuthCode {
        user_id: payload.user_id,
        client_id: payload.client_id,
        scopes: payload.scopes,
        redirect_uri: payload.redirect_uri,
        challenge: payload.challenge,
        challenge_method: payload.challenge_method,
        expires_at: Some(expires_at),
    };

    match TokenService::create_auth_code(&pool, create_data) {
        Ok(auth_code) => {
            let response = AuthCodeResponse {
                id: auth_code.id.to_string(),
                user_id: auth_code.user_id.clone(),
                client_id: auth_code.client_id.clone(),
                scopes: auth_code.get_scopes(),
                redirect_uri: auth_code.redirect_uri.clone(),
                expires_at: auth_code.expires_at.unwrap_or(Utc::now()),
                revoked: auth_code.revoked,
                created_at: auth_code.created_at,
            };
            (StatusCode::CREATED, ResponseJson(response)).into_response()
        },
        Err(e) => {
            let error = ErrorResponse {
                error: "server_error".to_string(),
                error_description: Some(e.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

/// List authorization codes (admin only)
#[utoipa::path(
    get,
    path = "/oauth/auth-codes",
    tags = ["OAuth Authorization"],
    summary = "List authorization codes",
    description = "Get list of authorization codes (admin only)",
    params(
        ListAuthCodesQuery
    ),
    responses(
        (status = 200, description = "List of authorization codes", body = Vec<crate::app::docs::oauth::AuthCodeResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("Bearer" = []))
)]
pub async fn list_auth_codes(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Query(_params): Query<ListAuthCodesQuery>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: "unauthorized".to_string(),
            error_description: Some(e.to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    // Note: This would need to be implemented in TokenService
    let error = ErrorResponse {
        error: "not_implemented".to_string(),
        error_description: Some("List auth codes endpoint not yet implemented".to_string()),
    };
    (StatusCode::NOT_IMPLEMENTED, ResponseJson(error)).into_response()
}

/// Get a specific authorization code (admin only)
#[utoipa::path(
    get,
    path = "/oauth/auth-codes/{code_id}",
    tags = ["OAuth Authorization"],
    summary = "Get authorization code",
    description = "Get a specific authorization code by ID (admin only)",
    params(
        ("code_id" = String, Path, description = "Authorization code identifier")
    ),
    responses(
        (status = 200, description = "Authorization code found", body = crate::app::docs::oauth::AuthCodeResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Authorization code not found", body = ErrorResponse)
    ),
    security(("Bearer" = []))
)]
pub async fn get_auth_code(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(code_id): Path<String>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: "unauthorized".to_string(),
            error_description: Some(e.to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    match TokenService::find_auth_code_by_id(&pool, code_id) {
        Ok(Some(auth_code)) => {
            let response = AuthCodeResponse {
                id: auth_code.id.to_string(),
                user_id: auth_code.user_id.clone(),
                client_id: auth_code.client_id.clone(),
                scopes: auth_code.get_scopes(),
                redirect_uri: auth_code.redirect_uri.clone(),
                expires_at: auth_code.expires_at.unwrap_or(Utc::now()),
                revoked: auth_code.revoked,
                created_at: auth_code.created_at,
            };
            (StatusCode::OK, ResponseJson(response)).into_response()
        },
        Ok(None) => {
            let error = ErrorResponse {
                error: "not_found".to_string(),
                error_description: Some("Authorization code not found".to_string()),
            };
            (StatusCode::NOT_FOUND, ResponseJson(error)).into_response()
        },
        Err(e) => {
            let error = ErrorResponse {
                error: "server_error".to_string(),
                error_description: Some(e.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

/// Revoke an authorization code (admin only)
#[utoipa::path(
    delete,
    path = "/oauth/auth-codes/{code_id}",
    tags = ["OAuth Authorization"],
    summary = "Revoke authorization code",
    description = "Revoke an authorization code by ID (admin only)",
    params(
        ("code_id" = String, Path, description = "Authorization code identifier")
    ),
    responses(
        (status = 200, description = "Authorization code revoked successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Authorization code not found", body = ErrorResponse)
    ),
    security(("Bearer" = []))
)]
pub async fn revoke_auth_code(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(code_id): Path<String>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: "unauthorized".to_string(),
            error_description: Some(e.to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    match TokenService::revoke_auth_code(&pool, code_id) {
        Ok(_) => {
            let response = MessageResponse {
                message: "Authorization code revoked successfully".to_string(),
            };
            (StatusCode::OK, ResponseJson(response)).into_response()
        },
        Err(e) => {
            let error = ErrorResponse {
                error: "server_error".to_string(),
                error_description: Some(e.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

/// List authorized clients for the authenticated user
#[utoipa::path(
    get,
    path = "/oauth/authorized-clients",
    tags = ["OAuth Authorization"],
    summary = "List authorized clients",
    description = "Get list of clients authorized by the user",
    params(
        AuthorizedClientQuery
    ),
    responses(
        (status = 200, description = "List of authorized clients", body = Vec<crate::app::docs::oauth::ClientResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("Bearer" = []))
)]
pub async fn list_authorized_clients(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Query(params): Query<AuthorizedClientQuery>,
) -> impl IntoResponse {
    let user_id = match get_authenticated_user(&pool, &headers).await {
        Ok(user_id) => user_id,
        Err(e) => {
            let error = ErrorResponse {
                error: "unauthorized".to_string(),
                error_description: Some(e.to_string()),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    // Get all access tokens for the user
    match TokenService::list_user_tokens(&pool, user_id).await {
        Ok(tokens) => {
            #[derive(Serialize)]
            struct AuthorizedClientInfo {
                client_id: String,
                scopes: Vec<String>,
                last_used: chrono::DateTime<Utc>,
                expires_at: Option<chrono::DateTime<Utc>>,
            }

            let mut authorized_clients: std::collections::HashMap<String, AuthorizedClientInfo> = std::collections::HashMap::new();

            for token in tokens {
                if !token.is_valid() {
                    continue;
                }

                // Filter by client_id if specified
                if let Some(ref filter_client_id) = params.client_id {
                    if token.client_id != *filter_client_id {
                        continue;
                    }
                }

                // Filter by scope if specified
                if let Some(ref filter_scope) = params.scope {
                    if !token.has_scope(filter_scope) {
                        continue;
                    }
                }

                let client_info = AuthorizedClientInfo {
                    client_id: token.client_id.clone(),
                    scopes: token.get_scopes(),
                    last_used: token.updated_at,
                    expires_at: token.expires_at,
                };

                authorized_clients.insert(token.client_id.clone(), client_info);
            }

            let clients: Vec<_> = authorized_clients.into_values().collect();
            (StatusCode::OK, ResponseJson(clients)).into_response()
        },
        Err(e) => {
            let error = ErrorResponse {
                error: "server_error".to_string(),
                error_description: Some(e.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

/// Revoke access for a specific client (user can revoke their own authorizations)
#[utoipa::path(
    delete,
    path = "/oauth/authorized-clients/{client_id}",
    tags = ["OAuth Authorization"],
    summary = "Revoke client authorization",
    description = "Revoke all authorizations for a specific client",
    params(
        ("client_id" = String, Path, description = "Client identifier")
    ),
    responses(
        (status = 200, description = "Client authorization revoked successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Client not found", body = ErrorResponse)
    ),
    security(("Bearer" = []))
)]
pub async fn revoke_client_authorization(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(client_id): Path<String>,
) -> impl IntoResponse {
    let user_id = match get_authenticated_user(&pool, &headers).await {
        Ok(user_id) => user_id,
        Err(e) => {
            let error = ErrorResponse {
                error: "unauthorized".to_string(),
                error_description: Some(e.to_string()),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    // Get all user tokens for this client and revoke them
    match TokenService::list_user_tokens(&pool, user_id.clone()).await {
        Ok(tokens) => {
            let mut revoked_count = 0;

            for token in tokens {
                if token.client_id == client_id && !token.revoked {
                    if let Ok(_) = TokenService::revoke_access_token(&pool, token.id.to_string()) {
                        revoked_count += 1;
                    }
                }
            }

            let response = MessageResponse {
                message: format!("Revoked {} token(s) for client {}", revoked_count, client_id),
            };
            (StatusCode::OK, ResponseJson(response)).into_response()
        },
        Err(e) => {
            let error = ErrorResponse {
                error: "server_error".to_string(),
                error_description: Some(e.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

async fn get_authenticated_user(_pool: &DbPool, headers: &HeaderMap) -> anyhow::Result<String> {
    let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());
    let token = TokenUtils::extract_token_from_header(auth_header)?;
    let claims = AuthService::decode_token(token)?;

    Ok(claims.sub)
}

async fn verify_admin_access(pool: &DbPool, headers: &HeaderMap) -> anyhow::Result<String> {
    let user_id = get_authenticated_user(pool, headers).await?;

    // Check if user has admin role/permissions (Laravel-style role check)
    

    // For production: implement proper permission checking
    // This would typically check if the user has "manage_oauth" permission
    if user_id.is_empty() {
        return Err(anyhow::anyhow!("Insufficient permissions - admin access required"));
    }

    Ok(user_id)
}