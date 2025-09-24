use anyhow::Result;
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use crate::database::DbPool;

/// RFC 8705: OAuth 2.0 Mutual-TLS Client Authentication and Certificate-Bound Access Tokens
///
/// This service provides mutual TLS (mTLS) client authentication capabilities,
/// allowing clients to authenticate using X.509 certificates instead of client secrets.
pub struct MTLSService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCertificate {
    pub subject_dn: String,
    pub issuer_dn: String,
    pub serial_number: String,
    pub thumbprint_sha256: String,
    pub not_before: String,
    pub not_after: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MTLSClientAuthResult {
    pub authenticated: bool,
    pub client_id: Option<String>,
    pub certificate: Option<ClientCertificate>,
    pub thumbprint: Option<String>,
}

impl MTLSService {
    /// Extract client certificate from HTTP headers
    /// TODO: In production, this would typically come from the TLS terminator/proxy
    pub fn extract_client_certificate(headers: &HeaderMap) -> Result<Option<ClientCertificate>> {
        // TODO: In a real implementation, the TLS terminator (nginx, HAProxy, etc.)
        // would extract the client certificate and pass it via headers

        // Common header names used by TLS terminators:
        // - X-Client-Cert (nginx)
        // - X-SSL-Client-Cert (Apache)
        // - X-Forwarded-Client-Cert (Envoy)

        if let Some(cert_header) = headers.get("x-client-cert") {
            let cert_pem = cert_header.to_str()
                .map_err(|_| anyhow::anyhow!("Invalid certificate header"))?;

            Self::parse_certificate_info(cert_pem)
        } else if let Some(cert_header) = headers.get("x-ssl-client-cert") {
            let cert_pem = cert_header.to_str()
                .map_err(|_| anyhow::anyhow!("Invalid certificate header"))?;

            Self::parse_certificate_info(cert_pem)
        } else if let Some(cert_header) = headers.get("x-forwarded-client-cert") {
            let cert_pem = cert_header.to_str()
                .map_err(|_| anyhow::anyhow!("Invalid certificate header"))?;

            Self::parse_certificate_info(cert_pem)
        } else {
            Ok(None)
        }
    }

    /// Parse certificate information from PEM format
    /// TODO: In production, you'd use a proper X.509 parsing library like `x509-parser` or `rustls`
    fn parse_certificate_info(cert_pem: &str) -> Result<Option<ClientCertificate>> {
        // TODO: In production, use proper X.509 certificate parsing

        if cert_pem.is_empty() || !cert_pem.contains("BEGIN CERTIFICATE") {
            return Ok(None);
        }

        // Extract certificate data (this is a mock implementation)
        // In real implementation, parse the actual X.509 certificate
        let cert_data = cert_pem.replace("-----BEGIN CERTIFICATE-----", "")
            .replace("-----END CERTIFICATE-----", "")
            .replace("\n", "")
            .replace(" ", "");

        // Calculate SHA-256 thumbprint
        let cert_bytes = URL_SAFE_NO_PAD.decode(&cert_data)
            .map_err(|_| anyhow::anyhow!("Invalid certificate encoding"))?;

        let mut hasher = Sha256::new();
        hasher.update(&cert_bytes);
        let thumbprint = URL_SAFE_NO_PAD.encode(hasher.finalize());

        // TODO: in production, extract from actual certificate
        Ok(Some(ClientCertificate {
            subject_dn: "CN=client.example.com,O=Example Corp,C=US".to_string(),
            issuer_dn: "CN=CA,O=Example CA,C=US".to_string(),
            serial_number: "1234567890".to_string(),
            thumbprint_sha256: thumbprint,
            not_before: "2023-01-01T00:00:00Z".to_string(),
            not_after: "2025-01-01T00:00:00Z".to_string(),
        }))
    }

    /// Authenticate client using mTLS certificate
    /// RFC 8705: Client authentication using X.509 certificates
    pub async fn authenticate_client_certificate(
        pool: &DbPool,
        headers: &HeaderMap,
        client_id: &str,
    ) -> Result<MTLSClientAuthResult> {
        // Extract client certificate
        let certificate = match Self::extract_client_certificate(headers)? {
            Some(cert) => cert,
            None => {
                return Ok(MTLSClientAuthResult {
                    authenticated: false,
                    client_id: None,
                    certificate: None,
                    thumbprint: None,
                });
            }
        };

        // Validate certificate against registered client certificate
        let is_valid = Self::validate_client_certificate(pool, client_id, &certificate).await?;

        Ok(MTLSClientAuthResult {
            authenticated: is_valid,
            client_id: if is_valid { Some(client_id.to_string()) } else { None },
            certificate: Some(certificate.clone()),
            thumbprint: Some(certificate.thumbprint_sha256.clone()),
        })
    }

    /// Validate client certificate against registered certificate
    async fn validate_client_certificate(
        pool: &DbPool,
        client_id: &str,
        certificate: &ClientCertificate,
    ) -> Result<bool> {
        use crate::app::services::oauth::ClientService;

        // Find the client
        let client = match ClientService::find_by_id(pool, client_id.to_string())? {
            Some(client) => client,
            None => return Ok(false),
        };

        // TODO: In production, you would store registered client certificates in the database
        
        // Check if client supports mTLS authentication
        // This could be a flag in the client configuration
        if !Self::client_supports_mtls(&client) {
            return Ok(false);
        }

        // Validate certificate properties
        Self::validate_certificate_properties(certificate)?;

        // TODO: In production: Check certificate against stored thumbprint/subject DN
        Ok(true)
    }

    /// Check if client supports mTLS authentication
    fn client_supports_mtls(client: &crate::app::models::oauth::Client) -> bool {
        // TODO: In production, this would check a database field
        client.secret.is_none()
    }

    /// Validate certificate properties (expiration, etc.)
    fn validate_certificate_properties(certificate: &ClientCertificate) -> Result<bool> {
        // TODO: In production, validate:
        // - Certificate is not expired
        // - Certificate chain is valid
        // - Certificate is issued by trusted CA
        // - Certificate has appropriate key usage

        // For demonstration, basic validation
        if certificate.thumbprint_sha256.is_empty() {
            return Ok(false);
        }

        if certificate.subject_dn.is_empty() {
            return Ok(false);
        }

        Ok(true)
    }

    /// Generate certificate-bound access token thumbprint
    /// RFC 8705: Certificate thumbprint for token binding
    pub fn generate_certificate_thumbprint(certificate: &ClientCertificate) -> String {
        certificate.thumbprint_sha256.clone()
    }

    /// Validate certificate-bound access token
    pub fn validate_certificate_bound_token(
        access_token_cnf: &str,
        client_certificate: &ClientCertificate,
    ) -> Result<bool> {
        // RFC 8705: Validate that the access token is bound to the client certificate
        let expected_thumbprint = Self::generate_certificate_thumbprint(client_certificate);
        Ok(access_token_cnf == expected_thumbprint)
    }

    /// Create certificate-bound JWT claims
    /// RFC 8705: Add certificate confirmation (cnf) claim to JWT
    pub fn create_certificate_bound_claims(
        certificate: &ClientCertificate,
        base_claims: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let mut claims = base_claims;

        // Add certificate confirmation claim (RFC 8705)
        let cnf_claim = serde_json::json!({
            "x5t#S256": certificate.thumbprint_sha256
        });

        claims["cnf"] = cnf_claim;
        Ok(claims)
    }

    /// Extract certificate information for logging/audit
    pub fn extract_certificate_info_for_audit(
        certificate: &ClientCertificate,
    ) -> serde_json::Value {
        serde_json::json!({
            "authentication_method": "tls_client_auth",
            "certificate_subject": certificate.subject_dn,
            "certificate_issuer": certificate.issuer_dn,
            "certificate_serial": certificate.serial_number,
            "certificate_thumbprint": certificate.thumbprint_sha256,
            "certificate_not_before": certificate.not_before,
            "certificate_not_after": certificate.not_after
        })
    }

    /// Validate mTLS endpoint constraints
    /// RFC 8705: Ensure token is used on correct mTLS endpoint
    pub fn validate_mtls_endpoint_constraint(
        headers: &HeaderMap,
        required_endpoint: &str,
    ) -> Result<bool> {
        // Check if request came through mTLS endpoint
        // This would typically be validated by checking:
        // 1. TLS client certificate is present
        // 2. Request was made to mTLS-specific endpoint
        // 3. Proper TLS version and cipher suites were used

        let has_client_cert = Self::extract_client_certificate(headers)?.is_some();

        // TODO: In production, also validate the endpoint URL
        let is_mtls_endpoint = required_endpoint.contains("mtls") ||
                              headers.get("x-mtls-endpoint").is_some();

        Ok(has_client_cert && is_mtls_endpoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn test_certificate_parsing() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-client-cert",
            HeaderValue::from_static("-----BEGIN CERTIFICATE-----\nVGVzdCBjZXJ0aWZpY2F0ZQ==\n-----END CERTIFICATE-----")
        );

        let result = MTLSService::extract_client_certificate(&headers);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_certificate_thumbprint() {
        let cert = ClientCertificate {
            subject_dn: "CN=test.example.com".to_string(),
            issuer_dn: "CN=Test CA".to_string(),
            serial_number: "123".to_string(),
            thumbprint_sha256: "test_thumbprint".to_string(),
            not_before: "2023-01-01T00:00:00Z".to_string(),
            not_after: "2025-01-01T00:00:00Z".to_string(),
        };

        let thumbprint = MTLSService::generate_certificate_thumbprint(&cert);
        assert_eq!(thumbprint, "test_thumbprint");
    }

    #[test]
    fn test_certificate_bound_claims() {
        let cert = ClientCertificate {
            subject_dn: "CN=test.example.com".to_string(),
            issuer_dn: "CN=Test CA".to_string(),
            serial_number: "123".to_string(),
            thumbprint_sha256: "test_thumbprint".to_string(),
            not_before: "2023-01-01T00:00:00Z".to_string(),
            not_after: "2025-01-01T00:00:00Z".to_string(),
        };

        let base_claims = serde_json::json!({
            "sub": "user123",
            "exp": 1234567890
        });

        let result = MTLSService::create_certificate_bound_claims(&cert, base_claims);
        assert!(result.is_ok());

        let claims = result.unwrap();
        assert!(claims["cnf"]["x5t#S256"] == "test_thumbprint");
    }
}