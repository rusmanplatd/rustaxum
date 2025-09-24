use axum::{
    extract::{Query, Json, State, Form},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json as ResponseJson, Redirect},
};
use serde::{Deserialize, Serialize};
use crate::database::DbPool;
use utoipa::ToSchema;
use ulid::Ulid;
use chrono::{Duration, Utc};
use std::collections::HashMap;

use crate::app::services::oauth::{TokenService, ClientService, ScopeService}; // DPoPService - TODO: Re-enable when ready
use crate::app::services::auth_service::AuthService;
use crate::app::models::oauth::{CreateAuthCode};
use crate::app::utils::token_utils::TokenUtils;
use crate::app::models::DieselUlid;

#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
    error_description: Option<String>,
}

#[derive(Deserialize)]
pub struct AuthorizeQuery {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub code_verifier: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct IntrospectRequest {
    pub token: String,
    pub token_type_hint: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct IntrospectResponse {
    pub active: bool,
    pub scope: Option<String>,
    pub client_id: Option<String>,
    pub username: Option<String>,
    pub exp: Option<i64>,
    pub iat: Option<i64>,
    pub sub: Option<String>,
    pub aud: Option<String>,
}

#[utoipa::path(
    get,
    path = "/oauth/authorize",
    tags = ["OAuth Core"],
    summary = "OAuth2 authorization endpoint",
    description = "Initiate OAuth2 authorization code flow",
    params(
        ("response_type" = String, Query, description = "Response type (must be 'code')"),
        ("client_id" = String, Query, description = "Client identifier"),
        ("redirect_uri" = String, Query, description = "Redirect URI"),
        ("scope" = Option<String>, Query, description = "Requested scopes"),
        ("state" = Option<String>, Query, description = "State parameter"),
        ("code_challenge" = Option<String>, Query, description = "PKCE code challenge"),
        ("code_challenge_method" = Option<String>, Query, description = "PKCE challenge method")
    ),
    responses(
        (status = 302, description = "Redirect to authorization page or back to client"),
        (status = 400, description = "Invalid request", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn authorize(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Query(params): Query<AuthorizeQuery>,
) -> impl IntoResponse {
    // OAuth 2.1 Compliance: Only authorization code flow is supported (implicit removed)
    if params.response_type != "code" {
        let error_url = format!(
            "{}?error=unsupported_response_type&error_description=Only+authorization+code+flow+is+supported+-+OAuth+2.1+compliance{}",
            params.redirect_uri,
            params.state.as_ref().map(|s| format!("&state={}", s)).unwrap_or_default()
        );
        return Redirect::temporary(&error_url).into_response();
    }

    // OAuth 2.1 Compliance: PKCE is mandatory for all authorization code flows
    if params.code_challenge.is_none() {
        let error_url = format!(
            "{}?error=invalid_request&error_description=PKCE+code_challenge+is+required+for+OAuth+2.1+compliance{}",
            params.redirect_uri,
            params.state.as_ref().map(|s| format!("&state={}", s)).unwrap_or_default()
        );
        return Redirect::temporary(&error_url).into_response();
    }

    // OAuth 2.1 Compliance: Validate PKCE challenge method (S256 preferred, plain deprecated)
    if let Some(ref method) = params.code_challenge_method {
        if method != "S256" && method != "plain" {
            let error_url = format!(
                "{}?error=invalid_request&error_description=Invalid+code_challenge_method+-+must+be+S256+or+plain{}",
                params.redirect_uri,
                params.state.as_ref().map(|s| format!("&state={}", s)).unwrap_or_default()
            );
            return Redirect::temporary(&error_url).into_response();
        }
        // Warn about plain method (deprecated in OAuth 2.1)
        if method == "plain" {
            tracing::warn!("Client {} using deprecated plain PKCE method", params.client_id);
        }
    } else if params.code_challenge.is_some() {
        // Default to S256 if challenge provided but method is not
        tracing::info!("Defaulting to S256 PKCE method for client {}", params.client_id);
    }

    // Parse client ID
    let client_id = match Ulid::from_string(&params.client_id) {
        Ok(id) => id,
        Err(_) => {
            let error_url = format!(
                "{}?error=invalid_client&error_description=Invalid+client+ID{}",
                params.redirect_uri,
                params.state.as_ref().map(|s| format!("&state={}", s)).unwrap_or_default()
            );
            return Redirect::temporary(&error_url).into_response();
        }
    };

    // OAuth 2.1 Compliance: Exact string matching for redirect URIs (no wildcard matching)
    match ClientService::is_valid_redirect_uri(&pool, client_id.to_string(), &params.redirect_uri) {
        Ok(true) => {
            tracing::debug!("Redirect URI validated for client {}: {}", client_id, params.redirect_uri);
        },
        Ok(false) => {
            tracing::warn!("Invalid redirect URI for client {}: {}", client_id, params.redirect_uri);
            return (StatusCode::BAD_REQUEST, ResponseJson(ErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Invalid redirect URI - OAuth 2.1 requires exact string matching".to_string()),
            })).into_response();
        },
        Err(_) => {
            tracing::error!("Client not found: {}", client_id);
            return (StatusCode::BAD_REQUEST, ResponseJson(ErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Client not found".to_string()),
            })).into_response();
        }
    }

    // Get user from authorization header
    let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());
    let user_id = match get_user_from_token(&pool, auth_header).await {
        Ok(user_id) => user_id,
        Err(_) => {
            // Redirect to login with authorization request in query params
            let login_url = format!(
                "/login?redirect_uri={}&response_type={}&client_id={}&scope={}{}{}{}",
                urlencoding::encode(&params.redirect_uri),
                params.response_type,
                params.client_id,
                params.scope.as_deref().unwrap_or(""),
                params.state.as_ref().map(|s| format!("&state={}", s)).unwrap_or_default(),
                params.code_challenge.as_ref().map(|c| format!("&code_challenge={}", c)).unwrap_or_default(),
                params.code_challenge_method.as_ref().map(|m| format!("&code_challenge_method={}", m)).unwrap_or_default()
            );
            return Redirect::temporary(&login_url).into_response();
        }
    };

    // Validate user has access to this OAuth client's organization
    let user_id_ulid = match Ulid::from_string(&user_id) {
        Ok(id) => DieselUlid(id),
        Err(_) => {
            let error_url = format!(
                "{}?error=server_error&error_description=Invalid+user+ID{}",
                params.redirect_uri,
                params.state.as_ref().map(|s| format!("&state={}", s)).unwrap_or_default()
            );
            return Redirect::temporary(&error_url).into_response();
        }
    };

    match ClientService::validate_user_organization_access(&pool, client_id.to_string(), user_id_ulid) {
        Ok(true) => {}, // User has access
        Ok(false) => {
            let error_url = format!(
                "{}?error=access_denied&error_description=User+does+not+have+access+to+this+application{}",
                params.redirect_uri,
                params.state.as_ref().map(|s| format!("&state={}", s)).unwrap_or_default()
            );
            return Redirect::temporary(&error_url).into_response();
        },
        Err(_) => {
            let error_url = format!(
                "{}?error=server_error&error_description=Organization+validation+failed{}",
                params.redirect_uri,
                params.state.as_ref().map(|s| format!("&state={}", s)).unwrap_or_default()
            );
            return Redirect::temporary(&error_url).into_response();
        }
    }

    // Validate and parse scopes
    let requested_scopes: Vec<String> = params.scope
        .as_deref()
        .unwrap_or("")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let scopes = match ScopeService::validate_scopes(&pool, &requested_scopes).await {
        Ok(scopes) => scopes,
        Err(e) => {
            let error_url = format!(
                "{}?error=invalid_scope&error_description={}{}",
                params.redirect_uri,
                urlencoding::encode(&e.to_string()),
                params.state.as_ref().map(|s| format!("&state={}", s)).unwrap_or_default()
            );
            return Redirect::temporary(&error_url).into_response();
        }
    };

    // Create authorization code with OAuth 2.1 compliant PKCE
    let challenge_method = params.code_challenge_method.clone()
        .or_else(|| Some("S256".to_string())); // Default to S256 per OAuth 2.1

    let auth_code_data = CreateAuthCode {
        user_id: user_id,
        client_id: client_id.to_string(),
        scopes: ScopeService::get_scope_names(&scopes),
        redirect_uri: params.redirect_uri.clone(),
        challenge: params.code_challenge,
        challenge_method,
        expires_at: Some(Utc::now() + Duration::minutes(10)), // OAuth 2.1: Short-lived auth codes
    };

    match TokenService::create_auth_code(&pool, auth_code_data) {
        Ok(auth_code) => {
            let redirect_url = format!(
                "{}?code={}{}",
                params.redirect_uri,
                auth_code.id,
                params.state.as_ref().map(|s| format!("&state={}", s)).unwrap_or_default()
            );
            Redirect::temporary(&redirect_url).into_response()
        },
        Err(e) => {
            let error_url = format!(
                "{}?error=server_error&error_description={}{}",
                params.redirect_uri,
                urlencoding::encode(&e.to_string()),
                params.state.as_ref().map(|s| format!("&state={}", s)).unwrap_or_default()
            );
            Redirect::temporary(&error_url).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/oauth/token",
    tags = ["OAuth Core"],
    summary = "OAuth2 token endpoint",
    description = "Exchange authorization code for access token",
    request_body = TokenRequest,
    responses(
        (status = 200, description = "Token granted", body = crate::app::docs::oauth::TokenResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Client authentication failed", body = ErrorResponse)
    )
)]
pub async fn token(State(pool): State<DbPool>, headers: HeaderMap, Form(params): Form<TokenRequest>) -> impl IntoResponse {
    match params.grant_type.as_str() {
        "authorization_code" => handle_authorization_code_grant(&pool, headers, params).await.into_response(),
        "refresh_token" => handle_refresh_token_grant(&pool, headers, params).await.into_response(),
        "client_credentials" => handle_client_credentials_grant(&pool, headers, params).await.into_response(),
        "password" => {
            // OAuth 2.1 Compliance: Password grant is removed from OAuth 2.1
            let error = ErrorResponse {
                error: "unsupported_grant_type".to_string(),
                error_description: Some("Password grant is deprecated and removed in OAuth 2.1 for security reasons".to_string()),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        },
        _ => {
            let error = ErrorResponse {
                error: "unsupported_grant_type".to_string(),
                error_description: Some("Grant type not supported - OAuth 2.1 supports: authorization_code, refresh_token, client_credentials".to_string()),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }.into_response()
}

async fn handle_authorization_code_grant(pool: &DbPool, _headers: HeaderMap, params: TokenRequest) -> impl IntoResponse + use<> {
    let code = match params.code {
        Some(code) => code,
        None => {
            let error = ErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("Missing authorization code".to_string()),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    let redirect_uri = match params.redirect_uri {
        Some(uri) => uri,
        None => {
            let error = ErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("Missing redirect URI".to_string()),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    let client_id = match Ulid::from_string(&params.client_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Invalid client ID".to_string()),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    // RFC 9449: DPoP (Demonstrating Proof of Possession) support - TODO: Re-enable when DPoP service is ready
    // let dpop_proof = headers.get("dpop").and_then(|h| h.to_str().ok());
    let jwk_thumbprint: Option<String> = None;

    match TokenService::exchange_auth_code_for_tokens_with_dpop(
        pool,
        &code,
        client_id.to_string(),
        params.client_secret.as_deref(),
        &redirect_uri,
        params.code_verifier.as_deref(),
        jwk_thumbprint,
    ).await {
        Ok(token_response) => (StatusCode::OK, ResponseJson(token_response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: "invalid_grant".to_string(),
                error_description: Some(e.to_string()),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

async fn handle_refresh_token_grant(pool: &DbPool, _headers: HeaderMap, params: TokenRequest) -> impl IntoResponse + use<> {
    let refresh_token = match params.refresh_token {
        Some(token) => token,
        None => {
            let error = ErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("Missing refresh token".to_string()),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    let client_id = match Ulid::from_string(&params.client_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Invalid client ID".to_string()),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    match TokenService::refresh_access_token(
        pool,
        &refresh_token,
        client_id.to_string(),
        params.client_secret.as_deref(),
    ).await {
        Ok(token_response) => (StatusCode::OK, ResponseJson(token_response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: "invalid_grant".to_string(),
                error_description: Some(e.to_string()),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

async fn handle_client_credentials_grant(pool: &DbPool, _headers: HeaderMap, params: TokenRequest) -> impl IntoResponse + use<> {
    let client_id = match Ulid::from_string(&params.client_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Invalid client ID".to_string()),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    // Verify client credentials
    let client = match params.client_secret {
        Some(ref secret) => ClientService::find_by_id_and_secret(pool, client_id.to_string(), secret),
        None => ClientService::find_by_id(pool, client_id.to_string()),
    };

    let client = match client {
        Ok(Some(client)) => client,
        _ => {
            let error = ErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Invalid client credentials".to_string()),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    if client.has_secret() && params.client_secret.is_none() {
        let error = ErrorResponse {
            error: "invalid_client".to_string(),
            error_description: Some("Client secret is required".to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    // Parse requested scopes
    let requested_scopes: Vec<String> = params.scope
        .as_deref()
        .unwrap_or("")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let scopes = match ScopeService::validate_scopes(pool, &requested_scopes).await {
        Ok(scopes) => scopes,
        Err(e) => {
            let error = ErrorResponse {
                error: "invalid_scope".to_string(),
                error_description: Some(e.to_string()),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    // Create access token (no user for client credentials)
    let create_token = crate::app::models::oauth::CreateAccessToken {
        user_id: None,
        client_id: client_id.to_string(),
        name: None,
        scopes: ScopeService::get_scope_names(&scopes),
        expires_at: Some(Utc::now() + Duration::seconds(3600)), // 1 hour
        jwk_thumbprint: None, // TODO: Add DPoP support for client credentials flow
    };

    // TODO: Extract user_id from auth context when available
    let granted_by = None; // Replace with actual user extraction

    match TokenService::create_access_token(pool, create_token, Some(3600), granted_by).await {
        Ok(access_token) => {
            match TokenService::generate_jwt_token(&access_token, &client_id.to_string()) {
                Ok(jwt_token) => {
                    let token_response = crate::app::services::oauth::TokenResponse {
                        access_token: jwt_token,
                        token_type: "Bearer".to_string(),
                        expires_in: 3600,
                        refresh_token: None,
                        scope: ScopeService::get_scope_names(&scopes).join(" "),
                    };
                    (StatusCode::OK, ResponseJson(token_response)).into_response()
                },
                Err(e) => {
                    let error = ErrorResponse {
                        error: "server_error".to_string(),
                        error_description: Some(e.to_string()),
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
                }
            }
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

#[utoipa::path(
    post,
    path = "/oauth/introspect",
    tags = ["OAuth Core"],
    summary = "OAuth2 token introspection endpoint",
    description = "Inspect access token validity and metadata",
    request_body = IntrospectRequest,
    responses(
        (status = 200, description = "Token introspection result", body = IntrospectResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse)
    )
)]
pub async fn introspect(State(pool): State<DbPool>, Json(params): Json<IntrospectRequest>) -> impl IntoResponse {
    let token_claims = match TokenService::decode_jwt_token(&params.token) {
        Ok(claims) => claims,
        Err(_) => {
            let response = IntrospectResponse {
                active: false,
                scope: None,
                client_id: None,
                username: None,
                exp: None,
                iat: None,
                sub: None,
                aud: None,
            };
            return (StatusCode::OK, ResponseJson(response)).into_response();
        }
    };

    // Parse token ID and find access token
    let token_id = match Ulid::from_string(&token_claims.jti.to_string()) {
        Ok(id) => id,
        Err(_) => {
            let response = IntrospectResponse {
                active: false,
                scope: None,
                client_id: None,
                username: None,
                exp: None,
                iat: None,
                sub: None,
                aud: None,
            };
            return (StatusCode::OK, ResponseJson(response)).into_response();
        }
    };

    let access_token = match TokenService::find_access_token_by_id(&pool, token_id.to_string()) {
        Ok(Some(token)) => token,
        _ => {
            let response = IntrospectResponse {
                active: false,
                scope: None,
                client_id: None,
                username: None,
                exp: None,
                iat: None,
                sub: None,
                aud: None,
            };
            return (StatusCode::OK, ResponseJson(response)).into_response();
        }
    };

    let response = IntrospectResponse {
        active: access_token.is_valid(),
        scope: Some(access_token.get_scopes().join(" ")),
        client_id: Some(access_token.client_id.to_string()),
        username: access_token.user_id.map(|id| id.to_string()),
        exp: Some(token_claims.exp as i64),
        iat: Some(token_claims.iat as i64),
        sub: Some(token_claims.sub.to_string()),
        aud: Some(token_claims.aud.to_string()),
    };

    (StatusCode::OK, ResponseJson(response)).into_response()
}

#[utoipa::path(
    post,
    path = "/oauth/revoke",
    tags = ["OAuth Core"],
    summary = "OAuth2 token revocation endpoint",
    description = "Revoke access or refresh token",
    request_body(content = HashMap<String, String>, description = "Token revocation form data"),
    responses(
        (status = 200, description = "Token revoked successfully"),
        (status = 400, description = "Invalid request", body = ErrorResponse)
    )
)]
pub async fn revoke(State(pool): State<DbPool>, Form(params): Form<HashMap<String, String>>) -> impl IntoResponse {
    let token = match params.get("token") {
        Some(token) => token,
        None => {
            let error = ErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("Missing token parameter".to_string()),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    // Try to decode as JWT first to get token ID
    let token_id = match TokenService::decode_jwt_token(token) {
        Ok(claims) => match Ulid::from_string(&claims.jti.to_string()) {
            Ok(id) => id,
            Err(_) => {
                let error = ErrorResponse {
                    error: "invalid_request".to_string(),
                    error_description: Some("Invalid token format".to_string()),
                };
                return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
            }
        },
        Err(_) => {
            // Maybe it's a refresh token ID
            match Ulid::from_string(token) {
                Ok(id) => id,
                Err(_) => {
                    let error = ErrorResponse {
                        error: "invalid_request".to_string(),
                        error_description: Some("Invalid token format".to_string()),
                    };
                    return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
                }
            }
        }
    };

    // Try to revoke as access token first
    if let Ok(_) = TokenService::revoke_access_token(&pool, token_id.to_string()) {
        return (StatusCode::OK, ResponseJson(serde_json::json!({"revoked": true}))).into_response();
    }

    // Try to revoke as refresh token
    if let Ok(_) = TokenService::revoke_refresh_token(&pool, token_id.to_string()) {
        return (StatusCode::OK, ResponseJson(serde_json::json!({"revoked": true}))).into_response();
    }

    // Token not found, but still return success per OAuth spec
    (StatusCode::OK, ResponseJson(serde_json::json!({"revoked": true}))).into_response()
}

async fn handle_password_grant(pool: &DbPool, params: TokenRequest) -> impl IntoResponse + use<> {
    let username = match params.username {
        Some(username) => username,
        None => {
            let error = ErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("Missing username parameter".to_string()),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    let password = match params.password {
        Some(password) => password,
        None => {
            let error = ErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("Missing password parameter".to_string()),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    let client_id = match Ulid::from_string(&params.client_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Invalid client ID".to_string()),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    // Verify client credentials
    let client = match params.client_secret {
        Some(ref secret) => ClientService::find_by_id_and_secret(pool, client_id.to_string(), secret),
        None => ClientService::find_by_id(pool, client_id.to_string()),
    };

    let client = match client {
        Ok(Some(client)) => client,
        _ => {
            let error = ErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Invalid client credentials".to_string()),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    // Check if client supports password grant
    if !client.password_client {
        let error = ErrorResponse {
            error: "unauthorized_client".to_string(),
            error_description: Some("Client is not authorized for password grant".to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    if client.has_secret() && params.client_secret.is_none() {
        let error = ErrorResponse {
            error: "invalid_client".to_string(),
            error_description: Some("Client secret is required".to_string()),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    // Authenticate user
    let user_id = match AuthService::authenticate_user(&username, &password, pool).await {
        Ok(user_id) => user_id,
        Err(_) => {
            let error = ErrorResponse {
                error: "invalid_grant".to_string(),
                error_description: Some("Invalid user credentials".to_string()),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    // Validate user has access to this OAuth client's organization
    let user_id_ulid = match Ulid::from_string(&user_id) {
        Ok(id) => DieselUlid(id),
        Err(_) => {
            let error = ErrorResponse {
                error: "server_error".to_string(),
                error_description: Some("Invalid user ID".to_string()),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    };

    match ClientService::validate_user_organization_access(pool, client_id.to_string(), user_id_ulid) {
        Ok(true) => {}, // User has access
        Ok(false) => {
            let error = ErrorResponse {
                error: "access_denied".to_string(),
                error_description: Some("User does not have access to this application".to_string()),
            };
            return (StatusCode::FORBIDDEN, ResponseJson(error)).into_response();
        },
        Err(_) => {
            let error = ErrorResponse {
                error: "server_error".to_string(),
                error_description: Some("Organization validation failed".to_string()),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    }

    // Parse requested scopes
    let requested_scopes: Vec<String> = params.scope
        .as_deref()
        .unwrap_or("")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let scopes = match ScopeService::validate_scopes(pool, &requested_scopes).await {
        Ok(scopes) => scopes,
        Err(e) => {
            let error = ErrorResponse {
                error: "invalid_scope".to_string(),
                error_description: Some(e.to_string()),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    // Create access token for the authenticated user
    let create_token = crate::app::models::oauth::CreateAccessToken {
        user_id: Some(user_id),
        client_id: client_id.to_string(),
        name: Some("Password Grant Token".to_string()),
        scopes: ScopeService::get_scope_names(&scopes),
        expires_at: Some(Utc::now() + Duration::seconds(3600)), // 1 hour
        jwk_thumbprint: None, // Password flow doesn't use DPoP
    };

    // TODO: Extract user_id from auth context when available
    let granted_by = None; // Replace with actual user extraction

    match TokenService::create_access_token(pool, create_token, Some(3600), granted_by).await {
        Ok(access_token) => {
            // Create refresh token
            let refresh_token = match TokenService::create_refresh_token(
                pool,
                access_token.id.to_string(),
                Some(604800), // 7 days
            ) {
                Ok(token) => Some(token.id.to_string()),
                Err(_) => None,
            };

            match TokenService::generate_jwt_token(&access_token, &client_id.to_string()) {
                Ok(jwt_token) => {
                    let token_response = crate::app::services::oauth::TokenResponse {
                        access_token: jwt_token,
                        token_type: "Bearer".to_string(),
                        expires_in: 3600,
                        refresh_token,
                        scope: ScopeService::get_scope_names(&scopes).join(" "),
                    };
                    (StatusCode::OK, ResponseJson(token_response)).into_response()
                },
                Err(e) => {
                    let error = ErrorResponse {
                        error: "server_error".to_string(),
                        error_description: Some(e.to_string()),
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
                }
            }
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

async fn get_user_from_token(_pool: &DbPool, auth_header: Option<&str>) -> anyhow::Result<String> {
    let token = TokenUtils::extract_token_from_header(auth_header)?;
    let claims = AuthService::decode_token(token, "jwt-secret")?;

    let user_id = Ulid::from_string(&claims.sub)?;
    Ok(user_id.to_string())
}