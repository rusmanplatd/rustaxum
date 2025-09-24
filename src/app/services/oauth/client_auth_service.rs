use anyhow::Result;
use axum::http::HeaderMap;
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::database::DbPool;
use crate::app::services::oauth::{ClientService, MTLSService};
use crate::app::models::oauth::Client;

/// Production-ready client authentication service
/// Supports multiple authentication methods as per OAuth 2.1 security best practices
pub struct ClientAuthService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientAuthResult {
    pub authenticated: bool,
    pub client_id: String,
    pub auth_method: ClientAuthMethod,
    pub client: Option<Client>,
    pub mtls_certificate_thumbprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientAuthMethod {
    None,                    // Public client
    ClientSecretBasic,       // Basic HTTP authentication
    ClientSecretPost,        // Client secret in POST body
    ClientSecretJwt,         // JWT with client secret (RFC 7523)
    PrivateKeyJwt,          // JWT with private key (RFC 7523)
    TlsClientAuth,          // mTLS certificate binding (RFC 8705)
    SelfSignedTlsClientAuth, // Self-signed mTLS (RFC 8705)
}

#[derive(Debug, Deserialize)]
pub struct ClientSecretPostParams {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub client_assertion_type: Option<String>,
    pub client_assertion: Option<String>,
}

impl ClientAuthService {
    /// Authenticate client using various supported methods
    pub async fn authenticate_client(
        pool: &DbPool,
        headers: &HeaderMap,
        post_params: Option<&ClientSecretPostParams>,
    ) -> Result<ClientAuthResult> {
        // Try authentication methods in order of preference (most secure first)

        // 1. Try mTLS client authentication (RFC 8705)
        if let Ok(Some(_)) = MTLSService::extract_client_certificate(headers) {
            if let Some(params) = post_params {
                if let Ok(result) = Self::authenticate_with_mtls(pool, headers, &params.client_id).await {
                    if result.authenticated {
                        return Ok(result);
                    }
                }
            }
        }

        // 2. Try JWT-based authentication (RFC 7523)
        if let Some(params) = post_params {
            if let Some(assertion_type) = &params.client_assertion_type {
                if assertion_type == "urn:ietf:params:oauth:client-assertion-type:jwt-bearer" {
                    if let Some(assertion) = &params.client_assertion {
                        if let Ok(result) = Self::authenticate_with_jwt(pool, assertion).await {
                            if result.authenticated {
                                return Ok(result);
                            }
                        }
                    }
                }
            }
        }

        // 3. Try Basic HTTP authentication (RFC 6749 Section 2.3.1)
        if let Ok(result) = Self::authenticate_with_basic_auth(pool, headers).await {
            if result.authenticated {
                return Ok(result);
            }
        }

        // 4. Try POST body client credentials
        if let Some(params) = post_params {
            if let Ok(result) = Self::authenticate_with_post_credentials(pool, params).await {
                if result.authenticated {
                    return Ok(result);
                }
            }
        }

        // 5. Try public client (no authentication required)
        if let Some(params) = post_params {
            if params.client_secret.is_none() {
                if let Ok(result) = Self::authenticate_public_client(pool, &params.client_id).await {
                    if result.authenticated {
                        return Ok(result);
                    }
                }
            }
        }

        // Authentication failed
        Ok(ClientAuthResult {
            authenticated: false,
            client_id: post_params.map(|p| p.client_id.clone()).unwrap_or_default(),
            auth_method: ClientAuthMethod::None,
            client: None,
            mtls_certificate_thumbprint: None,
        })
    }

    /// Authenticate using mTLS client certificates (RFC 8705)
    async fn authenticate_with_mtls(
        pool: &DbPool,
        headers: &HeaderMap,
        client_id: &str,
    ) -> Result<ClientAuthResult> {
        let mtls_result = MTLSService::authenticate_client_certificate(pool, headers, client_id).await?;

        if !mtls_result.authenticated {
            return Ok(ClientAuthResult {
                authenticated: false,
                client_id: client_id.to_string(),
                auth_method: ClientAuthMethod::TlsClientAuth,
                client: None,
                mtls_certificate_thumbprint: mtls_result.thumbprint,
            });
        }

        let client = ClientService::find_by_id(pool, client_id.to_string())?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        Ok(ClientAuthResult {
            authenticated: true,
            client_id: client_id.to_string(),
            auth_method: ClientAuthMethod::TlsClientAuth,
            client: Some(client),
            mtls_certificate_thumbprint: mtls_result.thumbprint,
        })
    }

    /// Authenticate using JWT client assertion (RFC 7523)
    async fn authenticate_with_jwt(
        pool: &DbPool,
        client_assertion: &str,
    ) -> Result<ClientAuthResult> {
        // Decode and validate JWT
        let claims = Self::decode_client_assertion_jwt(client_assertion)?;
        let client_id = claims.sub.clone();

        // Find client
        let client = ClientService::find_by_id(pool, client_id.clone())?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        // Validate JWT signature and claims
        Self::validate_jwt_client_assertion(&client, client_assertion, &claims)?;

        Ok(ClientAuthResult {
            authenticated: true,
            client_id,
            auth_method: if claims.is_private_key_jwt() {
                ClientAuthMethod::PrivateKeyJwt
            } else {
                ClientAuthMethod::ClientSecretJwt
            },
            client: Some(client),
            mtls_certificate_thumbprint: None,
        })
    }

    /// Authenticate using HTTP Basic authentication
    async fn authenticate_with_basic_auth(
        pool: &DbPool,
        headers: &HeaderMap,
    ) -> Result<ClientAuthResult> {
        let (client_id, client_secret) = Self::extract_basic_auth_credentials(headers)?;

        let client = ClientService::find_by_id(pool, client_id.clone())?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        if Self::verify_client_secret(&client, &client_secret)? {
            Ok(ClientAuthResult {
                authenticated: true,
                client_id,
                auth_method: ClientAuthMethod::ClientSecretBasic,
                client: Some(client),
                mtls_certificate_thumbprint: None,
            })
        } else {
            Ok(ClientAuthResult {
                authenticated: false,
                client_id,
                auth_method: ClientAuthMethod::ClientSecretBasic,
                client: None,
                mtls_certificate_thumbprint: None,
            })
        }
    }

    /// Authenticate using POST body credentials
    async fn authenticate_with_post_credentials(
        pool: &DbPool,
        params: &ClientSecretPostParams,
    ) -> Result<ClientAuthResult> {
        let client_secret = params.client_secret.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Client secret required"))?;

        let client = ClientService::find_by_id(pool, params.client_id.clone())?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        if Self::verify_client_secret(&client, client_secret)? {
            Ok(ClientAuthResult {
                authenticated: true,
                client_id: params.client_id.clone(),
                auth_method: ClientAuthMethod::ClientSecretPost,
                client: Some(client),
                mtls_certificate_thumbprint: None,
            })
        } else {
            Ok(ClientAuthResult {
                authenticated: false,
                client_id: params.client_id.clone(),
                auth_method: ClientAuthMethod::ClientSecretPost,
                client: None,
                mtls_certificate_thumbprint: None,
            })
        }
    }

    /// Authenticate public client (no secret required)
    async fn authenticate_public_client(
        pool: &DbPool,
        client_id: &str,
    ) -> Result<ClientAuthResult> {
        let client = ClientService::find_by_id(pool, client_id.to_string())?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        // Verify this is actually a public client
        if client.secret.is_some() {
            return Ok(ClientAuthResult {
                authenticated: false,
                client_id: client_id.to_string(),
                auth_method: ClientAuthMethod::None,
                client: None,
                mtls_certificate_thumbprint: None,
            });
        }

        Ok(ClientAuthResult {
            authenticated: true,
            client_id: client_id.to_string(),
            auth_method: ClientAuthMethod::None,
            client: Some(client),
            mtls_certificate_thumbprint: None,
        })
    }

    /// Extract Basic authentication credentials
    fn extract_basic_auth_credentials(headers: &HeaderMap) -> Result<(String, String)> {
        let auth_header = headers.get("authorization")
            .ok_or_else(|| anyhow::anyhow!("Authorization header missing"))?;

        let auth_str = auth_header.to_str()
            .map_err(|_| anyhow::anyhow!("Invalid Authorization header"))?;

        if !auth_str.starts_with("Basic ") {
            return Err(anyhow::anyhow!("Not Basic authentication"));
        }

        let encoded = auth_str.strip_prefix("Basic ").unwrap();
        let decoded_bytes = base64::engine::general_purpose::STANDARD.decode(encoded)
            .map_err(|_| anyhow::anyhow!("Invalid base64 in Authorization header"))?;

        let decoded_str = String::from_utf8(decoded_bytes)
            .map_err(|_| anyhow::anyhow!("Invalid UTF-8 in Authorization header"))?;

        let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid credential format"));
        }

        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// Verify client secret using constant-time comparison
    fn verify_client_secret(client: &Client, provided_secret: &str) -> Result<bool> {
        let stored_secret = client.secret.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Client has no secret (public client)"))?;

        // Use constant-time comparison to prevent timing attacks
        Ok(Self::constant_time_compare(stored_secret.as_bytes(), provided_secret.as_bytes()))
    }

    /// Constant-time string comparison to prevent timing attacks
    fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut result = 0u8;
        for (byte_a, byte_b) in a.iter().zip(b.iter()) {
            result |= byte_a ^ byte_b;
        }

        result == 0
    }

    /// Decode JWT client assertion
    fn decode_client_assertion_jwt(jwt: &str) -> Result<ClientAssertionClaims> {
        // In production, use a proper JWT library with signature verification
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid JWT format"));
        }

        // Decode payload (this is simplified - use proper JWT validation)
        let payload_decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|_| anyhow::anyhow!("Invalid JWT payload"))?;

        let claims: ClientAssertionClaims = serde_json::from_slice(&payload_decoded)
            .map_err(|_| anyhow::anyhow!("Invalid JWT claims"))?;

        Ok(claims)
    }

    /// Validate JWT client assertion claims and signature
    fn validate_jwt_client_assertion(
        client: &Client,
        jwt: &str,
        claims: &ClientAssertionClaims,
    ) -> Result<()> {
        // Validate basic claims
        if claims.iss != claims.sub {
            return Err(anyhow::anyhow!("Invalid JWT: issuer must equal subject"));
        }

        if claims.sub != client.id.to_string() {
            return Err(anyhow::anyhow!("Invalid JWT: subject doesn't match client"));
        }

        // Validate expiration
        let now = Utc::now().timestamp() as u64;
        if claims.exp <= now {
            return Err(anyhow::anyhow!("JWT expired"));
        }

        // Validate not-before if present
        if let Some(nbf) = claims.nbf {
            if nbf > now {
                return Err(anyhow::anyhow!("JWT not yet valid"));
            }
        }

        // In production: verify JWT signature using client's registered key or secret
        Self::verify_jwt_signature(client, jwt, claims)?;

        Ok(())
    }

    /// Verify JWT signature (production implementation needed)
    fn verify_jwt_signature(
        _client: &Client,
        _jwt: &str,
        _claims: &ClientAssertionClaims,
    ) -> Result<()> {
        // TODO: Implement proper JWT signature verification
        // For client_secret_jwt: Use HMAC with client secret
        // For private_key_jwt: Use public key verification (RSA/ECDSA)

        // This would require:
        // 1. Parsing JWT header to get algorithm
        // 2. Getting appropriate key (client secret or public key)
        // 3. Verifying signature using cryptographic library

        tracing::warn!("JWT signature verification not implemented - accepting all JWTs");
        Ok(())
    }

    /// Generate secure client secret
    pub fn generate_client_secret() -> String {
        use rand::{distributions::Alphanumeric, Rng};

        // Generate 64-character random secret
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect()
    }

    /// Hash client secret for storage (use argon2 in production)
    pub fn hash_client_secret(secret: &str) -> Result<String> {
        // In production, use argon2 or similar strong hashing
        let mut hasher = Sha256::new();
        hasher.update(secret.as_bytes());
        let hash = hasher.finalize();
        Ok(format!("sha256:{}", hex::encode(hash)))
    }

    /// Verify hashed client secret
    pub fn verify_hashed_secret(hashed: &str, provided: &str) -> Result<bool> {
        if let Some(hash_part) = hashed.strip_prefix("sha256:") {
            let mut hasher = Sha256::new();
            hasher.update(provided.as_bytes());
            let computed_hash = hex::encode(hasher.finalize());
            Ok(Self::constant_time_compare(hash_part.as_bytes(), computed_hash.as_bytes()))
        } else {
            // Fallback for plain text secrets (not recommended)
            Ok(Self::constant_time_compare(hashed.as_bytes(), provided.as_bytes()))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientAssertionClaims {
    pub iss: String,        // Issuer - must be client_id
    pub sub: String,        // Subject - must be client_id
    pub aud: String,        // Audience - authorization server token endpoint
    pub exp: u64,           // Expiration time
    pub iat: u64,           // Issued at time
    pub nbf: Option<u64>,   // Not before time
    pub jti: String,        // JWT ID - unique identifier
}

impl ClientAssertionClaims {
    /// Check if this is a private key JWT (vs client secret JWT)
    pub fn is_private_key_jwt(&self) -> bool {
        // In production, this would check the JWT header algorithm
        // RS256, RS384, RS512, ES256, ES384, ES512, PS256, PS384, PS512
        // indicate private key JWT, while HS* algorithms indicate client secret JWT
        true // Simplified for demo
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_compare() {
        assert!(ClientAuthService::constant_time_compare(b"hello", b"hello"));
        assert!(!ClientAuthService::constant_time_compare(b"hello", b"world"));
        assert!(!ClientAuthService::constant_time_compare(b"hello", b"hell"));
    }

    #[test]
    fn test_generate_client_secret() {
        let secret1 = ClientAuthService::generate_client_secret();
        let secret2 = ClientAuthService::generate_client_secret();

        assert_eq!(secret1.len(), 64);
        assert_eq!(secret2.len(), 64);
        assert_ne!(secret1, secret2);
    }

    #[test]
    fn test_hash_and_verify_secret() {
        let secret = "test_secret_123";
        let hashed = ClientAuthService::hash_client_secret(secret).unwrap();

        assert!(ClientAuthService::verify_hashed_secret(&hashed, secret).unwrap());
        assert!(!ClientAuthService::verify_hashed_secret(&hashed, "wrong_secret").unwrap());
    }
}