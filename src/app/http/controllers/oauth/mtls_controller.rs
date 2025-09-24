use axum::{
    extract::State,
    response::Json,
    http::{StatusCode, HeaderMap},
};
use serde_json::{json, Value};
use crate::database::DbPool;
use crate::app::services::oauth::MTLSService;

/// RFC 8705: OAuth 2.0 Mutual-TLS Client Authentication Controller
///
/// This controller provides endpoints for mTLS client authentication and certificate management,
/// enabling clients to authenticate using X.509 certificates instead of client secrets.

/// Validate client certificate (for testing/debugging)
/// POST /oauth/mtls/validate
pub async fn validate_client_certificate(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let client_id = payload.get("client_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_request",
                "error_description": "client_id is required"
            }))
        ))?;

    // Validate client certificate
    match MTLSService::authenticate_client_certificate(&pool, &headers, client_id).await {
        Ok(auth_result) => {
            tracing::info!("mTLS authentication result for client {}: {}", client_id, auth_result.authenticated);
            Ok(Json(json!({
                "authenticated": auth_result.authenticated,
                "client_id": auth_result.client_id,
                "certificate_present": auth_result.certificate.is_some(),
                "certificate_thumbprint": auth_result.thumbprint,
                "certificate_info": auth_result.certificate.as_ref().map(|cert| {
                    MTLSService::extract_certificate_info_for_audit(cert)
                })
            })))
        }
        Err(err) => {
            tracing::error!("mTLS validation failed for client {}: {}", client_id, err);
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_request",
                    "error_description": err.to_string()
                }))
            ))
        }
    }
}

/// Extract certificate information from headers
/// GET /oauth/mtls/certificate-info
pub async fn get_certificate_info(
    headers: HeaderMap,
    State(_pool): State<DbPool>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match MTLSService::extract_client_certificate(&headers) {
        Ok(Some(certificate)) => {
            let cert_info = MTLSService::extract_certificate_info_for_audit(&certificate);
            let thumbprint = MTLSService::generate_certificate_thumbprint(&certificate);

            Ok(Json(json!({
                "certificate_present": true,
                "certificate_info": cert_info,
                "certificate_thumbprint": thumbprint,
                "certificate_details": {
                    "subject_dn": certificate.subject_dn,
                    "issuer_dn": certificate.issuer_dn,
                    "serial_number": certificate.serial_number,
                    "not_before": certificate.not_before,
                    "not_after": certificate.not_after
                }
            })))
        }
        Ok(None) => {
            Ok(Json(json!({
                "certificate_present": false,
                "message": "No client certificate found in request headers"
            })))
        }
        Err(err) => {
            tracing::error!("Failed to extract certificate info: {}", err);
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_certificate",
                    "error_description": err.to_string()
                }))
            ))
        }
    }
}

/// Validate certificate-bound token
/// POST /oauth/mtls/validate-bound-token
pub async fn validate_certificate_bound_token(
    headers: HeaderMap,
    State(_pool): State<DbPool>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let access_token_cnf = payload.get("cnf")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_request",
                "error_description": "cnf (confirmation) claim is required"
            }))
        ))?;

    // Extract client certificate
    let certificate = match MTLSService::extract_client_certificate(&headers)? {
        Some(cert) => cert,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_request",
                    "error_description": "Client certificate required for bound token validation"
                }))
            ));
        }
    };

    // Validate token binding
    match MTLSService::validate_certificate_bound_token(access_token_cnf, &certificate) {
        Ok(is_bound) => {
            Ok(Json(json!({
                "token_bound": is_bound,
                "certificate_thumbprint": MTLSService::generate_certificate_thumbprint(&certificate),
                "provided_cnf": access_token_cnf
            })))
        }
        Err(err) => {
            tracing::error!("Failed to validate certificate-bound token: {}", err);
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_token",
                    "error_description": err.to_string()
                }))
            ))
        }
    }
}

/// Create certificate-bound JWT claims (for testing)
/// POST /oauth/mtls/create-bound-claims
pub async fn create_certificate_bound_claims(
    headers: HeaderMap,
    State(_pool): State<DbPool>,
    Json(base_claims): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Extract client certificate
    let certificate = match MTLSService::extract_client_certificate(&headers)? {
        Some(cert) => cert,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_request",
                    "error_description": "Client certificate required for creating bound claims"
                }))
            ));
        }
    };

    // Create certificate-bound claims
    match MTLSService::create_certificate_bound_claims(&certificate, base_claims) {
        Ok(bound_claims) => {
            Ok(Json(json!({
                "bound_claims": bound_claims,
                "certificate_thumbprint": MTLSService::generate_certificate_thumbprint(&certificate)
            })))
        }
        Err(err) => {
            tracing::error!("Failed to create certificate-bound claims: {}", err);
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_request",
                    "error_description": err.to_string()
                }))
            ))
        }
    }
}

/// Validate mTLS endpoint constraint
/// POST /oauth/mtls/validate-endpoint
pub async fn validate_mtls_endpoint(
    headers: HeaderMap,
    State(_pool): State<DbPool>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let required_endpoint = payload.get("endpoint")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_request",
                "error_description": "endpoint parameter is required"
            }))
        ))?;

    match MTLSService::validate_mtls_endpoint_constraint(&headers, required_endpoint) {
        Ok(is_valid) => {
            Ok(Json(json!({
                "endpoint_valid": is_valid,
                "required_endpoint": required_endpoint,
                "has_client_certificate": MTLSService::extract_client_certificate(&headers)
                    .unwrap_or(None)
                    .is_some()
            })))
        }
        Err(err) => {
            tracing::error!("Failed to validate mTLS endpoint constraint: {}", err);
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_request",
                    "error_description": err.to_string()
                }))
            ))
        }
    }
}