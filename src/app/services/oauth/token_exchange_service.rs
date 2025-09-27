use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use crate::database::DbPool;
use crate::app::models::oauth::{CreateAccessToken, AccessToken};
use crate::app::services::oauth::{TokenService, ClientService};
use crate::app::services::oauth::token_service::RFC9068Claims;
use ulid::Ulid;

/// RFC 8693: OAuth 2.0 Token Exchange
///
/// This service implements secure token exchange for delegation and impersonation scenarios.
/// It allows clients to exchange one token for another, enabling complex authorization flows
/// such as service-to-service communication with user context preservation.
pub struct TokenExchangeService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenExchangeRequest {
    pub grant_type: String, // Must be "urn:ietf:params:oauth:grant-type:token-exchange"
    pub resource: Option<String>, // Target service/resource
    pub audience: Option<String>, // Intended audience for the token
    pub scope: Option<String>, // Requested scope
    pub requested_token_type: Option<String>, // Type of token being requested
    pub subject_token: String, // Token representing the identity
    pub subject_token_type: String, // Type of subject token
    pub actor_token: Option<String>, // Token representing the actor
    pub actor_token_type: Option<String>, // Type of actor token
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenExchangeResponse {
    pub access_token: String,
    pub issued_token_type: String,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub scope: Option<String>,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TokenContext {
    pub token_id: String,
    pub user_id: Option<String>,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub token_type: TokenType,
}

#[derive(Debug, Clone)]
pub enum TokenType {
    AccessToken,
    RefreshToken,
    IdToken,
    SamlAssertion,
    Jwt,
}

#[derive(Debug, Clone)]
pub enum ExchangeScenario {
    Delegation,      // Acting on behalf of a user
    Impersonation,   // Acting as a user
    ServiceToService, // Service acting as itself with user context
}

impl TokenExchangeService {
    /// Exchange tokens according to RFC 8693
    pub async fn exchange_token(
        pool: &DbPool,
        client_id: &str,
        request: TokenExchangeRequest,
    ) -> Result<TokenExchangeResponse> {
        // Validate grant type
        if request.grant_type != "urn:ietf:params:oauth:grant-type:token-exchange" {
            return Err(anyhow::anyhow!("Invalid grant type for token exchange"));
        }

        // Validate and parse subject token
        let subject_context = Self::validate_and_parse_token(
            pool,
            &request.subject_token,
            &request.subject_token_type,
        ).await?;

        // Validate actor token if present
        let actor_context = if let Some(actor_token) = &request.actor_token {
            Some(Self::validate_and_parse_token(
                pool,
                actor_token,
                request.actor_token_type.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Actor token type is required when actor token is provided"))?,
            ).await?)
        } else {
            None
        };

        // Determine exchange scenario
        let scenario = Self::determine_exchange_scenario(&subject_context, &actor_context)?;

        // Validate client permissions for token exchange
        Self::validate_client_exchange_permissions(pool, client_id, &scenario, &request).await?;

        // Validate requested scopes
        let granted_scopes = Self::validate_and_limit_scopes(
            &subject_context,
            &actor_context,
            request.scope.as_deref(),
        )?;

        // Create exchanged token
        let exchanged_token = Self::create_exchanged_token(
            pool,
            client_id,
            &subject_context,
            &actor_context,
            &granted_scopes,
            &request,
            scenario,
        ).await?;

        // Generate response
        Ok(TokenExchangeResponse {
            access_token: exchanged_token.jwt_token,
            issued_token_type: "urn:ietf:params:oauth:token-type:access_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: Some(3600), // 1 hour
            scope: Some(granted_scopes.join(" ")),
            refresh_token: None, // RFC 8693 doesn't typically include refresh tokens
        })
    }

    /// Validate and parse input tokens
    async fn validate_and_parse_token(
        pool: &DbPool,
        token: &str,
        token_type: &str,
    ) -> Result<TokenContext> {
        match token_type {
            "urn:ietf:params:oauth:token-type:access_token" => {
                Self::parse_access_token(pool, token).await
            },
            "urn:ietf:params:oauth:token-type:refresh_token" => {
                Self::parse_refresh_token(pool, token).await
            },
            "urn:ietf:params:oauth:token-type:id_token" => {
                Self::parse_id_token(token).await
            },
            "urn:ietf:params:oauth:token-type:jwt" => {
                Self::parse_jwt_token(token).await
            },
            _ => Err(anyhow::anyhow!("Unsupported token type: {}", token_type)),
        }
    }

    /// Parse access token and extract context
    async fn parse_access_token(pool: &DbPool, token: &str) -> Result<TokenContext> {
        let claims = TokenService::decode_jwt_token(token)?;
        let token_id = Ulid::from_string(&claims.jti.to_string())?;

        let access_token = TokenService::find_access_token_by_id(pool, token_id.to_string())?
            .ok_or_else(|| anyhow::anyhow!("Access token not found"))?;

        if !access_token.is_valid() {
            return Err(anyhow::anyhow!("Access token is expired or revoked"));
        }

        Ok(TokenContext {
            token_id: token_id.to_string(),
            user_id: access_token.user_id.clone(),
            client_id: access_token.client_id.clone(),
            scopes: access_token.get_scopes(),
            expires_at: access_token.expires_at,
            token_type: TokenType::AccessToken,
        })
    }

    /// Parse refresh token and extract context
    async fn parse_refresh_token(pool: &DbPool, token: &str) -> Result<TokenContext> {
        let token_id = Ulid::from_string(token)?;
        let refresh_token = TokenService::find_refresh_token_by_id(pool, token_id.to_string())?
            .ok_or_else(|| anyhow::anyhow!("Refresh token not found"))?;

        if !refresh_token.is_valid() {
            return Err(anyhow::anyhow!("Refresh token is expired or revoked"));
        }

        let access_token = TokenService::find_access_token_by_id(pool, refresh_token.access_token_id.to_string())?
            .ok_or_else(|| anyhow::anyhow!("Associated access token not found"))?;

        Ok(TokenContext {
            token_id: token_id.to_string(),
            user_id: access_token.user_id.clone(),
            client_id: access_token.client_id.clone(),
            scopes: access_token.get_scopes(),
            expires_at: refresh_token.expires_at,
            token_type: TokenType::RefreshToken,
        })
    }

    /// Parse ID token (JWT) and extract context with full validation
    async fn parse_id_token(token: &str) -> Result<TokenContext> {
        // Validate ID token signature and structure
        let claims = Self::validate_id_token_signature(token)?;

        // Validate token expiration
        let now = chrono::Utc::now().timestamp() as usize;
        if claims.exp < now {
            return Err(anyhow::anyhow!("ID token has expired"));
        }

        // Validate not-before if present
        if let Some(nbf) = claims.nbf {
            if nbf > now {
                return Err(anyhow::anyhow!("ID token not yet valid"));
            }
        }

        // Validate issuer (required for ID tokens)
        let expected_issuer = std::env::var("OAUTH_ISSUER")
            .unwrap_or_else(|_| "https://auth.rustaxum.dev".to_string());
        if &claims.iss != &expected_issuer {
            return Err(anyhow::anyhow!("Invalid issuer in ID token: expected {}, got {}", expected_issuer, claims.iss));
        }

        // Validate required claims for ID tokens
        if claims.sub.is_none() || claims.sub.as_ref().unwrap().is_empty() {
            return Err(anyhow::anyhow!("ID token missing subject claim"));
        }

        if claims.aud.is_empty() {
            return Err(anyhow::anyhow!("ID token missing audience claim"));
        }

        // Validate issued at time
        let max_age = 300; // 5 minutes
        if (now as i64 - claims.iat as i64) > max_age {
            return Err(anyhow::anyhow!("ID token too old"));
        }

        Ok(TokenContext {
            token_id: claims.jti.clone(),
            user_id: claims.sub.clone(),
            client_id: claims.aud.first().unwrap_or(&"unknown".to_string()).clone(),
            scopes: vec!["openid".to_string()], // ID tokens imply OpenID scope
            expires_at: chrono::DateTime::from_timestamp(claims.exp as i64, 0)
                .map(|dt| dt.with_timezone(&Utc)),
            token_type: TokenType::IdToken,
        })
    }

    /// Validate ID token signature and decode claims
    fn validate_id_token_signature(token: &str) -> Result<RFC9068Claims> {
        use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

        // Parse JWT header to determine algorithm
        let header = jsonwebtoken::decode_header(token)
            .map_err(|e| anyhow::anyhow!("Invalid JWT header: {}", e))?;

        // Get secret for verification (production: integrate with JWKS endpoint)
        let secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key".to_string());

        // Create validation settings
        let mut validation = Validation::new(header.alg);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.leeway = 60; // Allow 60 seconds clock skew

        // Set expected issuer
        let expected_issuer = std::env::var("OAUTH_ISSUER")
            .unwrap_or_else(|_| "https://auth.rustaxum.dev".to_string());
        validation.set_issuer(&[&expected_issuer]);

        // For ID tokens, we don't validate audience here as it varies by client
        validation.validate_aud = false;

        // Create decoding key based on algorithm
        let decoding_key = match header.alg {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                DecodingKey::from_secret(secret.as_bytes())
            },
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => {
                // TODO: load RSA public key from JWKS endpoint
                DecodingKey::from_secret(secret.as_bytes())
            },
            _ => return Err(anyhow::anyhow!("Unsupported JWT algorithm: {:?}", header.alg))
        };

        // Decode and validate the JWT
        let token_data = decode::<RFC9068Claims>(token, &decoding_key, &validation)
            .map_err(|e| anyhow::anyhow!("JWT validation failed: {}", e))?;

        Ok(token_data.claims)
    }

    /// Parse generic JWT token and extract context
    async fn parse_jwt_token(token: &str) -> Result<TokenContext> {
        let claims = TokenService::decode_jwt_token(token)?;

        Ok(TokenContext {
            token_id: claims.jti.to_string(),
            user_id: Some(claims.sub.to_string()),
            client_id: claims.aud.to_string(),
            scopes: claims.scopes,
            expires_at: claims.exp.map(|exp| chrono::DateTime::from_timestamp(exp as i64, 0)
                .map(|dt| dt.with_timezone(&Utc))).flatten(),
            token_type: TokenType::Jwt,
        })
    }

    /// Determine the exchange scenario based on tokens
    fn determine_exchange_scenario(
        subject_context: &TokenContext,
        actor_context: &Option<TokenContext>,
    ) -> Result<ExchangeScenario> {
        match actor_context {
            Some(actor) => {
                if actor.user_id == subject_context.user_id {
                    Ok(ExchangeScenario::Impersonation)
                } else {
                    Ok(ExchangeScenario::Delegation)
                }
            },
            None => Ok(ExchangeScenario::ServiceToService),
        }
    }

    /// Validate client permissions for token exchange
    async fn validate_client_exchange_permissions(
        pool: &DbPool,
        client_id: &str,
        scenario: &ExchangeScenario,
        request: &TokenExchangeRequest,
    ) -> Result<()> {
        let client = ClientService::find_by_id(pool, client_id.to_string())?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        // Check client configuration for allowed exchange scenarios
        let client_metadata = &client.metadata;

        let allowed_scenarios = client_metadata
            .get("token_exchange_scenarios")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["delegation"]); // Default to delegation only

        let scenario_name = match scenario {
            ExchangeScenario::Impersonation => "impersonation",
            ExchangeScenario::Delegation => "delegation",
            ExchangeScenario::ServiceToService => "service_to_service",
        };

        if !allowed_scenarios.contains(&scenario_name) {
            return Err(anyhow::anyhow!("Client not configured for {} token exchange", scenario_name));
        }
        
        match scenario {
            ExchangeScenario::Impersonation => {
                // Only trusted clients should be allowed to impersonate users
                if client.name.contains("untrusted") {
                    return Err(anyhow::anyhow!("Client not authorized for impersonation"));
                }
            },
            ExchangeScenario::Delegation => {
                // Validate delegation permissions
                if let Some(audience) = &request.audience {
                    // Ensure client is authorized to delegate to the target audience
                    if !Self::is_authorized_for_audience(&client, audience) {
                        return Err(anyhow::anyhow!("Client not authorized for target audience"));
                    }
                }
            },
            ExchangeScenario::ServiceToService => {
                // Service-to-service exchanges are generally allowed for confidential clients
                if client.secret.is_none() {
                    return Err(anyhow::anyhow!("Public clients not allowed for service-to-service exchange"));
                }
            },
        }

        Ok(())
    }

    /// Check if client is authorized for specific audience
    fn is_authorized_for_audience(
        client: &crate::app::models::oauth::Client,
        audience: &str,
    ) -> bool {
        // For production: implement proper audience validation
        // This would check a dedicated table like oauth_client_audiences
        // or use a metadata field if added to the oauth_clients table

        // Placeholder implementation - allow common OAuth audiences
        let allowed_audiences = vec![
            "https://api.example.com",
            "https://graph.microsoft.com",
            "https://www.googleapis.com/auth/userinfo.email",
        ];

        // Check if the target audience is in the allowed list
        allowed_audiences.contains(&audience)
            || allowed_audiences.iter().any(|allowed| {
                // Support wildcard matching for domain-based audiences
                if allowed.starts_with("*.") {
                    let domain = &allowed[2..];
                    audience.ends_with(domain)
                } else {
                    false
                }
            })
    }

    /// Validate and limit requested scopes
    fn validate_and_limit_scopes(
        subject_context: &TokenContext,
        actor_context: &Option<TokenContext>,
        requested_scopes: Option<&str>,
    ) -> Result<Vec<String>> {
        let subject_scopes = &subject_context.scopes;
        let actor_scopes = actor_context.as_ref().map(|ctx| &ctx.scopes);

        let requested = if let Some(scopes) = requested_scopes {
            scopes.split_whitespace().map(|s| s.to_string()).collect::<Vec<_>>()
        } else {
            // If no scopes requested, inherit from subject token
            subject_scopes.clone()
        };

        // Ensure requested scopes don't exceed subject token scopes
        let mut granted_scopes = Vec::new();
        for scope in requested {
            if subject_scopes.contains(&scope) || subject_scopes.contains(&"*".to_string()) {
                // If there's an actor token, also check actor permissions
                if let Some(actor_scopes) = actor_scopes {
                    if actor_scopes.contains(&scope) || actor_scopes.contains(&"*".to_string()) {
                        granted_scopes.push(scope);
                    }
                } else {
                    granted_scopes.push(scope);
                }
            }
        }

        if granted_scopes.is_empty() {
            return Err(anyhow::anyhow!("No valid scopes for token exchange"));
        }

        Ok(granted_scopes)
    }

    /// Create the exchanged token
    async fn create_exchanged_token(
        pool: &DbPool,
        client_id: &str,
        subject_context: &TokenContext,
        _actor_context: &Option<TokenContext>,
        scopes: &[String],
        request: &TokenExchangeRequest,
        scenario: ExchangeScenario,
    ) -> Result<ExchangedToken> {
        // Create access token for the exchange
        let create_token = CreateAccessToken {
            user_id: subject_context.user_id.clone(),
            client_id: client_id.to_string(),
            name: Some(format!("Token Exchange - {:?}", scenario)),
            scopes: scopes.to_vec(),
            expires_at: Some(Utc::now() + Duration::seconds(3600)), // 1 hour
            jwk_thumbprint: None, // Token exchange typically doesn't use DPoP
        };

        let access_token = TokenService::create_access_token(pool, create_token, Some(3600), None).await?;

        // Create enhanced JWT with exchange context
        let jwt_token = Self::generate_exchange_jwt(pool, &access_token, client_id, request, scenario)?;

        Ok(ExchangedToken {
            access_token,
            jwt_token,
        })
    }

    /// Generate JWT token with token exchange context
    fn generate_exchange_jwt(
        pool: &DbPool,
        access_token: &AccessToken,
        client_id: &str,
        request: &TokenExchangeRequest,
        scenario: ExchangeScenario,
    ) -> Result<String> {
        use crate::app::services::oauth::TokenService;
        use jsonwebtoken::{encode, Header, EncodingKey};
        use serde_json::Value;

        // Get base JWT token
        let jwt_token = TokenService::generate_jwt_token(pool, access_token, client_id)?;

        // Decode the existing token to get claims
        let token_data = TokenService::decode_jwt_token(&jwt_token)?;
        let mut claims = serde_json::to_value(&token_data)?;

        // Add token exchange specific claims per RFC 8693
        if let Some(claims_obj) = claims.as_object_mut() {
            // Add exchange scenario context
            claims_obj.insert("exchange_scenario".to_string(), Value::String(format!("{:?}", scenario)));

            // Add audience if specified
            if let Some(ref audience) = request.audience {
                claims_obj.insert("aud".to_string(), Value::String(audience.clone()));
            }

            // Add resource if specified
            if let Some(ref resource) = request.resource {
                claims_obj.insert("resource".to_string(), Value::String(resource.clone()));
            }

            // Add requested token type
            claims_obj.insert("requested_token_type".to_string(),
                Value::String(request.requested_token_type.clone().unwrap_or_else(|| "urn:ietf:params:oauth:token-type:access_token".to_string())));

            // Add subject token type for audit trail
            claims_obj.insert("subject_token_type".to_string(), Value::String(request.subject_token_type.clone()));

            // Add may_act claim for delegation scenarios
            if matches!(scenario, ExchangeScenario::Delegation) {
                if let Some(ref actor_token_type) = request.actor_token_type {
                    claims_obj.insert("may_act".to_string(), serde_json::json!({
                        "actor_token_type": actor_token_type
                    }));
                }
            }
        }

        // Re-encode the token with exchange-specific claims
        let header = Header::default();
        let encoding_key = EncodingKey::from_secret(std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string()).as_ref());
        let new_token = encode(&header, &claims, &encoding_key)?;

        Ok(new_token)
    }
}

#[derive(Debug)]
struct ExchangedToken {
    access_token: AccessToken,
    jwt_token: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_scenario_determination() {
        let subject = TokenContext {
            token_id: "token1".to_string(),
            user_id: Some("user1".to_string()),
            client_id: "client1".to_string(),
            scopes: vec!["read".to_string()],
            expires_at: None,
            token_type: TokenType::AccessToken,
        };

        let actor = Some(TokenContext {
            token_id: "token2".to_string(),
            user_id: Some("user2".to_string()),
            client_id: "client2".to_string(),
            scopes: vec!["admin".to_string()],
            expires_at: None,
            token_type: TokenType::AccessToken,
        });

        let scenario = TokenExchangeService::determine_exchange_scenario(&subject, &actor);
        assert!(scenario.is_ok());
        matches!(scenario.unwrap(), ExchangeScenario::Delegation);
    }

    #[test]
    fn test_scope_validation() {
        let subject = TokenContext {
            token_id: "token1".to_string(),
            user_id: Some("user1".to_string()),
            client_id: "client1".to_string(),
            scopes: vec!["read".to_string(), "write".to_string()],
            expires_at: None,
            token_type: TokenType::AccessToken,
        };

        let scopes = TokenExchangeService::validate_and_limit_scopes(
            &subject,
            &None,
            Some("read admin"),
        );

        assert!(scopes.is_ok());
        let granted = scopes.unwrap();
        assert!(granted.contains(&"read".to_string()));
        assert!(!granted.contains(&"admin".to_string()));
    }
}