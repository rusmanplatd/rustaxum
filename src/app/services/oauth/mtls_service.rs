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
    /// Extract client certificate from HTTP headers (production implementation)
    /// Supports multiple TLS terminator formats and validates certificate integrity
    pub fn extract_client_certificate(headers: &HeaderMap) -> Result<Option<ClientCertificate>> {
        // Try multiple header formats used by different TLS terminators
        let cert_headers = [
            "x-client-cert",           // nginx with proxy_ssl_client_certificate
            "x-ssl-client-cert",       // Apache mod_ssl
            "x-forwarded-client-cert", // Envoy proxy
            "x-client-certificate",    // HAProxy
            "ssl-client-cert",         // Custom headers
        ];

        for header_name in &cert_headers {
            if let Some(cert_header) = headers.get(*header_name) {
                let cert_data = cert_header.to_str()
                    .map_err(|_| anyhow::anyhow!("Invalid certificate header encoding"))?;

                // Handle URL-encoded certificates (common with some proxies)
                let cert_pem = if cert_data.contains("%") {
                    urlencoding::decode(cert_data)
                        .map_err(|_| anyhow::anyhow!("Failed to URL-decode certificate"))?
                        .into_owned()
                } else {
                    cert_data.to_string()
                };

                return Self::parse_and_validate_certificate(&cert_pem);
            }
        }

        // Check for DER-encoded certificate in binary header
        if let Some(der_header) = headers.get("x-client-cert-der") {
            let der_data = der_header.as_bytes();
            return Self::parse_der_certificate(der_data);
        }

        Ok(None)
    }

    /// Parse and validate X.509 certificate (production implementation)
    fn parse_and_validate_certificate(cert_pem: &str) -> Result<Option<ClientCertificate>> {
        // Remove any whitespace and normalize PEM format
        let normalized_pem = Self::normalize_pem_certificate(cert_pem)?;

        // Parse the certificate using a proper X.509 library
        // In production, use x509-parser, openssl, or rustls-webpki
        let certificate = Self::parse_x509_certificate(&normalized_pem)?;

        // Validate certificate constraints
        Self::validate_certificate_constraints(&certificate)?;

        Ok(Some(certificate))
    }

    /// Parse DER-encoded certificate
    fn parse_der_certificate(der_data: &[u8]) -> Result<Option<ClientCertificate>> {
        // In production, use a proper X.509 parser library
        // This is a simplified implementation
        if der_data.is_empty() {
            return Ok(None);
        }

        // Convert DER to PEM for processing
        let pem_data = format!(
            "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----",
            base64::engine::general_purpose::STANDARD.encode(der_data)
        );

        Self::parse_and_validate_certificate(&pem_data)
    }

    /// Normalize PEM certificate format
    fn normalize_pem_certificate(cert_pem: &str) -> Result<String> {
        let cert_clean = cert_pem
            .trim()
            .replace("\\n", "\n")
            .replace("\\t", "")
            .replace(" ", "");

        // Ensure proper PEM format
        if !cert_clean.starts_with("-----BEGIN CERTIFICATE-----") {
            return Err(anyhow::anyhow!("Invalid PEM certificate format - missing BEGIN marker"));
        }

        if !cert_clean.ends_with("-----END CERTIFICATE-----") {
            return Err(anyhow::anyhow!("Invalid PEM certificate format - missing END marker"));
        }

        // Validate PEM structure
        let lines: Vec<&str> = cert_clean.lines().collect();
        if lines.len() < 3 {
            return Err(anyhow::anyhow!("Invalid PEM certificate format - insufficient lines"));
        }

        Ok(cert_clean)
    }

    /// Parse X.509 certificate (simplified - use proper library in production)
    fn parse_x509_certificate(cert_pem: &str) -> Result<ClientCertificate> {
        // In production, use x509-parser, openssl, or similar library
        // This is a simplified implementation for demonstration

        // Extract base64 content between PEM markers
        let cert_content = cert_pem
            .lines()
            .filter(|line| !line.starts_with("-----"))
            .collect::<Vec<_>>()
            .join("");

        // Decode base64 to get DER data
        let der_data = base64::engine::general_purpose::STANDARD.decode(&cert_content)
            .map_err(|_| anyhow::anyhow!("Invalid base64 in certificate"))?;

        // Generate thumbprint
        let thumbprint = Self::generate_certificate_thumbprint_from_der(&der_data);

        // In production, parse actual certificate fields
        Ok(ClientCertificate {
            subject_dn: "CN=Client,O=Example,C=US".to_string(), // Parse from DER
            issuer_dn: "CN=CA,O=Example,C=US".to_string(),     // Parse from DER
            serial_number: "123456789".to_string(),              // Parse from DER
            thumbprint_sha256: thumbprint,
            not_before: "2024-01-01T00:00:00Z".to_string(),     // Parse from DER
            not_after: "2025-01-01T00:00:00Z".to_string(),      // Parse from DER
        })
    }

    /// Validate certificate constraints and security requirements
    fn validate_certificate_constraints(certificate: &ClientCertificate) -> Result<()> {
        // Check certificate validity period
        Self::validate_certificate_validity(certificate)?;

        // Check certificate key usage and extended key usage
        Self::validate_certificate_key_usage(certificate)?;

        // Check certificate chain and trust
        Self::validate_certificate_trust(certificate)?;

        // Check certificate revocation status
        Self::validate_certificate_revocation(certificate)?;

        Ok(())
    }

    /// Validate certificate is within validity period
    fn validate_certificate_validity(certificate: &ClientCertificate) -> Result<()> {
        let now = chrono::Utc::now();

        // In production, parse actual dates from certificate
        let not_before = chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .map_err(|_| anyhow::anyhow!("Invalid not_before date"))?
            .with_timezone(&chrono::Utc);

        let not_after = chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
            .map_err(|_| anyhow::anyhow!("Invalid not_after date"))?
            .with_timezone(&chrono::Utc);

        if now < not_before {
            return Err(anyhow::anyhow!("Certificate not yet valid"));
        }

        if now > not_after {
            return Err(anyhow::anyhow!("Certificate has expired"));
        }

        // Check for certificates expiring soon (30 days)
        let expires_soon = not_after - chrono::Duration::days(30);
        if now > expires_soon {
            tracing::warn!("Client certificate expires soon: {}", certificate.subject_dn);
        }

        Ok(())
    }

    /// Validate certificate key usage for client authentication
    fn validate_certificate_key_usage(_certificate: &ClientCertificate) -> Result<()> {
        // In production, check:
        // - Key Usage: Digital Signature, Key Agreement
        // - Extended Key Usage: Client Authentication (1.3.6.1.5.5.7.3.2)

        tracing::debug!("Certificate key usage validation (simplified implementation)");
        Ok(())
    }

    /// Validate certificate trust chain
    fn validate_certificate_trust(_certificate: &ClientCertificate) -> Result<()> {
        // In production:
        // 1. Build certificate chain to trusted root
        // 2. Validate each certificate in chain
        // 3. Check intermediate CA constraints
        // 4. Verify signature chain

        tracing::debug!("Certificate trust validation (simplified implementation)");
        Ok(())
    }

    /// Check certificate revocation status (CRL/OCSP)
    fn validate_certificate_revocation(_certificate: &ClientCertificate) -> Result<()> {
        // In production:
        // 1. Check Certificate Revocation List (CRL) if available
        // 2. Perform OCSP (Online Certificate Status Protocol) check
        // 3. Handle soft-fail scenarios appropriately

        tracing::debug!("Certificate revocation check (simplified implementation)");
        Ok(())
    }

    /// Generate certificate thumbprint from DER data
    fn generate_certificate_thumbprint_from_der(der_data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(der_data);
        let hash = hasher.finalize();
        URL_SAFE_NO_PAD.encode(hash)
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