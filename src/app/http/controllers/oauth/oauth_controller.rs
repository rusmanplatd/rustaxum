use axum::{
    extract::{Query, Json, State, Form},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json as ResponseJson, Redirect},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use ulid::Ulid;
use chrono::{Duration, Utc};
use std::collections::HashMap;

use crate::app::services::oauth::{TokenService, ClientService, ScopeService};
use crate::app::services::auth_service::AuthService;
use crate::app::models::oauth::{CreateAuthCode};
use crate::app::utils::token_utils::TokenUtils;

#[derive(Serialize)]
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

#[derive(Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub code_verifier: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

#[derive(Deserialize)]
pub struct IntrospectRequest {
    pub token: String,
    pub token_type_hint: Option<String>,
}

#[derive(Serialize)]
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

pub async fn authorize(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Query(params): Query<AuthorizeQuery>,
) -> impl IntoResponse {
    // Validate required parameters
    if params.response_type != "code" {
        let error_url = format!(
            "{}?error=unsupported_response_type&error_description=Only+authorization+code+flow+is+supported{}",
            params.redirect_uri,
            params.state.as_ref().map(|s| format!("&state={}", s)).unwrap_or_default()
        );
        return Redirect::temporary(&error_url).into_response();
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

    // Validate client and redirect URI
    match ClientService::is_valid_redirect_uri(&pool, client_id, &params.redirect_uri).await {
        Ok(true) => {},
        Ok(false) => {
            return (StatusCode::BAD_REQUEST, ResponseJson(ErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Invalid redirect URI".to_string()),
            })).into_response();
        },
        Err(_) => {
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

    // Create authorization code
    let auth_code_data = CreateAuthCode {
        user_id,
        client_id,
        scopes: ScopeService::get_scope_names(&scopes).await,
        redirect_uri: params.redirect_uri.clone(),
        challenge: params.code_challenge,
        challenge_method: params.code_challenge_method,
        expires_at: Some(Utc::now() + Duration::minutes(10)), // 10 minutes
    };

    match TokenService::create_auth_code(&pool, auth_code_data).await {
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

pub async fn token(State(pool): State<PgPool>, Form(params): Form<TokenRequest>) -> impl IntoResponse {
    match params.grant_type.as_str() {
        "authorization_code" => handle_authorization_code_grant(&pool, params).await.into_response(),
        "refresh_token" => handle_refresh_token_grant(&pool, params).await.into_response(),
        "client_credentials" => handle_client_credentials_grant(&pool, params).await.into_response(),
        _ => {
            let error = ErrorResponse {
                error: "unsupported_grant_type".to_string(),
                error_description: Some("Grant type not supported".to_string()),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }.into_response()
}

async fn handle_authorization_code_grant(pool: &PgPool, params: TokenRequest) -> impl IntoResponse {
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

    match TokenService::exchange_auth_code_for_tokens(
        pool,
        &code,
        client_id,
        params.client_secret.as_deref(),
        &redirect_uri,
        params.code_verifier.as_deref(),
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

async fn handle_refresh_token_grant(pool: &PgPool, params: TokenRequest) -> impl IntoResponse {
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
        client_id,
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

async fn handle_client_credentials_grant(pool: &PgPool, params: TokenRequest) -> impl IntoResponse {
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
        Some(ref secret) => ClientService::find_by_id_and_secret(pool, client_id, secret).await,
        None => ClientService::find_by_id(pool, client_id).await,
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
        client_id,
        name: None,
        scopes: ScopeService::get_scope_names(&scopes).await,
        expires_at: Some(Utc::now() + Duration::seconds(3600)), // 1 hour
    };

    match TokenService::create_access_token(pool, create_token, Some(3600)).await {
        Ok(access_token) => {
            match TokenService::generate_jwt_token(&access_token, &client_id.to_string()) {
                Ok(jwt_token) => {
                    let token_response = crate::app::services::oauth::TokenResponse {
                        access_token: jwt_token,
                        token_type: "Bearer".to_string(),
                        expires_in: 3600,
                        refresh_token: None,
                        scope: ScopeService::get_scope_names(&scopes).await.join(" "),
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

pub async fn introspect(State(pool): State<PgPool>, Json(params): Json<IntrospectRequest>) -> impl IntoResponse {
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
    let token_id = match Ulid::from_string(&token_claims.jti) {
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

    let access_token = match TokenService::find_access_token_by_id(&pool, token_id).await {
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
        sub: if token_claims.sub.is_empty() { None } else { Some(token_claims.sub) },
        aud: Some(token_claims.aud),
    };

    (StatusCode::OK, ResponseJson(response)).into_response()
}

pub async fn revoke(State(pool): State<PgPool>, Form(params): Form<HashMap<String, String>>) -> impl IntoResponse {
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
        Ok(claims) => match Ulid::from_string(&claims.jti) {
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
    if let Ok(_) = TokenService::revoke_access_token(&pool, token_id).await {
        return (StatusCode::OK, ResponseJson(serde_json::json!({"revoked": true}))).into_response();
    }

    // Try to revoke as refresh token
    if let Ok(_) = TokenService::revoke_refresh_token(&pool, token_id).await {
        return (StatusCode::OK, ResponseJson(serde_json::json!({"revoked": true}))).into_response();
    }

    // Token not found, but still return success per OAuth spec
    (StatusCode::OK, ResponseJson(serde_json::json!({"revoked": true}))).into_response()
}

async fn get_user_from_token(pool: &PgPool, auth_header: Option<&str>) -> anyhow::Result<Ulid> {
    let token = TokenUtils::extract_token_from_header(auth_header)?;
    let claims = AuthService::decode_token(token, "jwt-secret")?;

    let user_id = Ulid::from_string(&claims.sub)?;
    Ok(user_id)
}