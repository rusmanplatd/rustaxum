use axum::{
    extract::{State, Request},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::{IntoResponse, Response, Json as ResponseJson},
};
use serde::Serialize;
use sqlx::PgPool;

use crate::app::services::oauth::TokenService;
use crate::app::utils::token_utils::TokenUtils;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    error_description: Option<String>,
}

pub async fn oauth_middleware(
    State(pool): State<PgPool>,
    mut req: Request,
    next: Next,
) -> Result<Response, Response> {
    let headers = req.headers().clone();

    match validate_oauth_token(&pool, &headers).await {
        Ok((access_token, claims)) => {
            // Add token info to request extensions so controllers can access it
            req.extensions_mut().insert(access_token);
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        },
        Err(e) => {
            let error = ErrorResponse {
                error: "invalid_token".to_string(),
                error_description: Some(e.to_string()),
            };
            Err((StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response())
        }
    }
}

pub fn require_scopes(required_scopes: Vec<&'static str>) -> impl Fn(State<PgPool>, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, Response>> + Send>> + Clone {
    move |State(pool): State<PgPool>, mut req: Request, next: Next| {
        let required_scopes = required_scopes.clone();
        Box::pin(async move {
            let headers = req.headers().clone();

            match validate_oauth_token(&pool, &headers).await {
                Ok((access_token, claims)) => {
                    // Check if token has required scopes
                    let has_required_scopes = required_scopes.iter().all(|required_scope| {
                        access_token.has_scope(required_scope) || claims.scopes.contains(&required_scope.to_string())
                    });

                    if !has_required_scopes {
                        let error = ErrorResponse {
                            error: "insufficient_scope".to_string(),
                            error_description: Some(format!(
                                "The request requires higher privileges than provided by the access token. Required scopes: {}",
                                required_scopes.join(", ")
                            )),
                        };
                        return Err((StatusCode::FORBIDDEN, ResponseJson(error)).into_response());
                    }

                    // Add token info to request extensions
                    req.extensions_mut().insert(access_token);
                    req.extensions_mut().insert(claims);
                    Ok(next.run(req).await)
                },
                Err(e) => {
                    let error = ErrorResponse {
                        error: "invalid_token".to_string(),
                        error_description: Some(e.to_string()),
                    };
                    Err((StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response())
                }
            }
        })
    }
}

pub fn require_scope(required_scope: &'static str) -> impl Fn(State<PgPool>, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, Response>> + Send>> + Clone {
    require_scopes(vec![required_scope])
}

async fn validate_oauth_token(
    pool: &PgPool,
    headers: &HeaderMap,
) -> anyhow::Result<(crate::app::models::oauth::AccessToken, crate::app::services::oauth::TokenClaims)> {
    // Extract Bearer token from Authorization header
    let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());
    let token = TokenUtils::extract_token_from_header(auth_header)?;

    // Validate token and check scopes
    let (access_token, claims) = TokenService::validate_token_and_scopes(pool, token, &[]).await?;

    Ok((access_token, claims))
}

// Convenience macros for common scope requirements
#[macro_export]
macro_rules! require_read_scope {
    () => {
        $crate::app::http::middleware::oauth::require_scope("read")
    };
}

#[macro_export]
macro_rules! require_write_scope {
    () => {
        $crate::app::http::middleware::oauth::require_scope("write")
    };
}

#[macro_export]
macro_rules! require_admin_scope {
    () => {
        $crate::app::http::middleware::oauth::require_scope("admin")
    };
}

#[macro_export]
macro_rules! require_custom_scopes {
    ($($scope:expr_2021),+) => {
        $crate::app::http::middleware::oauth::require_scopes(vec![$($scope),+])
    };
}