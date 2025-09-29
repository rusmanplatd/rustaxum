use anyhow::Result;
use axum::http::HeaderMap;
use base64::Engine;
use chrono::Utc;
use jsonwebtoken::DecodingKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::database::DbPool;
use crate::app::services::oauth::{ClientService, MTLSService};
use crate::app::models::oauth::Client;
use p256::elliptic_curve::sec1::FromEncodedPoint;

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
        Self::validate_jwt_client_assertion(&client, client_assertion, &claims).await?;

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
    pub fn verify_client_secret(client: &Client, provided_secret: &str) -> Result<bool> {
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

    /// Decode JWT client assertion (basic parsing without verification)
    fn decode_client_assertion_jwt(jwt: &str) -> Result<ClientAssertionClaims> {
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid JWT format"));
        }

        // Decode payload without verification (verification happens later)
        let payload_decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|_| anyhow::anyhow!("Invalid JWT payload"))?;

        let claims: ClientAssertionClaims = serde_json::from_slice(&payload_decoded)
            .map_err(|_| anyhow::anyhow!("Invalid JWT claims"))?;

        Ok(claims)
    }

    /// Extract public key from JWT header and verify signature
    fn extract_and_verify_jwt_with_public_key(jwt: &str) -> Result<ClientAssertionClaims> {
        use jsonwebtoken::Validation;

        // Parse JWT header
        let header = jsonwebtoken::decode_header(jwt)
            .map_err(|e| anyhow::anyhow!("Invalid JWT header: {}", e))?;

        // Extract public key from JWK in header (if present)
        if let Some(jwk) = header.jwk {
            let validation = Validation::new(header.alg);

            // Extract key type from JWK
            let key_type = jwk.common.key_algorithm.map(|k| format!("{:?}", k)).unwrap_or_else(|| "Unknown".to_string());

            match key_type.as_str() {
                "RSA" => {
                    return Err(anyhow::anyhow!("RSA JWK verification requires additional implementation"));
                },
                "EC" => {
                    return Err(anyhow::anyhow!("EC JWK verification requires additional implementation"));
                },
                _ => {
                    return Err(anyhow::anyhow!("Unsupported key type: {}", key_type));
                }
            }
        } else {
            return Err(anyhow::anyhow!("No public key found in JWT header"));
        }
    }

    /// Validate JWT client assertion claims and signature
    async fn validate_jwt_client_assertion(
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

        // Verify JWT signature
        Self::verify_jwt_signature(client, jwt, claims).await?;

        Ok(())
    }

    /// Verify JWT signature using proper cryptographic validation
    async fn verify_jwt_signature(
        client: &Client,
        jwt: &str,
        claims: &ClientAssertionClaims,
    ) -> Result<()> {
        use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

        // Parse JWT header to get algorithm
        let header = jsonwebtoken::decode_header(jwt)
            .map_err(|e| anyhow::anyhow!("Invalid JWT header: {}", e))?;

        // Validate algorithm - currently only HMAC algorithms are supported
        if client.secret.is_some() {
            match header.alg {
                Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {},
                _ => return Err(anyhow::anyhow!(
                    "Only HMAC algorithms (HS256, HS384, HS512) are currently supported for JWT client assertions"
                )),
            }
        } else {
            return Err(anyhow::anyhow!(
                "JWT client assertion requires client_secret for HMAC verification"
            ));
        }

        // Create validation settings
        let mut validation = Validation::new(header.alg);
        validation.set_audience(&[&claims.aud]);
        validation.set_issuer(&[&claims.iss]);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.leeway = 60; // Allow 60 seconds clock skew

        // Get appropriate key for verification based on algorithm
        let decoding_key = match header.alg {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                // HMAC verification with client secret
                let client_secret = client.secret.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Client secret required for HMAC JWT verification"))?;
                DecodingKey::from_secret(client_secret.as_bytes())
            },
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 |
            Algorithm::ES256 | Algorithm::ES384 |
            Algorithm::PS256 | Algorithm::PS384 | Algorithm::PS512 => {
                // Asymmetric verification - extract public key from JWT header or use stored key
                match Self::extract_public_key_from_jwt_header(&jwt, client).await {
                    Ok(key) => key,
                    Err(_) => {
                        // Fallback: try to load from client's public_key_pem field if available
                        if let Some(pem) = Self::get_client_public_key_pem(client) {
                            match header.alg {
                                Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 |
                                Algorithm::PS256 | Algorithm::PS384 | Algorithm::PS512 => {
                                    DecodingKey::from_rsa_pem(pem.as_bytes())
                                        .map_err(|e| anyhow::anyhow!("Invalid RSA PEM: {}", e))?
                                },
                                Algorithm::ES256 | Algorithm::ES384 => {
                                    DecodingKey::from_ec_pem(pem.as_bytes())
                                        .map_err(|e| anyhow::anyhow!("Invalid EC PEM: {}", e))?
                                },
                                _ => return Err(anyhow::anyhow!("Unsupported algorithm for stored PEM key"))
                            }
                        } else {
                            return Err(anyhow::anyhow!(
                                "JWT client assertion requires either client_secret (HMAC) or jwk in JWT header (asymmetric). Consider adding public_key_pem field to oauth_clients table."
                            ));
                        }
                    }
                }
            },
            _ => return Err(anyhow::anyhow!("Unsupported JWT algorithm: {:?}", header.alg))
        };

        // Verify the JWT signature and claims
        decode::<ClientAssertionClaims>(jwt, &decoding_key, &validation)
            .map_err(|e| anyhow::anyhow!("JWT verification failed: {}", e))?;

        tracing::debug!("JWT signature verification successful for client {}", client.id);
        Ok(())
    }

    /// Extract public key from JWT header's jwk field for asymmetric verification
    async fn extract_public_key_from_jwt_header(jwt: &str, client: &Client) -> Result<DecodingKey> {
        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

        // Parse JWT header
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid JWT format"));
        }

        let header_bytes = URL_SAFE_NO_PAD.decode(parts[0])
            .map_err(|_| anyhow::anyhow!("Invalid JWT header encoding"))?;

        let header: serde_json::Value = serde_json::from_slice(&header_bytes)
            .map_err(|_| anyhow::anyhow!("Invalid JWT header JSON"))?;

        // Extract algorithm
        let alg = header.get("alg")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing algorithm in JWT header"))?;

        // Extract JWK if present
        if let Some(jwk) = header.get("jwk") {
            return Self::jwk_to_decoding_key(jwk, alg);
        }

        // If no JWK in header, check for kid (key ID) and look up in JWKS
        if let Some(kid) = header.get("kid").and_then(|v| v.as_str()) {
            // Production: implement JWKS endpoint lookup with caching
            tracing::info!("JWKS lookup requested for key ID: {}", kid);

            // This would be implemented as:
            // 1. Check cache for JWKS by issuer
            // 2. Fetch from /.well-known/jwks.json if not cached
            // 3. Find key by kid
            // 4. Convert to DecodingKey
            // 5. Cache the result with TTL

            // Try to fetch from JWKS endpoint if client has jwks_uri configured
            if let Some(jwks_uri) = &client.jwks_uri {
                return Self::fetch_jwk_public_key(jwks_uri, kid).await;
            }

            return Err(anyhow::anyhow!(
                "No JWKS endpoint configured for client '{}' with key ID '{}'. Configure jwks_uri or public_key_pem.",
                client.id, kid
            ));
        }

        Err(anyhow::anyhow!("No public key found in JWT header"))
    }

    /// Convert JWK to DecodingKey for JWT verification
    fn jwk_to_decoding_key(jwk: &serde_json::Value, alg: &str) -> Result<DecodingKey> {
        let kty = jwk.get("kty")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing key type in JWK"))?;

        match (kty, alg) {
            ("RSA", "RS256") | ("RSA", "RS384") | ("RSA", "RS512") => {
                // Extract RSA public key parameters
                let n = jwk.get("n")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing RSA modulus in JWK"))?;
                let e = jwk.get("e")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing RSA exponent in JWK"))?;

                // Construct RSA public key from modulus and exponent
                use base64::engine::{general_purpose::URL_SAFE_NO_PAD, Engine};

                let n_bytes = URL_SAFE_NO_PAD.decode(n)
                    .map_err(|_| anyhow::anyhow!("Invalid base64 in RSA modulus"))?;
                let e_bytes = URL_SAFE_NO_PAD.decode(e)
                    .map_err(|_| anyhow::anyhow!("Invalid base64 in RSA exponent"))?;

                // Construct RSA public key from modulus and exponent
                use rsa::{RsaPublicKey, BigUint};
                use rsa::pkcs8::EncodePublicKey;

                let n_big = BigUint::from_bytes_be(&n_bytes);
                let e_big = BigUint::from_bytes_be(&e_bytes);

                let rsa_key = RsaPublicKey::new(n_big, e_big)
                    .map_err(|e| anyhow::anyhow!("Failed to construct RSA public key: {}", e))?;

                // Convert to PEM format and then to DecodingKey
                let pem = rsa_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF)
                    .map_err(|e| anyhow::anyhow!("Failed to encode RSA key to PEM: {}", e))?;

                DecodingKey::from_rsa_pem(pem.as_bytes())
                    .map_err(|e| anyhow::anyhow!("Failed to create DecodingKey from RSA PEM: {}", e))
            },
            ("EC", "ES256") | ("EC", "ES384") => {
                // Extract ECDSA public key parameters
                let crv = jwk.get("crv")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing curve in EC JWK"))?;
                let x = jwk.get("x")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing x coordinate in EC JWK"))?;
                let y = jwk.get("y")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing y coordinate in EC JWK"))?;

                // Construct ECDSA public key from curve parameters
                use base64::engine::{general_purpose::URL_SAFE_NO_PAD, Engine};

                let x_bytes = URL_SAFE_NO_PAD.decode(x)
                    .map_err(|_| anyhow::anyhow!("Invalid base64 in EC x coordinate"))?;
                let y_bytes = URL_SAFE_NO_PAD.decode(y)
                    .map_err(|_| anyhow::anyhow!("Invalid base64 in EC y coordinate"))?;

                // Construct ECDSA public key from curve parameters
                match crv {
                    "P-256" => {
                        use p256::{EncodedPoint, PublicKey};
                        use p256::pkcs8::EncodePublicKey;

                        // Construct uncompressed point (0x04 + x + y)
                        let mut point_bytes = vec![0x04];
                        point_bytes.extend_from_slice(&x_bytes);
                        point_bytes.extend_from_slice(&y_bytes);

                        let encoded_point = EncodedPoint::from_bytes(&point_bytes)
                            .map_err(|e| anyhow::anyhow!("Invalid EC point: {}", e))?;

                        let public_key = PublicKey::from_encoded_point(&encoded_point)
                            .into_option()
                            .ok_or_else(|| anyhow::anyhow!("Failed to construct P-256 public key"))?;

                        // Convert to PEM format
                        let pem = public_key.to_public_key_pem(p256::pkcs8::LineEnding::LF)
                            .map_err(|e| anyhow::anyhow!("Failed to encode EC key to PEM: {}", e))?;

                        DecodingKey::from_ec_pem(pem.as_bytes())
                            .map_err(|e| anyhow::anyhow!("Failed to create DecodingKey from EC PEM: {}", e))
                    },
                    _ => Err(anyhow::anyhow!("Unsupported EC curve: {}", crv))
                }
            },
            _ => Err(anyhow::anyhow!("Unsupported key type/algorithm combination: {}/{}", kty, alg))
        }
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

    /// Hash client secret for storage using Argon2 (production-ready)
    pub fn hash_client_secret(secret: &str) -> Result<String> {
        use argon2::{
            password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
            Argon2
        };

        // Use Argon2 for secure password hashing (OWASP recommended)
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        match argon2.hash_password(secret.as_bytes(), &salt) {
            Ok(hash) => Ok(hash.to_string()),
            Err(e) => {
                tracing::error!("Failed to hash client secret: {}", e);
                Err(anyhow::anyhow!("Secret hashing failed: {}", e))
            }
        }
    }

    /// Verify hashed client secret using Argon2 (production-ready)
    pub fn verify_hashed_secret(hashed: &str, provided: &str) -> Result<bool> {
        use argon2::{
            password_hash::{PasswordHash, PasswordVerifier},
            Argon2
        };

        // Try Argon2 hash first (modern format)
        if hashed.starts_with("$argon2") {
            let parsed_hash = PasswordHash::new(hashed)
                .map_err(|e| anyhow::anyhow!("Invalid Argon2 hash format: {}", e))?;

            let argon2 = Argon2::default();
            Ok(argon2.verify_password(provided.as_bytes(), &parsed_hash).is_ok())
        } else if let Some(hash_part) = hashed.strip_prefix("sha256:") {
            // Legacy SHA256 support for backward compatibility
            let mut hasher = Sha256::new();
            hasher.update(provided.as_bytes());
            let computed_hash = hex::encode(hasher.finalize());
            Ok(Self::constant_time_compare(hash_part.as_bytes(), computed_hash.as_bytes()))
        } else {
            // Fallback for plain text secrets (not recommended)
            tracing::warn!("Plain text client secret verification - migrate to hashed secrets");
            Ok(Self::constant_time_compare(hashed.as_bytes(), provided.as_bytes()))
        }
    }

    /// Get client public key PEM if stored in client record
    fn get_client_public_key_pem(client: &Client) -> Option<String> {
        // Access public_key_pem field from oauth_clients table
        client.public_key_pem.clone().or_else(|| {
            // Try to get from metadata as fallback
            client.metadata.get("public_key_pem")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
    }

    /// Extract algorithm from JWT header
    pub fn get_algorithm_from_jwt(jwt: &str) -> Result<String> {
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid JWT format"));
        }

        let header_decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[0])
            .map_err(|_| anyhow::anyhow!("Invalid JWT header"))?;

        let header: serde_json::Value = serde_json::from_slice(&header_decoded)
            .map_err(|_| anyhow::anyhow!("Invalid JWT header JSON"))?;

        header.get("alg")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Missing algorithm in JWT header"))
    }

    /// Header-based authentication for basic authorization
    pub async fn authenticate_client_from_header(
        pool: &DbPool,
        client_id: &str,
        auth_header: &str,
    ) -> Result<()> {
        if let Some(basic_auth) = auth_header.strip_prefix("Basic ") {
            use base64::engine::{general_purpose::STANDARD, Engine};

            let decoded = STANDARD.decode(basic_auth)
                .map_err(|_| anyhow::anyhow!("Invalid base64 in Basic auth"))?;

            let credentials = String::from_utf8(decoded)
                .map_err(|_| anyhow::anyhow!("Invalid UTF-8 in Basic auth"))?;

            let parts: Vec<&str> = credentials.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(anyhow::anyhow!("Invalid Basic auth format"));
            }

            let (provided_client_id, provided_secret) = (parts[0], parts[1]);
            if provided_client_id != client_id {
                return Err(anyhow::anyhow!("Client ID mismatch"));
            }

            // Get client from database
            let client = ClientService::find_by_id(pool, client_id.to_string())?
                .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

            // Verify client secret
            if let Some(stored_secret) = &client.secret {
                if Self::verify_hashed_secret(stored_secret, provided_secret)? {
                    return Ok(());
                }
            }

            Err(anyhow::anyhow!("Invalid client credentials"))
        } else {
            Err(anyhow::anyhow!("Only Basic authentication supported"))
        }
    }

    /// Fetch public key from JWKS endpoint
    async fn fetch_jwk_public_key(jwks_uri: &str, kid: &str) -> Result<DecodingKey> {
        // Fetch JWKS document
        let response = reqwest::get(jwks_uri).await
            .map_err(|e| anyhow::anyhow!("Failed to fetch JWKS from {}: {}", jwks_uri, e))?;

        let jwks: serde_json::Value = response.json().await
            .map_err(|e| anyhow::anyhow!("Failed to parse JWKS JSON: {}", e))?;

        // Find key with matching kid
        if let Some(keys) = jwks.get("keys").and_then(|k| k.as_array()) {
            for key in keys {
                if let Some(key_id) = key.get("kid").and_then(|k| k.as_str()) {
                    if key_id == kid {
                        // Extract public key from JWK
                        return Self::convert_jwk_to_decoding_key(key);
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Key with ID '{}' not found in JWKS endpoint {}", kid, jwks_uri))
    }

    /// Convert JWK to DecodingKey for JWT validation
    fn convert_jwk_to_decoding_key(jwk: &serde_json::Value) -> Result<DecodingKey> {
        // Extract key type and algorithm
        let kty = jwk.get("kty").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing key type (kty) in JWK"))?;

        match kty {
            "RSA" => {
                // For RSA keys, try to get PEM if available, otherwise error
                if let Some(x5c) = jwk.get("x5c").and_then(|v| v.as_array()) {
                    if let Some(cert_b64) = x5c.first().and_then(|v| v.as_str()) {
                        // Decode X.509 certificate and extract public key
                        use base64::{Engine as _, engine::general_purpose::STANDARD};
                        let cert_der = STANDARD.decode(cert_b64)
                            .map_err(|e| anyhow::anyhow!("Failed to decode x5c certificate: {}", e))?;

                        // Convert DER to PEM format
                        let pem = format!(
                            "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----",
                            STANDARD.encode(&cert_der).chars()
                                .collect::<Vec<char>>()
                                .chunks(64)
                                .map(|chunk| chunk.iter().collect::<String>())
                                .collect::<Vec<String>>()
                                .join("\n")
                        );

                        return DecodingKey::from_rsa_pem(pem.as_bytes())
                            .map_err(|e| anyhow::anyhow!("Failed to create RSA key from certificate: {}", e));
                    }
                }

                // Fallback error message
                Err(anyhow::anyhow!(
                    "RSA JWK conversion not fully implemented. Please configure public_key_pem directly in the client."
                ))
            },
            _ => Err(anyhow::anyhow!("Unsupported key type: {}", kty))
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
        // This is a simplified check - in practice, you'd pass the original JWT
        // Default to false for safety (requires client secret)
        false
    }

    /// Check if JWT uses private key algorithm based on original JWT string
    pub fn check_private_key_jwt_from_token(jwt: &str) -> Result<bool> {
        let algorithm = ClientAuthService::get_algorithm_from_jwt(jwt)?;
        Ok(matches!(algorithm.as_str(),
            "RS256" | "RS384" | "RS512" |
            "ES256" | "ES384" | "ES512" |
            "PS256" | "PS384" | "PS512"
        ))
    }

    /// Extract algorithm from JWT for validation
    pub fn get_jwt_algorithm(jwt: &str) -> Result<String> {
        ClientAuthService::get_algorithm_from_jwt(jwt)
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