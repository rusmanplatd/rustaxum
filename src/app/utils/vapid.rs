use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonwebtoken::{Algorithm, Header, EncodingKey};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// VAPID JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct VapidClaims {
    /// Audience - the origin of the push service
    pub aud: String,
    /// Expiration time (Unix timestamp)
    pub exp: u64,
    /// Subject - contact information (email or URL)
    pub sub: String,
}

/// VAPID token generator for web push authentication
#[derive(Debug)]
pub struct VapidTokenGenerator {
    private_key: String,
    public_key: String,
    subject: String,
}

impl VapidTokenGenerator {
    /// Create a new VAPID token generator
    pub fn new(private_key: String, public_key: String, subject: String) -> Self {
        Self {
            private_key,
            public_key,
            subject,
        }
    }

    /// Generate a VAPID JWT token for a specific push service endpoint
    pub fn generate_token(&self, endpoint: &str) -> Result<String> {
        // Extract the origin from the endpoint URL
        let audience = self.extract_origin(endpoint)?;

        // Create claims with 24-hour expiration
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() + 24 * 60 * 60; // 24 hours from now

        let claims = VapidClaims {
            aud: audience,
            exp,
            sub: self.subject.clone(),
        };

        // Create JWT header
        let header = Header::new(Algorithm::ES256);

        // Decode the base64 private key
        let private_key_bytes = URL_SAFE_NO_PAD.decode(&self.private_key)?;

        // Create encoding key from the private key bytes
        let encoding_key = EncodingKey::from_ec_der(&private_key_bytes);

        // Generate the JWT token
        let token = jsonwebtoken::encode(&header, &claims, &encoding_key)?;

        Ok(token)
    }

    /// Generate the complete Authorization header value
    pub fn generate_auth_header(&self, endpoint: &str) -> Result<String> {
        let token = self.generate_token(endpoint)?;
        Ok(format!("vapid t={}, k={}", token, self.public_key))
    }

    /// Extract the origin (scheme + host + port) from a URL
    fn extract_origin(&self, url: &str) -> Result<String> {
        let parsed = url::Url::parse(url)?;

        let mut origin = format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or(""));

        if let Some(port) = parsed.port() {
            // Only include port if it's not the default port for the scheme
            let default_port = match parsed.scheme() {
                "https" => 443,
                "http" => 80,
                _ => 0,
            };

            if port != default_port {
                origin = format!("{}:{}", origin, port);
            }
        }

        Ok(origin)
    }

    /// Get the public key for client-side subscription
    pub fn get_public_key(&self) -> &str {
        &self.public_key
    }

    /// Validate that the VAPID keys are properly formatted
    pub fn validate_keys(&self) -> Result<()> {
        // Try to decode both keys to ensure they're valid base64
        URL_SAFE_NO_PAD.decode(&self.private_key)
            .map_err(|_| anyhow::anyhow!("Invalid VAPID private key format"))?;

        URL_SAFE_NO_PAD.decode(&self.public_key)
            .map_err(|_| anyhow::anyhow!("Invalid VAPID public key format"))?;

        // Validate subject format (should be mailto: or https:)
        if !self.subject.starts_with("mailto:") && !self.subject.starts_with("https://") {
            return Err(anyhow::anyhow!(
                "VAPID subject must be a mailto: URI or https: URL"
            ));
        }

        Ok(())
    }
}

/// Generate new VAPID key pair for development/testing
pub fn generate_vapid_keys() -> Result<(String, String)> {
    use p256::{SecretKey, ecdsa::SigningKey};
    use p256::elliptic_curve::rand_core::OsRng;
    use p256::pkcs8::EncodePrivateKey;

    // Generate a new secret key
    let secret_key = SecretKey::random(&mut OsRng);
    let signing_key = SigningKey::from(&secret_key);
    let verifying_key = signing_key.verifying_key();

    // Convert to DER format and then base64
    let private_key_der = secret_key.to_pkcs8_der()?;
    let public_key_der = verifying_key.to_sec1_bytes();

    let private_key_b64 = URL_SAFE_NO_PAD.encode(private_key_der.as_bytes());
    let public_key_b64 = URL_SAFE_NO_PAD.encode(&public_key_der);

    Ok((private_key_b64, public_key_b64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_origin() {
        let generator = VapidTokenGenerator::new(
            "test_private".to_string(),
            "test_public".to_string(),
            "mailto:test@example.com".to_string(),
        );

        // Test HTTPS with default port
        assert_eq!(
            generator.extract_origin("https://fcm.googleapis.com/fcm/send/abc123").unwrap(),
            "https://fcm.googleapis.com"
        );

        // Test HTTP with custom port
        assert_eq!(
            generator.extract_origin("http://localhost:3000/push").unwrap(),
            "http://localhost:3000"
        );

        // Test HTTPS with custom port
        assert_eq!(
            generator.extract_origin("https://example.com:8443/push").unwrap(),
            "https://example.com:8443"
        );
    }

    #[test]
    fn test_validate_keys() {
        // Valid keys
        let generator = VapidTokenGenerator::new(
            URL_SAFE_NO_PAD.encode(b"valid_private_key_32_bytes_long!"),
            URL_SAFE_NO_PAD.encode(b"valid_public_key_65_bytes_long_for_testing_purposes_here!"),
            "mailto:test@example.com".to_string(),
        );
        assert!(generator.validate_keys().is_ok());

        // Invalid subject
        let generator = VapidTokenGenerator::new(
            URL_SAFE_NO_PAD.encode(b"valid_private_key_32_bytes_long!"),
            URL_SAFE_NO_PAD.encode(b"valid_public_key_65_bytes_long_for_testing_purposes_here!"),
            "invalid_subject".to_string(),
        );
        assert!(generator.validate_keys().is_err());
    }

    #[test]
    fn test_generate_vapid_keys() {
        let result = generate_vapid_keys();
        assert!(result.is_ok());

        let (private_key, public_key) = result.unwrap();
        assert!(!private_key.is_empty());
        assert!(!public_key.is_empty());

        // Keys should be different
        assert_ne!(private_key, public_key);
    }
}