use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::database::DbPool;
use crate::app::services::oauth::{DPoPService, TokenService};

/// RFC 9449: DPoP (Demonstrating Proof of Possession) middleware
/// Validates DPoP tokens for protected resources
pub async fn dpop_validation_middleware(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check if this is a DPoP-bound request
    if let Some(dpop_header) = headers.get("dpop") {
        if let Some(auth_header) = headers.get("authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("DPoP ") {
                    let access_token = &auth_str[5..]; // Remove "DPoP " prefix

                    // Validate the DPoP proof
                    if let Err(e) = validate_dpop_request(
                        &pool,
                        dpop_header.to_str().unwrap_or(""),
                        access_token,
                        request.method().as_str(),
                        request.uri().to_string().as_str(),
                    ).await {
                        tracing::warn!("DPoP validation failed: {}", e);
                        return Err(StatusCode::UNAUTHORIZED);
                    }

                    // Add DPoP info to request extensions for downstream handlers
                    request.extensions_mut().insert(DPoPInfo {
                        validated: true,
                        access_token: access_token.to_string(),
                    });
                }
            }
        }
    }

    Ok(next.run(request).await)
}

#[derive(Clone)]
pub struct DPoPInfo {
    pub validated: bool,
    pub access_token: String,
}

async fn validate_dpop_request(
    pool: &DbPool,
    dpop_proof: &str,
    access_token: &str,
    http_method: &str,
    http_url: &str,
) -> anyhow::Result<()> {
    // First decode the JWT access token to get the token ID and validate it
    let token_claims = TokenService::decode_jwt_token(access_token)?;
    let token_id = ulid::Ulid::from_string(&token_claims.jti.to_string())?;

    // Find the access token record in the database
    let db_token = TokenService::find_access_token_by_id(pool, token_id.to_string())?
        .ok_or_else(|| anyhow::anyhow!("Access token not found"))?;

    // Check if token is valid and not revoked
    if !db_token.is_valid() {
        return Err(anyhow::anyhow!("Access token is invalid or expired"));
    }

    // If token has a JWK thumbprint, it's DPoP-bound and we need to validate the proof
    if let Some(expected_jwk_thumbprint) = &db_token.jwk_thumbprint {
        DPoPService::validate_dpop_bound_token(
            dpop_proof,
            access_token,
            http_method,
            http_url,
            expected_jwk_thumbprint,
        )?;

        tracing::debug!("DPoP validation successful for token {}", token_id);
    } else {
        // Token is not DPoP-bound, but client sent DPoP header
        // This is allowed - client can send DPoP even for Bearer tokens
        tracing::debug!("DPoP proof provided for Bearer token, ignoring");
    }

    Ok(())
}