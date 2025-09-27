use anyhow::Result;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use crate::database::DbPool;

/// RFC 9449: OAuth 2.0 Demonstrating Proof of Possession (DPoP)
/// Service for handling DPoP token validation and management

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DPoPClaims {
    /// JWT ID - unique identifier for the DPoP proof
    pub jti: String,
    /// HTTP method for the request being made
    pub htm: String,
    /// HTTP URL for the request being made (without query and fragment)
    pub htu: String,
    /// Issued at time (Unix timestamp)
    pub iat: u64,
    /// Optional access token hash (for binding with access token)
    pub ath: Option<String>,
    /// Optional nonce (set by server in error responses)
    pub nonce: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DPoPJWK {
    /// Key type (always "EC" for ECDSA or "RSA")
    pub kty: String,
    /// Algorithm (e.g., "ES256", "RS256")
    pub alg: Option<String>,
    /// Curve (for ECDSA keys, e.g., "P-256")
    pub crv: Option<String>,
    /// X coordinate (for ECDSA keys, base64url-encoded)
    pub x: Option<String>,
    /// Y coordinate (for ECDSA keys, base64url-encoded)
    pub y: Option<String>,
    /// Modulus (for RSA keys, base64url-encoded)
    pub n: Option<String>,
    /// Exponent (for RSA keys, base64url-encoded)
    pub e: Option<String>,
    /// Key use (optional, typically "sig" for signature)
    pub r#use: Option<String>,
    /// Key ID (optional)
    pub kid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DPoPHeader {
    /// Algorithm used for signing
    pub alg: String,
    /// Token type (always "dpop+jwt")
    pub typ: String,
    /// JSON Web Key containing the public key
    pub jwk: DPoPJWK,
}

pub struct DPoPService;

impl DPoPService {
    /// RFC 9449: Validate DPoP proof JWT
    /// Returns the JWK thumbprint for token binding
    pub fn validate_dpop_proof(
        dpop_proof: &str,
        http_method: &str,
        http_url: &str,
        access_token_hash: Option<&str>,
        expected_nonce: Option<&str>,
    ) -> Result<String> {
        // Decode the DPoP proof without verification first to get the JWK
        let header = jsonwebtoken::decode_header(dpop_proof)?;

        // Validate header type
        if header.typ != Some("dpop+jwt".to_string()) {
            return Err(anyhow::anyhow!("Invalid DPoP token type"));
        }

        // Extract JWK from header (custom parsing needed)
        let dpop_header: DPoPHeader = Self::parse_dpop_header(dpop_proof)?;

        // Create public key from JWK
        let decoding_key = Self::jwk_to_decoding_key(&dpop_header.jwk)?;

        // Set up validation
        let mut validation = Validation::new(match dpop_header.alg.as_str() {
            "ES256" => Algorithm::ES256,
            "ES384" => Algorithm::ES384,
            "RS256" => Algorithm::RS256,
            "RS384" => Algorithm::RS384,
            "RS512" => Algorithm::RS512,
            // Note: ES512 is not supported by the jsonwebtoken crate at this time
            _ => return Err(anyhow::anyhow!("Unsupported DPoP algorithm: {}", dpop_header.alg)),
        });

        // Disable default validations - we'll do them manually
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.validate_aud = false;
        validation.set_issuer(&[] as &[String]);

        // Decode and validate the DPoP proof
        let token_data = decode::<DPoPClaims>(dpop_proof, &decoding_key, &validation)?;
        let claims = token_data.claims;

        // RFC 9449 Validations
        Self::validate_dpop_claims(&claims, http_method, http_url, access_token_hash, expected_nonce)?;

        // Calculate JWK thumbprint for token binding
        let jwk_thumbprint = Self::calculate_jwk_thumbprint(&dpop_header.jwk)?;

        Ok(jwk_thumbprint)
    }

    /// Validate DPoP claims according to RFC 9449
    fn validate_dpop_claims(
        claims: &DPoPClaims,
        http_method: &str,
        http_url: &str,
        access_token_hash: Option<&str>,
        expected_nonce: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now().timestamp() as u64;

        // 1. Check token freshness (iat must be recent)
        let max_age = 60; // 60 seconds maximum age
        if claims.iat > now {
            return Err(anyhow::anyhow!("DPoP token issued in the future"));
        }
        if now - claims.iat > max_age {
            return Err(anyhow::anyhow!("DPoP token too old"));
        }

        // 2. Validate HTTP method
        if claims.htm != http_method {
            return Err(anyhow::anyhow!(
                "DPoP HTTP method mismatch: expected {}, got {}",
                http_method,
                claims.htm
            ));
        }

        // 3. Validate HTTP URL (without query and fragment)
        let normalized_url = Self::normalize_url(http_url)?;
        if claims.htu != normalized_url {
            return Err(anyhow::anyhow!(
                "DPoP HTTP URL mismatch: expected {}, got {}",
                normalized_url,
                claims.htu
            ));
        }

        // 4. Validate access token hash if provided
        if let (Some(expected_hash), Some(actual_hash)) = (access_token_hash, &claims.ath) {
            if expected_hash != actual_hash {
                return Err(anyhow::anyhow!("DPoP access token hash mismatch"));
            }
        }

        // 5. Validate nonce if expected
        if let (Some(expected_nonce), Some(actual_nonce)) = (expected_nonce, &claims.nonce) {
            if expected_nonce != actual_nonce {
                return Err(anyhow::anyhow!("DPoP nonce mismatch"));
            }
        } else if expected_nonce.is_some() && claims.nonce.is_none() {
            return Err(anyhow::anyhow!("DPoP nonce required but not provided"));
        }

        // 6. JTI uniqueness check would go here (requires storage)
        // This is implementation-specific and may use Redis or database

        Ok(())
    }

    /// Calculate access token hash for DPoP binding
    pub fn calculate_access_token_hash(access_token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(access_token.as_bytes());
        let hash = hasher.finalize();
        URL_SAFE_NO_PAD.encode(hash)
    }

    /// Calculate JWK thumbprint according to RFC 7638
    fn calculate_jwk_thumbprint(jwk: &DPoPJWK) -> Result<String> {
        // Create canonical JWK representation for thumbprint
        let canonical_jwk = match jwk.kty.as_str() {
            "EC" => {
                // For ECDSA keys: {"crv":"P-256","kty":"EC","x":"...","y":"..."}
                serde_json::json!({
                    "crv": jwk.crv.as_ref().ok_or_else(|| anyhow::anyhow!("Missing curve for EC key"))?,
                    "kty": "EC",
                    "x": jwk.x.as_ref().ok_or_else(|| anyhow::anyhow!("Missing x coordinate for EC key"))?,
                    "y": jwk.y.as_ref().ok_or_else(|| anyhow::anyhow!("Missing y coordinate for EC key"))?,
                })
            },
            "RSA" => {
                // For RSA keys: {"e":"AQAB","kty":"RSA","n":"..."}
                serde_json::json!({
                    "e": jwk.e.as_ref().ok_or_else(|| anyhow::anyhow!("Missing exponent for RSA key"))?,
                    "kty": "RSA",
                    "n": jwk.n.as_ref().ok_or_else(|| anyhow::anyhow!("Missing modulus for RSA key"))?,
                })
            },
            _ => return Err(anyhow::anyhow!("Unsupported key type: {}", jwk.kty)),
        };

        // Calculate SHA-256 hash of canonical JSON
        let canonical_json = serde_json::to_string(&canonical_jwk)?;
        let mut hasher = Sha256::new();
        hasher.update(canonical_json.as_bytes());
        let hash = hasher.finalize();

        Ok(URL_SAFE_NO_PAD.encode(hash))
    }

    /// Parse DPoP header to extract JWK using proper JWT validation
    fn parse_dpop_header(dpop_proof: &str) -> Result<DPoPHeader> {
        use jsonwebtoken::decode_header;

        // Use jsonwebtoken library to properly decode the header
        let header = decode_header(dpop_proof)
            .map_err(|e| anyhow::anyhow!("Failed to decode DPoP JWT header: {}", e))?;

        // Extract JWK from header
        let jwk = header.jwk
            .ok_or_else(|| anyhow::anyhow!("DPoP JWT header must contain jwk field"))?;

        // Extract fields from JWK common parameters
        let kty = jwk.common.key_algorithm.map(|k| format!("{:?}", k)).unwrap_or_else(|| "Unknown".to_string());
        let kid = jwk.common.key_id.clone();
        let key_use = jwk.common.public_key_use.clone();

        // Create simplified JWK structure for DPoP
        let dpop_jwk = DPoPJWK {
            kty,
            alg: Some(format!("{:?}", header.alg)),
            r#use: key_use.map(|u| format!("{:?}", u)),
            kid,
            n: None, // Would need to extract from algorithm-specific parameters
            e: None,
            x: None,
            y: None,
            crv: None,
        };

        Ok(DPoPHeader {
            typ: header.typ.unwrap_or_else(|| "dpop+jwt".to_string()),
            alg: format!("{:?}", header.alg),
            jwk: dpop_jwk,
        })
    }

    /// Convert JWK to decoding key
    fn jwk_to_decoding_key(jwk: &DPoPJWK) -> Result<DecodingKey> {
        use jsonwebtoken::DecodingKey;

        match jwk.kty.as_str() {
            "RSA" => {
                // For RSA keys, construct the public key from n and e parameters
                if let (Some(n), Some(e)) = (&jwk.n, &jwk.e) {
                    // Decode base64url-encoded RSA parameters
                    let n_bytes = URL_SAFE_NO_PAD.decode(n)
                        .map_err(|_| anyhow::anyhow!("Invalid RSA modulus encoding"))?;
                    let e_bytes = URL_SAFE_NO_PAD.decode(e)
                        .map_err(|_| anyhow::anyhow!("Invalid RSA exponent encoding"))?;

                    // Convert to RSA public key using rsa crate
                    use rsa::{RsaPublicKey, BigUint};
                    let n_bigint = BigUint::from_bytes_be(&n_bytes);
                    let e_bigint = BigUint::from_bytes_be(&e_bytes);

                    let rsa_key = RsaPublicKey::new(n_bigint, e_bigint)
                        .map_err(|e| anyhow::anyhow!("Failed to create RSA public key: {}", e))?;

                    // Convert to PEM format for jsonwebtoken
                    use rsa::pkcs8::EncodePublicKey;
                    let pem = rsa_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF)
                        .map_err(|e| anyhow::anyhow!("Failed to encode RSA key to PEM: {}", e))?;

                    DecodingKey::from_rsa_pem(pem.as_bytes())
                        .map_err(|e| anyhow::anyhow!("Failed to create DecodingKey from RSA PEM: {}", e))
                } else {
                    Err(anyhow::anyhow!("RSA JWK missing required parameters n and e"))
                }
            },
            "EC" => {
                // For ECDSA keys, implement proper curve-based key construction
                // This requires additional ECDSA support which may need p256 or similar crates
                Err(anyhow::anyhow!("ECDSA JWK support requires additional implementation with curve-specific libraries"))
            },
            _ => Err(anyhow::anyhow!("Unsupported key type: {}", jwk.kty)),
        }
    }

    /// Normalize URL by removing query and fragment components
    fn normalize_url(url: &str) -> Result<String> {
        let parsed_url = url::Url::parse(url)?;

        let normalized = format!(
            "{}://{}{}",
            parsed_url.scheme(),
            parsed_url.host_str().ok_or_else(|| anyhow::anyhow!("No host in URL"))?,
            parsed_url.path()
        );

        Ok(normalized)
    }

    /// Generate a DPoP nonce for use in error responses
    pub fn generate_nonce() -> String {
        use rand::Rng;
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();

        (0..32)
            .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
            .collect()
    }

    /// Create DPoP-bound access token response
    pub fn create_dpop_bound_token_response(
        access_token: String,
        _jwk_thumbprint: String,
        expires_in: i64,
        refresh_token: Option<String>,
        scope: String,
    ) -> serde_json::Value {
        serde_json::json!({
            "access_token": access_token,
            "token_type": "DPoP", // RFC 9449: Token type is "DPoP" not "Bearer"
            "expires_in": expires_in,
            "refresh_token": refresh_token,
            "scope": scope,
            // Additional DPoP-specific fields could go here
        })
    }

    /// Validate DPoP-bound token during resource access
    pub fn validate_dpop_bound_token(
        dpop_proof: &str,
        access_token: &str,
        http_method: &str,
        http_url: &str,
        expected_jwk_thumbprint: &str,
    ) -> Result<()> {
        // Calculate access token hash
        let access_token_hash = Self::calculate_access_token_hash(access_token);

        // Validate DPoP proof
        let jwk_thumbprint = Self::validate_dpop_proof(
            dpop_proof,
            http_method,
            http_url,
            Some(&access_token_hash),
            None, // No nonce expected during normal validation
        )?;

        // Verify JWK thumbprint matches the one bound to the token
        if jwk_thumbprint != expected_jwk_thumbprint {
            return Err(anyhow::anyhow!("DPoP JWK thumbprint mismatch"));
        }

        Ok(())
    }

    /// Store JTI for replay protection using Redis
    pub async fn store_jti_for_replay_protection(
        _pool: &DbPool,
        jti: &str,
        expiry: chrono::DateTime<Utc>,
    ) -> Result<()> {
        // Use Redis for JTI storage with TTL for automatic cleanup
        use redis::{AsyncCommands, Client};

        // Get Redis URL from environment or use default
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        // Connect to Redis
        let client = Client::open(redis_url)
            .map_err(|e| anyhow::anyhow!("Failed to connect to Redis: {}", e))?;

        let mut conn = client.get_multiplexed_async_connection()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get Redis connection: {}", e))?;

        // Calculate TTL in seconds
        let now = chrono::Utc::now();
        let ttl_seconds = if expiry > now {
            (expiry - now).num_seconds() as u64
        } else {
            3600 // Default 1 hour TTL if expiry is in the past
        };

        // Store JTI with TTL
        let key = format!("dpop:jti:{}", jti);
        let _: () = conn.set_ex(&key, "1", ttl_seconds)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to store JTI in Redis: {}", e))?;

        tracing::debug!("Stored JTI {} for replay protection with TTL {}s", jti, ttl_seconds);
        Ok(())
    }

    /// Check JTI for replay attacks using Redis
    pub async fn check_jti_replay(
        _pool: &DbPool,
        jti: &str,
    ) -> Result<bool> {
        use redis::{AsyncCommands, Client};

        // Get Redis URL from environment or use default
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        // Connect to Redis
        let client = Client::open(redis_url)
            .map_err(|e| anyhow::anyhow!("Failed to connect to Redis: {}", e))?;

        let mut conn = client.get_multiplexed_async_connection()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get Redis connection: {}", e))?;

        // Check if JTI exists
        let key = format!("dpop:jti:{}", jti);
        let exists: bool = conn.exists(&key)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check JTI in Redis: {}", e))?;

        if exists {
            tracing::warn!("JTI {} has been used before - potential replay attack", jti);
        }

        Ok(exists)
    }
}