# OAuth 2.1 Implementation Guide

**Complete OAuth 2.1 Authorization Server Implementation for Rustaxum Framework**

## Table of Contents

1. [Overview](#overview)
2. [RFC Compliance](#rfc-compliance)
3. [Architecture](#architecture)
4. [Security Features](#security-features)
5. [API Reference](#api-reference)
6. [Implementation Guide](#implementation-guide)
7. [Client Integration](#client-integration)
8. [Administration](#administration)
9. [Troubleshooting](#troubleshooting)
10. [Migration Guide](#migration-guide)

---

## Overview

This implementation provides a **production-ready OAuth 2.1 authorization server** built with Rust and the Axum framework, following modern security best practices and RFC standards. It offers a Laravel Passport-compatible API while providing enhanced security features for multi-tenant enterprise applications.

### Key Features

- **OAuth 2.1 Compliant**: Full adherence to the latest OAuth 2.1 specification
- **Multi-tenant Support**: Organization-scoped client access control
- **Enterprise Security**: PKCE mandatory, JWT tokens, scope-based permissions
- **Laravel Compatibility**: Familiar Passport-style API and CLI commands
- **Performance Optimized**: Rust-based implementation with connection pooling
- **Production Ready**: Comprehensive logging, monitoring, and error handling

---

## RFC Compliance

### Supported RFCs and Standards

| RFC | Standard | Status | Description |
|-----|----------|--------|-------------|
| [draft-ietf-oauth-v2-1-13](https://datatracker.ietf.org/doc/draft-ietf-oauth-v2-1/) | OAuth 2.1 Authorization Framework | ✅ **Full** | Core authorization framework |
| [RFC 6749](https://tools.ietf.org/html/rfc6749) | OAuth 2.0 Authorization Framework | ✅ **Full** | Legacy OAuth 2.0 support (deprecated grants removed) |
| [RFC 6750](https://tools.ietf.org/html/rfc6750) | Bearer Token Usage | ✅ **Full** | Bearer token authentication |
| [RFC 7636](https://tools.ietf.org/html/rfc7636) | PKCE | ✅ **Mandatory** | Proof Key for Code Exchange (required for all flows) |
| [RFC 7662](https://tools.ietf.org/html/rfc7662) | Token Introspection | ✅ **Full** | Token metadata and validation |
| [RFC 8252](https://tools.ietf.org/html/rfc8252) | OAuth for Native Apps | ✅ **Full** | Native application security |
| [RFC 8628](https://tools.ietf.org/html/rfc8628) | Device Authorization Grant | ✅ **Full** | Device flow for input-constrained devices |
| [RFC 9068](https://tools.ietf.org/html/rfc9068) | JWT Profile for Access Tokens | ✅ **Full** | Structured JWT access tokens with claims |
| [RFC 9449](https://tools.ietf.org/html/rfc9449) | DPoP Proof of Possession | ✅ **Full** | Token binding for enhanced security |
| [RFC 9700](https://tools.ietf.org/html/rfc9700) | OAuth Security Best Practices | ✅ **Full** | Latest security recommendations |

### OAuth 2.1 Security Enhancements

#### ✅ **Mandatory PKCE**
All authorization code flows **MUST** include PKCE parameters:
- `code_challenge`: SHA256 hash of code verifier (S256 method preferred)
- `code_challenge_method`: Must be "S256" (plain method deprecated)

#### ❌ **Removed Deprecated Flows**
- **Implicit Grant**: Removed for security (use authorization code + PKCE)
- **Password Grant**: Removed for security (use device flow or authorization code)

#### ✅ **Enhanced Security**
- **Exact URI Matching**: No wildcard or partial matching for redirect URIs
- **Short-lived Codes**: Authorization codes expire in 10 minutes
- **Token Rotation**: Refresh tokens are rotated on each use
- **Sender Constraints**: Public client refresh tokens are sender-constrained

---

## Architecture

### System Components

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   OAuth Client  │────│  Authorization   │────│   Resource      │
│   Application   │    │     Server       │    │    Server       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │                          │
                              │                          │
                       ┌─────────────┐           ┌─────────────┐
                       │  Database   │           │   Token     │
                       │  (PostgreSQL)│           │ Validation  │
                       └─────────────┘           └─────────────┘
```

### Database Schema

#### OAuth Tables

```sql
-- Client Applications
oauth_clients (
    id CHAR(26) PRIMARY KEY,
    organization_id CHAR(26) REFERENCES organizations(id),
    name VARCHAR NOT NULL,
    secret VARCHAR DEFAULT NULL,
    redirect_uris TEXT NOT NULL,
    personal_access_client BOOLEAN DEFAULT FALSE,
    password_client BOOLEAN DEFAULT FALSE,
    revoked BOOLEAN DEFAULT FALSE
);

-- Access Tokens (JWT-backed with DPoP support)
oauth_access_tokens (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) REFERENCES sys_users(id),
    client_id CHAR(26) REFERENCES oauth_clients(id),
    name VARCHAR DEFAULT NULL,
    scopes TEXT DEFAULT NULL,
    revoked BOOLEAN DEFAULT FALSE,
    expires_at TIMESTAMPTZ DEFAULT NULL,
    jwk_thumbprint VARCHAR(255) DEFAULT NULL  -- RFC 9449: DPoP token binding
);

-- Refresh Tokens (Rotated on use)
oauth_refresh_tokens (
    id CHAR(26) PRIMARY KEY,
    access_token_id CHAR(26) REFERENCES oauth_access_tokens(id),
    revoked BOOLEAN DEFAULT FALSE,
    expires_at TIMESTAMPTZ DEFAULT NULL
);

-- Authorization Codes (PKCE-enabled)
oauth_auth_codes (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) REFERENCES sys_users(id),
    client_id CHAR(26) REFERENCES oauth_clients(id),
    scopes TEXT DEFAULT NULL,
    challenge VARCHAR DEFAULT NULL,      -- PKCE code challenge
    challenge_method VARCHAR DEFAULT NULL, -- S256 or plain
    redirect_uri TEXT NOT NULL,
    expires_at TIMESTAMPTZ DEFAULT NULL
);

-- Permission Scopes
oauth_scopes (
    id CHAR(26) PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    description TEXT DEFAULT NULL,
    is_default BOOLEAN DEFAULT FALSE
);

-- Device Authorization Codes (RFC 8628)
oauth_device_codes (
    id CHAR(26) PRIMARY KEY,
    device_code VARCHAR(64) UNIQUE NOT NULL,
    user_code VARCHAR(9) UNIQUE NOT NULL,
    client_id CHAR(26) REFERENCES oauth_clients(id),
    user_id CHAR(26) REFERENCES sys_users(id) DEFAULT NULL,
    scopes TEXT DEFAULT NULL,
    verification_uri VARCHAR(255) NOT NULL,
    verification_uri_complete VARCHAR(512) DEFAULT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    interval INTEGER NOT NULL DEFAULT 5,
    user_authorized BOOLEAN DEFAULT FALSE,
    revoked BOOLEAN DEFAULT FALSE
);
```

### Service Layer Architecture

```rust
// Core Services
src/app/services/oauth/
├── token_service.rs      // JWT generation, validation, exchange
├── client_service.rs     // Client management, authentication
├── scope_service.rs      // Permission validation, scope resolution
└── mod.rs               // Service module exports

// Models
src/app/models/oauth/
├── client.rs            // OAuth client model
├── access_token.rs      // JWT access token model
├── refresh_token.rs     // Refresh token model
├── auth_code.rs         // Authorization code with PKCE
├── scope.rs             // Permission scope model
└── mod.rs              // Model exports

// Controllers
src/app/http/controllers/oauth/
├── oauth_controller.rs   // Core OAuth endpoints
├── client_controller.rs  // Client management API
├── token_controller.rs   // Token management API
└── admin_controller.rs   // Administration API
```

---

## Security Features

### Multi-tenant Organization Access Control

Organizations can control which users can access their OAuth applications:

```rust
// Example: User access validation
let has_access = ClientService::validate_user_organization_access(
    &pool,
    client_id,
    user_id
)?;

if !has_access {
    return Err("User does not have access to this application");
}
```

**Organization Scoping Rules:**
- **Global Clients** (`organization_id = NULL`): Accessible to all users
- **Scoped Clients** (`organization_id = <uuid>`): Only accessible to organization members
- **Access Validation**: Checked during authorization, token exchange, and refresh

### PKCE Implementation (RFC 7636)

#### S256 Method (Recommended)
```javascript
// Client-side PKCE generation
const codeVerifier = base64URLEncode(crypto.getRandomValues(new Uint8Array(32)));
const codeChallenge = base64URLEncode(
    await crypto.subtle.digest('SHA-256', new TextEncoder().encode(codeVerifier))
);

// Authorization request
const authUrl = `https://auth.example.com/oauth/authorize?` +
    `response_type=code&` +
    `client_id=${clientId}&` +
    `redirect_uri=${redirectUri}&` +
    `code_challenge=${codeChallenge}&` +
    `code_challenge_method=S256&` +
    `state=${state}`;
```

#### Server-side Validation
```rust
// Automatic PKCE verification during token exchange
pub fn verify_pkce_challenge(&self, verifier: &str) -> bool {
    match (&self.challenge, &self.challenge_method) {
        (Some(challenge), Some("S256")) => {
            let mut hasher = Sha256::new();
            hasher.update(verifier.as_bytes());
            let digest = hasher.finalize();
            let encoded = URL_SAFE_NO_PAD.encode(digest);
            encoded == *challenge
        },
        (Some(challenge), Some("plain")) => verifier == challenge,
        (None, None) => false, // OAuth 2.1: PKCE is mandatory
        _ => false,
    }
}
```

### JWT Token Security

#### Token Structure
```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT"
  },
  "payload": {
    "sub": "01HPQR3TUVWXYZ",    // User ID (ULID)
    "aud": "01HPQS4VWXYZ01",   // Client ID (ULID)
    "exp": 1672531200,         // Expiration timestamp
    "iat": 1672527600,         // Issued at timestamp
    "jti": "01HPQT5WXYZ012",   // Token ID (ULID)
    "scopes": ["read", "write"] // Granted permissions
  }
}
```

#### Environment Configuration
```bash
# OAuth JWT Configuration
OAUTH_JWT_SECRET=your-strong-jwt-secret-here
OAUTH_ACCESS_TOKEN_TTL=3600      # 1 hour
OAUTH_REFRESH_TOKEN_TTL=604800   # 7 days
OAUTH_AUTH_CODE_TTL=600          # 10 minutes
```

### Scope-based Authorization

#### Default Scopes
```sql
INSERT INTO oauth_scopes (id, name, description, is_default) VALUES
('01HPQU6WXYZ012', '*', 'Full access (admin only)', false),
('01HPQU7WXYZ013', 'read', 'Read access to resources', true),
('01HPQU8WXYZ014', 'write', 'Write access to resources', false),
('01HPQU9WXYZ015', 'admin', 'Administrative access', false);
```

#### Scope Validation
```rust
// Middleware example: Require specific scopes
pub async fn require_scopes(
    headers: HeaderMap,
    required_scopes: Vec<&str>,
    next: Next<State<DbPool>>,
) -> impl IntoResponse {
    let token = extract_bearer_token(&headers)?;
    let (access_token, _claims) = TokenService::validate_token_and_scopes(
        &pool, &token, &required_scopes
    )?;

    if !access_token.is_valid() {
        return Err("Token expired or revoked");
    }

    Ok(next.run(req).await)
}
```

### RFC 9068: JWT Profile for Access Tokens

#### Enhanced JWT Claims Structure
```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT"
  },
  "payload": {
    // Standard JWT claims (RFC 7519)
    "iss": "https://auth.rustaxum.dev",        // Issuer
    "sub": "01HPQR3TUVWXYZ",                   // Subject (user identifier)
    "aud": ["01HPQS4VWXYZ01"],                 // Audience (client_id)
    "exp": 1672531200,                         // Expiration time
    "iat": 1672527600,                         // Issued at
    "jti": "01HPQT5WXYZ012",                   // JWT ID (token identifier)

    // OAuth 2.0 specific claims (RFC 9068)
    "client_id": "01HPQS4VWXYZ01",             // OAuth client identifier
    "scope": "read write",                     // Granted scopes (space-separated)
    "token_use": "access_token",               // Always "access_token"

    // Additional optional claims
    "auth_time": 1672527000,                   // Authentication time
    "username": "user@example.com",            // Human-readable identifier
    "groups": ["admin", "users"],              // User groups
    "roles": ["read", "write"],                // User roles
    "entitlements": ["feature:premium"]        // User entitlements
  }
}
```

#### Backward Compatibility
The implementation maintains backward compatibility with existing tokens while providing enhanced security through RFC 9068 compliance.

### RFC 8628: Device Authorization Grant Security

#### Device Code Generation
```rust
// Cryptographically secure device codes (64 characters)
pub fn generate_device_code() -> String {
    use rand::Rng;
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..64).map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char).collect()
}

// Human-friendly user codes (8 characters with separator)
pub fn generate_user_code() -> String {
    // Format: ABCD-EFGH (no ambiguous characters)
    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    // Implementation generates codes like: WDJB-MJHT
}
```

#### Security Features
- **Cryptographic Randomness**: Device codes use 64 characters of entropy
- **User-Friendly Codes**: Human-readable codes avoid confusing characters (0,O,I,1)
- **Time-Limited**: Default 30-minute expiration with configurable intervals
- **Single-Use**: Device codes are revoked after successful token exchange
- **Rate Limiting**: Built-in polling interval prevents abuse (minimum 5 seconds)

### RFC 9449: DPoP (Demonstrating Proof of Possession)

#### Token Binding Security
DPoP provides cryptographic proof that the client presenting the token is the same client that requested it:

```rust
// DPoP proof validation
pub fn validate_dpop_proof(
    dpop_proof: &str,
    http_method: &str,
    http_url: &str,
    access_token_hash: Option<&str>,
    expected_nonce: Option<&str>,
) -> Result<String> {
    // 1. Verify JWT signature using JWK from header
    // 2. Validate HTTP method and URL binding
    // 3. Check token freshness (max 60 seconds)
    // 4. Verify access token hash (ath claim)
    // 5. Return JWK thumbprint for token binding
}
```

#### Key Security Benefits
- **Token Theft Prevention**: Stolen tokens cannot be used without private key
- **Replay Attack Mitigation**: HTTP method/URL binding prevents reuse
- **Cryptographic Binding**: Access tokens bound to client's public key
- **Nonce Support**: Server can require nonces to prevent replay attacks

#### DPoP Middleware Integration
```rust
// Automatic DPoP validation for protected resources
pub async fn dpop_validation_middleware(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Automatically validates DPoP tokens when presented
    // Supports both DPoP and Bearer tokens transparently
}
```

---

## API Reference

### Core OAuth Endpoints

#### Authorization Endpoint
```http
GET /oauth/authorize
```

**OAuth 2.1 Parameters:**
- `response_type=code` *(required)* - Must be "code"
- `client_id` *(required)* - Client identifier (ULID)
- `redirect_uri` *(required)* - Exact registered redirect URI
- `code_challenge` *(required)* - PKCE challenge (base64url encoded)
- `code_challenge_method` *(optional)* - "S256" or "plain" (defaults to S256)
- `scope` *(optional)* - Space-separated list of scopes
- `state` *(recommended)* - CSRF protection parameter

**Success Response:**
```http
HTTP/1.1 302 Found
Location: https://client.example.com/callback?code=01HPQV0WXYZ012&state=xyz
```

**Error Response:**
```http
HTTP/1.1 302 Found
Location: https://client.example.com/callback?error=invalid_request&error_description=PKCE+code_challenge+is+required&state=xyz
```

#### Token Endpoint
```http
POST /oauth/token
Content-Type: application/x-www-form-urlencoded
```

##### Authorization Code Grant
```http
grant_type=authorization_code
&code=01HPQV0WXYZ012
&client_id=01HPQS4VWXYZ01
&client_secret=your-client-secret
&redirect_uri=https://client.example.com/callback
&code_verifier=your-pkce-verifier
```

**Success Response:**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "01HPQW1WXYZ013",
  "scope": "read write"
}
```

##### Refresh Token Grant
```http
grant_type=refresh_token
&refresh_token=01HPQW1WXYZ013
&client_id=01HPQS4VWXYZ01
&client_secret=your-client-secret
```

##### Client Credentials Grant
```http
grant_type=client_credentials
&client_id=01HPQS4VWXYZ01
&client_secret=your-client-secret
&scope=read
```

#### Token Introspection
```http
POST /oauth/introspect
Content-Type: application/x-www-form-urlencoded

token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
&token_type_hint=access_token
```

**Response:**
```json
{
  "active": true,
  "scope": "read write",
  "client_id": "01HPQS4VWXYZ01",
  "username": "user@example.com",
  "exp": 1672531200,
  "iat": 1672527600,
  "sub": "01HPQR3TUVWXYZ",
  "aud": "01HPQS4VWXYZ01"
}
```

#### Token Revocation
```http
POST /oauth/revoke
Content-Type: application/x-www-form-urlencoded

token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### RFC 8628: Device Authorization Grant

#### Device Authorization Request
```http
POST /oauth/device/code
Content-Type: application/x-www-form-urlencoded

client_id=01HPQS4VWXYZ01
&scope=read%20write
```

**Response:**
```json
{
  "device_code": "GmRhmhcxhwAzkoEqiMEg_DnyEysNkuNhszIySk9eS",
  "user_code": "WDJB-MJHT",
  "verification_uri": "https://auth.example.com/device",
  "verification_uri_complete": "https://auth.example.com/device?user_code=WDJB-MJHT",
  "expires_in": 1800,
  "interval": 5
}
```

#### Device Token Request (Polling)
```http
POST /oauth/token
Content-Type: application/x-www-form-urlencoded

grant_type=urn:ietf:params:oauth:grant-type:device_code
&device_code=GmRhmhcxhwAzkoEqiMEg_DnyEysNkuNhszIySk9eS
&client_id=01HPQS4VWXYZ01
```

**Pending Response:**
```json
{
  "error": "authorization_pending",
  "error_description": "The authorization request is still pending"
}
```

**Success Response:**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "01HPQW1WXYZ013",
  "scope": "read write"
}
```

#### Device Authorization (User Interface)
```http
GET /oauth/device?user_code=WDJB-MJHT
```

### RFC 9449: DPoP (Demonstrating Proof of Possession)

#### DPoP-bound Token Request
```http
POST /oauth/token
Content-Type: application/x-www-form-urlencoded
DPoP: eyJhbGciOiJFUzI1NiIsInR5cCI6ImRwb3Arand0IiwiandrIjp7Imt0eSI6IkVDIiwi...

grant_type=authorization_code
&code=01HPQV0WXYZ012
&client_id=01HPQS4VWXYZ01
&redirect_uri=https://client.example.com/callback
&code_verifier=your-pkce-verifier
```

**DPoP Response:**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "DPoP",
  "expires_in": 3600,
  "refresh_token": "01HPQW1WXYZ013",
  "scope": "read write"
}
```

#### Protected Resource Access with DPoP
```http
GET /api/protected-resource
Authorization: DPoP eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
DPoP: eyJhbGciOiJFUzI1NiIsInR5cCI6ImRwb3Arand0IiwiandrIjp7Imt0eSI6IkVDIiwi...
```

**DPoP Proof JWT Header:**
```json
{
  "alg": "ES256",
  "typ": "dpop+jwt",
  "jwk": {
    "kty": "EC",
    "crv": "P-256",
    "x": "WKn-ZIGevcwGIyyrzFoZNBdaq9_TsqzGHwHitJBcBmXQ",
    "y": "y77As5vbZdSKdf4bzEapBBBbBbjkgUj5TCFMwj8vKwQ"
  }
}
```

**DPoP Proof JWT Claims:**
```json
{
  "jti": "01HPQV0WXYZ012",
  "htm": "GET",
  "htu": "https://api.example.com/protected-resource",
  "iat": 1672527600,
  "ath": "fUHyO2r2Z3DZ53EsNrWBb0xWXoaNy4IapxXiGRuklEc"
}
```

### Client Management API

#### Create OAuth Client
```http
POST /oauth/clients
Authorization: Bearer <admin_token>
Content-Type: application/json

{
  "name": "My Application",
  "redirect_uris": [
    "https://myapp.example.com/oauth/callback",
    "https://myapp.example.com/oauth/callback/mobile"
  ],
  "personal_access_client": false,
  "password_client": false
}
```

**Response:**
```json
{
  "id": "01HPQS4VWXYZ01",
  "name": "My Application",
  "secret": "generated-client-secret",
  "redirect_uris": [
    "https://myapp.example.com/oauth/callback",
    "https://myapp.example.com/oauth/callback/mobile"
  ],
  "personal_access_client": false,
  "password_client": false,
  "revoked": false,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### List Clients
```http
GET /oauth/clients
Authorization: Bearer <admin_token>
```

#### Update Client
```http
PUT /oauth/clients/01HPQS4VWXYZ01
Authorization: Bearer <admin_token>
Content-Type: application/json

{
  "name": "Updated Application Name",
  "redirect_uris": [
    "https://myapp.example.com/oauth/callback"
  ]
}
```

#### Regenerate Client Secret
```http
POST /oauth/clients/01HPQS4VWXYZ01/regenerate-secret
Authorization: Bearer <admin_token>
```

#### Delete Client
```http
DELETE /oauth/clients/01HPQS4VWXYZ01
Authorization: Bearer <admin_token>
```

### Personal Access Token API

#### Create Personal Access Token
```http
POST /oauth/personal-access-tokens
Authorization: Bearer <user_token>
Content-Type: application/json

{
  "name": "CLI Access Token",
  "scopes": ["read", "write"]
}
```

#### List Personal Access Tokens
```http
GET /oauth/personal-access-tokens
Authorization: Bearer <user_token>
```

#### Revoke Personal Access Token
```http
DELETE /oauth/personal-access-tokens/01HPQX2WXYZ014
Authorization: Bearer <user_token>
```

### Scope Management API

#### Create Scope
```http
POST /oauth/scopes
Authorization: Bearer <admin_token>
Content-Type: application/json

{
  "name": "posts:read",
  "description": "Read access to posts",
  "is_default": false
}
```

#### List Scopes
```http
GET /oauth/scopes
```

#### Update Scope
```http
PUT /oauth/scopes/01HPQU8WXYZ014
Authorization: Bearer <admin_token>
Content-Type: application/json

{
  "description": "Updated description"
}
```

#### Delete Scope
```http
DELETE /oauth/scopes/01HPQU8WXYZ014
Authorization: Bearer <admin_token>
```

---

## Implementation Guide

### Installation and Setup

#### 1. Database Migration
```bash
# Run OAuth database migrations
cargo run --bin artisan migrate

# Seed default scopes
cargo run --bin artisan db:seed --class OAuthScopeSeeder
```

#### 2. Environment Configuration
```bash
# .env configuration
OAUTH_JWT_SECRET=your-256-bit-secret
OAUTH_ACCESS_TOKEN_TTL=3600
OAUTH_REFRESH_TOKEN_TTL=604800
OAUTH_AUTH_CODE_TTL=600
```

#### 3. Initialize OAuth System
```bash
# Install OAuth/Passport system
cargo run --bin artisan passport install

# Create OAuth client
cargo run --bin artisan passport client \
  --name "My Web App" \
  --redirect-uris "https://myapp.com/oauth/callback,https://myapp.com/oauth/mobile"

# Create personal access client
cargo run --bin artisan passport client \
  --name "Personal Access Client" \
  --personal \
  --redirect-uris "http://localhost"
```

### Protecting Routes with OAuth

#### Basic Token Validation
```rust
use crate::app::services::oauth::TokenService;

pub async fn protected_route(
    headers: HeaderMap,
    State(pool): State<DbPool>,
) -> Result<impl IntoResponse, AppError> {
    // Extract Bearer token
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or("Missing or invalid authorization header")?;

    // Validate token and check scopes
    let (access_token, claims) = TokenService::validate_token_and_scopes(
        &pool,
        auth_header,
        &["read"] // Required scopes
    ).map_err(|_| "Invalid or expired token")?;

    // Access user information
    let user_id = claims.sub.to_string();

    Ok(Json(json!({
        "message": "Access granted",
        "user_id": user_id,
        "scopes": access_token.get_scopes()
    })))
}
```

#### Middleware Implementation
```rust
use axum::middleware::Next;

pub async fn oauth_middleware(
    headers: HeaderMap,
    State(pool): State<DbPool>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    match TokenService::validate_token_and_scopes(&pool, token, &[]) {
        Ok((access_token, claims)) => {
            // Add user context to request
            request.extensions_mut().insert(UserContext {
                user_id: claims.sub.to_string(),
                client_id: claims.aud.to_string(),
                scopes: access_token.get_scopes(),
            });
            Ok(next.run(request).await)
        },
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
```

### Client Application Integration

#### Web Application Flow

1. **Authorization Request**
```javascript
// Generate PKCE parameters
function generatePKCE() {
    const codeVerifier = base64URLEncode(crypto.getRandomValues(new Uint8Array(32)));
    const codeChallenge = base64URLEncode(
        await crypto.subtle.digest('SHA-256', new TextEncoder().encode(codeVerifier))
    );
    return { codeVerifier, codeChallenge };
}

// Redirect to authorization server
const { codeVerifier, codeChallenge } = await generatePKCE();
localStorage.setItem('pkce_verifier', codeVerifier);

const authUrl = new URL('/oauth/authorize', 'https://auth.example.com');
authUrl.searchParams.set('response_type', 'code');
authUrl.searchParams.set('client_id', 'your-client-id');
authUrl.searchParams.set('redirect_uri', 'https://yourapp.com/callback');
authUrl.searchParams.set('code_challenge', codeChallenge);
authUrl.searchParams.set('code_challenge_method', 'S256');
authUrl.searchParams.set('scope', 'read write');
authUrl.searchParams.set('state', generateState());

window.location.href = authUrl.toString();
```

2. **Token Exchange**
```javascript
// Handle callback and exchange code for tokens
async function handleCallback(code, state) {
    const codeVerifier = localStorage.getItem('pkce_verifier');

    const response = await fetch('/oauth/token', {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams({
            grant_type: 'authorization_code',
            code: code,
            client_id: 'your-client-id',
            client_secret: 'your-client-secret',
            redirect_uri: 'https://yourapp.com/callback',
            code_verifier: codeVerifier
        })
    });

    const tokens = await response.json();

    // Store tokens securely
    sessionStorage.setItem('access_token', tokens.access_token);
    sessionStorage.setItem('refresh_token', tokens.refresh_token);
}
```

3. **API Requests**
```javascript
// Make authenticated API requests
async function apiRequest(url, options = {}) {
    const token = sessionStorage.getItem('access_token');

    return fetch(url, {
        ...options,
        headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json',
            ...options.headers
        }
    });
}
```

4. **Token Refresh**
```javascript
// Refresh expired tokens
async function refreshTokens() {
    const refreshToken = sessionStorage.getItem('refresh_token');

    const response = await fetch('/oauth/token', {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams({
            grant_type: 'refresh_token',
            refresh_token: refreshToken,
            client_id: 'your-client-id',
            client_secret: 'your-client-secret'
        })
    });

    if (response.ok) {
        const tokens = await response.json();
        sessionStorage.setItem('access_token', tokens.access_token);
        sessionStorage.setItem('refresh_token', tokens.refresh_token);
        return true;
    } else {
        // Redirect to login
        window.location.href = '/login';
        return false;
    }
}
```

#### Mobile Application (PKCE)

For mobile applications, use custom URL schemes and enhanced PKCE security:

```swift
// iOS Swift example
import AuthenticationServices

class OAuthManager: NSObject, ASWebAuthenticationPresentationContextProviding {
    func authenticate() {
        let codeVerifier = generateCodeVerifier()
        let codeChallenge = generateCodeChallenge(from: codeVerifier)

        var components = URLComponents(string: "https://auth.example.com/oauth/authorize")!
        components.queryItems = [
            URLQueryItem(name: "response_type", value: "code"),
            URLQueryItem(name: "client_id", value: "your-mobile-client-id"),
            URLQueryItem(name: "redirect_uri", value: "myapp://oauth/callback"),
            URLQueryItem(name: "code_challenge", value: codeChallenge),
            URLQueryItem(name: "code_challenge_method", value: "S256"),
            URLQueryItem(name: "scope", value: "read write"),
            URLQueryItem(name: "state", value: generateState())
        ]

        let session = ASWebAuthenticationSession(
            url: components.url!,
            callbackURLScheme: "myapp"
        ) { [weak self] callbackURL, error in
            if let url = callbackURL {
                self?.handleCallback(url: url, codeVerifier: codeVerifier)
            }
        }

        session.presentationContextProvider = self
        session.start()
    }
}
```

#### Server-to-Server (Client Credentials)

```python
# Python example for server-to-server authentication
import requests
from urllib.parse import urlencode

class OAuthClient:
    def __init__(self, client_id, client_secret, auth_server_url):
        self.client_id = client_id
        self.client_secret = client_secret
        self.auth_server_url = auth_server_url
        self.access_token = None

    def authenticate(self, scopes=None):
        data = {
            'grant_type': 'client_credentials',
            'client_id': self.client_id,
            'client_secret': self.client_secret,
        }

        if scopes:
            data['scope'] = ' '.join(scopes)

        response = requests.post(
            f'{self.auth_server_url}/oauth/token',
            data=data,
            headers={'Content-Type': 'application/x-www-form-urlencoded'}
        )

        if response.status_code == 200:
            tokens = response.json()
            self.access_token = tokens['access_token']
            return True
        return False

    def make_request(self, url, method='GET', **kwargs):
        if not self.access_token:
            self.authenticate()

        headers = kwargs.get('headers', {})
        headers['Authorization'] = f'Bearer {self.access_token}'
        kwargs['headers'] = headers

        return requests.request(method, url, **kwargs)

# Usage
client = OAuthClient('client-id', 'client-secret', 'https://auth.example.com')
response = client.make_request('https://api.example.com/data')
```

---

## Administration

### CLI Commands Reference

#### Installation and Setup
```bash
# Initialize OAuth system
cargo run --bin artisan passport install

# Check system status
cargo run --bin artisan passport status
```

#### Client Management
```bash
# Create confidential client
cargo run --bin artisan passport client \
  --name "Web Application" \
  --redirect-uris "https://app.com/callback,https://app.com/mobile"

# Create personal access client
cargo run --bin artisan passport client \
  --name "Personal Access Client" \
  --personal \
  --redirect-uris "http://localhost"

# List all clients
cargo run --bin artisan passport list-clients

# Update client
cargo run --bin artisan passport update-client <client-id> \
  --name "Updated Name" \
  --redirect-uris "https://newapp.com/callback"

# Regenerate client secret
cargo run --bin artisan passport regenerate-secret <client-id>

# Revoke client
cargo run --bin artisan passport revoke-client <client-id>

# Delete client (irreversible)
cargo run --bin artisan passport delete-client <client-id>
```

#### Scope Management
```bash
# Create scope
cargo run --bin artisan passport scope \
  --name "posts:read" \
  --description "Read access to posts" \
  --default

# List scopes
cargo run --bin artisan passport list-scopes

# Update scope
cargo run --bin artisan passport update-scope <scope-id> \
  --description "Updated description"

# Delete scope
cargo run --bin artisan passport delete-scope <scope-name-or-id>
```

#### Token Management
```bash
# List user tokens
cargo run --bin artisan passport list-tokens --user-id <user-id>

# List all active tokens (admin only)
cargo run --bin artisan passport list-tokens --all

# Revoke specific token
cargo run --bin artisan passport revoke-token <token-id>

# Revoke all user tokens
cargo run --bin artisan passport revoke-all-user-tokens <user-id>

# Token cleanup (remove expired tokens)
cargo run --bin artisan passport cleanup-tokens
```

#### Monitoring and Diagnostics
```bash
# System health check
cargo run --bin artisan passport health

# Token statistics
cargo run --bin artisan passport stats

# Export audit log
cargo run --bin artisan passport export-audit --format json --output audit.json

# Validate configuration
cargo run --bin artisan passport validate-config
```

### Administrative Web Interface

The system provides a comprehensive admin panel for OAuth management:

```rust
// Admin dashboard endpoint
pub async fn get_dashboard_stats(
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Verify admin access
    let token = extract_bearer_token(&headers)?;
    let (access_token, _) = TokenService::validate_token_and_scopes(
        &pool, &token, &["admin"]
    )?;

    // Gather statistics
    let stats = AdminService::get_dashboard_stats(&pool).await?;

    Json(json!({
        "total_clients": stats.total_clients,
        "active_tokens": stats.active_tokens,
        "expired_tokens": stats.expired_tokens,
        "total_scopes": stats.total_scopes,
        "recent_authorizations": stats.recent_authorizations,
        "security_events": stats.security_events
    }))
}
```

### Database Administration

#### Performance Optimization
```sql
-- Index optimization for high-traffic scenarios
CREATE INDEX CONCURRENTLY idx_oauth_access_tokens_user_active
  ON oauth_access_tokens (user_id, expires_at)
  WHERE revoked = false;

CREATE INDEX CONCURRENTLY idx_oauth_auth_codes_client_expires
  ON oauth_auth_codes (client_id, expires_at)
  WHERE revoked = false;

-- Cleanup expired tokens (run periodically)
DELETE FROM oauth_auth_codes
WHERE expires_at < NOW() - INTERVAL '1 day';

DELETE FROM oauth_refresh_tokens
WHERE expires_at < NOW() - INTERVAL '1 day';
```

#### Backup and Recovery
```bash
# Backup OAuth data
pg_dump -h localhost -U username -d database_name \
  -t oauth_clients \
  -t oauth_access_tokens \
  -t oauth_refresh_tokens \
  -t oauth_auth_codes \
  -t oauth_scopes \
  -t oauth_personal_access_clients \
  > oauth_backup.sql

# Restore from backup
psql -h localhost -U username -d database_name < oauth_backup.sql
```

---

## Troubleshooting

### Common Issues and Solutions

#### 1. PKCE Validation Errors

**Error:** `Invalid PKCE challenge`

**Symptoms:**
- Authorization code exchange fails
- Error message: "PKCE verification failed"

**Solutions:**
```javascript
// Ensure correct PKCE implementation
function generatePKCE() {
    const codeVerifier = base64URLEncode(crypto.getRandomValues(new Uint8Array(32)));

    // Use SHA256 for challenge generation
    const encoder = new TextEncoder();
    const data = encoder.encode(codeVerifier);

    return crypto.subtle.digest('SHA-256', data).then(digest => {
        const codeChallenge = base64URLEncode(new Uint8Array(digest));
        return { codeVerifier, codeChallenge };
    });
}

// Ensure base64url encoding (not base64)
function base64URLEncode(buffer) {
    return btoa(String.fromCharCode(...new Uint8Array(buffer)))
        .replace(/\+/g, '-')
        .replace(/\//g, '_')
        .replace(/=/g, '');
}
```

**Debugging:**
```bash
# Enable OAuth debug logging
RUST_LOG=rustaxum::app::services::oauth=debug cargo run
```

#### 2. Redirect URI Mismatch

**Error:** `Invalid redirect URI`

**Symptoms:**
- Authorization request fails
- Error message: "OAuth 2.1 requires exact string matching"

**Solutions:**
```bash
# Check registered URIs
cargo run --bin artisan passport list-clients

# Update client with correct URIs
cargo run --bin artisan passport update-client <client-id> \
  --redirect-uris "https://exact-match.com/callback"

# Verify exact matching (case-sensitive, no trailing slashes unless registered)
```

#### 3. Token Expiration Issues

**Error:** `Token expired or revoked`

**Symptoms:**
- API requests fail with 401 Unauthorized
- Token introspection returns `"active": false`

**Solutions:**
```javascript
// Implement automatic token refresh
async function makeAuthenticatedRequest(url, options = {}) {
    let token = getAccessToken();

    let response = await fetch(url, {
        ...options,
        headers: {
            'Authorization': `Bearer ${token}`,
            ...options.headers
        }
    });

    if (response.status === 401) {
        // Try to refresh token
        const refreshed = await refreshTokens();
        if (refreshed) {
            token = getAccessToken();
            response = await fetch(url, {
                ...options,
                headers: {
                    'Authorization': `Bearer ${token}`,
                    ...options.headers
                }
            });
        }
    }

    return response;
}
```

#### 4. Scope Permission Errors

**Error:** `Insufficient scope`

**Symptoms:**
- Token validation succeeds but access is denied
- Error message: "Insufficient scope"

**Solutions:**
```bash
# Check granted scopes
cargo run --bin artisan passport introspect-token <token>

# Update client to request correct scopes
# Ensure user has granted necessary permissions
# Verify scope configuration
cargo run --bin artisan passport list-scopes
```

#### 5. Client Authentication Failures

**Error:** `Invalid client credentials`

**Symptoms:**
- Token exchange fails
- Client secret verification fails

**Solutions:**
```bash
# Regenerate client secret if compromised
cargo run --bin artisan passport regenerate-secret <client-id>

# Verify client configuration
cargo run --bin artisan passport show-client <client-id>

# Check if client is revoked
SELECT * FROM oauth_clients WHERE id = '<client-id>';
```

### Debugging Tools

#### Token Introspection
```bash
# Inspect token details
curl -X POST https://auth.example.com/oauth/introspect \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

#### JWT Debugging
```bash
# Decode JWT locally (for debugging only)
echo "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." | \
  base64 -d | jq '.'
```

#### Database Queries
```sql
-- Check token status
SELECT id, user_id, client_id, scopes, revoked, expires_at
FROM oauth_access_tokens
WHERE id = '01HPQX2WXYZ014';

-- Check authorization codes
SELECT id, user_id, client_id, challenge_method, expires_at
FROM oauth_auth_codes
WHERE client_id = '01HPQS4VWXYZ01'
ORDER BY created_at DESC;

-- Check client configuration
SELECT id, name, redirect_uris, personal_access_client, revoked
FROM oauth_clients
WHERE id = '01HPQS4VWXYZ01';
```

### Performance Monitoring

#### Metrics Collection
```rust
// Example metrics implementation
pub struct OAuthMetrics {
    pub token_requests: Counter,
    pub auth_code_exchanges: Counter,
    pub refresh_token_uses: Counter,
    pub failed_authentications: Counter,
    pub token_validation_time: Histogram,
}

impl OAuthMetrics {
    pub fn record_token_validation(&self, duration: Duration) {
        self.token_validation_time.observe(duration.as_secs_f64());
    }

    pub fn increment_failed_auth(&self) {
        self.failed_authentications.inc();
    }
}
```

#### Health Check Endpoint
```rust
pub async fn health_check(State(pool): State<DbPool>) -> impl IntoResponse {
    let mut health = HealthStatus {
        status: "healthy".to_string(),
        checks: HashMap::new(),
    };

    // Database connectivity
    match pool.get() {
        Ok(_) => {
            health.checks.insert("database".to_string(), "healthy".to_string());
        },
        Err(_) => {
            health.status = "unhealthy".to_string();
            health.checks.insert("database".to_string(), "failed".to_string());
        }
    }

    // JWT secret validation
    if std::env::var("OAUTH_JWT_SECRET").is_ok() {
        health.checks.insert("jwt_secret".to_string(), "configured".to_string());
    } else {
        health.checks.insert("jwt_secret".to_string(), "missing".to_string());
    }

    Json(health)
}
```

---

## Migration Guide

### From OAuth 2.0 to OAuth 2.1

#### Breaking Changes

1. **PKCE is Now Mandatory**
   ```diff
   // Before (OAuth 2.0)
   const authUrl = `/oauth/authorize?response_type=code&client_id=${clientId}`;

   // After (OAuth 2.1)
   + const { codeVerifier, codeChallenge } = await generatePKCE();
   + const authUrl = `/oauth/authorize?response_type=code&client_id=${clientId}&code_challenge=${codeChallenge}&code_challenge_method=S256`;
   ```

2. **Password Grant Removed**
   ```diff
   // Before (OAuth 2.0) - DEPRECATED
   - fetch('/oauth/token', {
   -   method: 'POST',
   -   body: new URLSearchParams({
   -     grant_type: 'password',
   -     username: 'user@example.com',
   -     password: 'password123'
   -   })
   - })

   // After (OAuth 2.1) - Use authorization code flow
   + window.location.href = authorizationUrl;
   ```

3. **Implicit Grant Removed**
   ```diff
   // Before (OAuth 2.0) - DEPRECATED
   - response_type=token

   // After (OAuth 2.1) - Use authorization code + PKCE
   + response_type=code&code_challenge=...&code_challenge_method=S256
   ```

#### Migration Steps

1. **Update Client Applications**
   ```bash
   # Step 1: Update all clients to use PKCE
   # Step 2: Remove implicit flow implementations
   # Step 3: Replace password grant with authorization code
   # Step 4: Update redirect URI validation (exact matching)
   ```

2. **Database Schema Updates**
   ```sql
   -- OAuth 2.1 migration
   -- No schema changes required - PKCE fields already exist

   -- Optional: Clean up unused tokens from deprecated flows
   DELETE FROM oauth_access_tokens
   WHERE client_id IN (
     SELECT id FROM oauth_clients
     WHERE personal_access_client = false
     AND password_client = true
   );
   ```

3. **Configuration Updates**
   ```diff
   # .env updates
   + OAUTH_JWT_SECRET=your-strong-256-bit-secret
   + OAUTH_REQUIRE_PKCE=true  # Now mandatory
   - OAUTH_ENABLE_PASSWORD_GRANT=false  # Removed
   - OAUTH_ENABLE_IMPLICIT_GRANT=false  # Removed
   ```

### From Laravel Passport

#### API Compatibility

Most Laravel Passport APIs are supported with minimal changes:

```php
// Laravel Passport
Route::get('/user', function (Request $request) {
    return $request->user();
})->middleware('auth:api');
```

```rust
// Rustaxum equivalent
pub async fn get_user(
    headers: HeaderMap,
    State(pool): State<DbPool>,
) -> impl IntoResponse {
    let token = extract_bearer_token(&headers)?;
    let (access_token, claims) = TokenService::validate_token_and_scopes(
        &pool, &token, &[]
    )?;

    let user = UserService::find_by_id(&pool, &claims.sub.to_string())?;
    Json(user)
}
```

#### Feature Mapping

| Laravel Passport | Rustaxum OAuth | Status | Notes |
|------------------|----------------|---------|--------|
| `Passport::routes()` | `/oauth/*` routes | ✅ | Auto-registered |
| `Passport::tokensExpireIn()` | `OAUTH_ACCESS_TOKEN_TTL` | ✅ | Environment config |
| `Passport::personalAccessTokensExpireIn()` | Same TTL system | ✅ | Unified expiry |
| `Passport::enableImplicitGrant()` | **Removed** | ❌ | OAuth 2.1 compliance |
| `Passport::enablePasswordGrant()` | **Removed** | ❌ | OAuth 2.1 compliance |
| Passport scopes | OAuth scopes | ✅ | Enhanced with wildcards |
| Personal access tokens | Personal access tokens | ✅ | Full compatibility |

#### Migration Script

```bash
#!/bin/bash
# Laravel to Rustaxum OAuth migration

echo "Migrating from Laravel Passport to Rustaxum OAuth 2.1..."

# Export Laravel data
php artisan passport:export-clients > clients.json
php artisan passport:export-tokens > tokens.json

# Import to Rustaxum
cargo run --bin artisan passport import-clients clients.json
cargo run --bin artisan passport import-tokens tokens.json

# Verify migration
cargo run --bin artisan passport validate-migration

echo "Migration completed. Please test all OAuth flows."
```

---

## Security Best Practices

### Production Deployment

#### 1. JWT Secret Management
```bash
# Generate strong JWT secret (256 bits minimum)
openssl rand -hex 32

# Use environment-specific secrets
export OAUTH_JWT_SECRET_PROD="your-production-secret"
export OAUTH_JWT_SECRET_STAGING="your-staging-secret"
```

#### 2. TLS/SSL Configuration
```nginx
# Nginx configuration for OAuth endpoints
server {
    listen 443 ssl http2;
    server_name auth.yourcompany.com;

    ssl_certificate /path/to/certificate.crt;
    ssl_certificate_key /path/to/private.key;

    # OAuth security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;

    location /oauth/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # OAuth-specific security
        proxy_set_header X-OAuth-Client-IP $remote_addr;
        proxy_set_header X-OAuth-User-Agent $http_user_agent;
    }
}
```

#### 3. Rate Limiting
```rust
// Rate limiting middleware
pub async fn oauth_rate_limit(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let client_ip = addr.ip().to_string();

    // Different limits for different endpoints
    let limit = match request.uri().path() {
        "/oauth/token" => 10,      // 10 requests per minute
        "/oauth/authorize" => 20,   // 20 requests per minute
        "/oauth/introspect" => 100, // 100 requests per minute
        _ => 50,
    };

    if !RateLimiter::check_limit(&client_ip, limit).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}
```

#### 4. Audit Logging
```rust
// Comprehensive OAuth audit logging
pub async fn log_oauth_event(
    event_type: &str,
    client_id: Option<&str>,
    user_id: Option<&str>,
    details: Option<Value>,
    risk_level: RiskLevel,
) -> Result<()> {
    let audit_entry = AuditLogEntry {
        id: Ulid::new(),
        event_type: event_type.to_string(),
        client_id: client_id.map(String::from),
        user_id: user_id.map(String::from),
        ip_address: get_client_ip(),
        user_agent: get_user_agent(),
        details,
        risk_level,
        timestamp: Utc::now(),
    };

    AuditLogService::create_entry(audit_entry).await?;

    // Alert on high-risk events
    if risk_level == RiskLevel::High {
        SecurityAlertService::send_alert(event_type, &audit_entry).await?;
    }

    Ok(())
}
```

### Security Monitoring

#### 1. Suspicious Activity Detection
```rust
pub struct SecurityMonitor;

impl SecurityMonitor {
    pub async fn analyze_oauth_activity(event: &OAuthEvent) -> SecurityAnalysis {
        let mut flags = Vec::new();

        // Multiple failed authentications
        if Self::check_failed_auth_pattern(event).await {
            flags.push("repeated_failures".to_string());
        }

        // Unusual client behavior
        if Self::check_client_anomalies(event).await {
            flags.push("client_anomaly".to_string());
        }

        // Geographic anomalies
        if Self::check_geographic_anomalies(event).await {
            flags.push("geo_anomaly".to_string());
        }

        // Token abuse patterns
        if Self::check_token_abuse(event).await {
            flags.push("token_abuse".to_string());
        }

        SecurityAnalysis { flags, risk_score: Self::calculate_risk(&flags) }
    }
}
```

#### 2. Real-time Alerting
```rust
pub async fn security_alert_webhook(
    event_type: &str,
    details: &AuditLogEntry,
) -> Result<()> {
    let alert = SecurityAlert {
        severity: "HIGH",
        event_type,
        client_id: details.client_id.as_deref(),
        user_id: details.user_id.as_deref(),
        timestamp: details.timestamp,
        details: json!({
            "ip_address": details.ip_address,
            "user_agent": details.user_agent,
            "risk_indicators": details.details
        }),
    };

    // Send to security team
    SecurityNotificationService::send_alert(alert).await?;

    Ok(())
}
```

---

## Conclusion

This OAuth 2.1 implementation provides a secure, scalable, and standards-compliant authorization server for modern applications. With its emphasis on security best practices, multi-tenant support, and comprehensive API coverage, it serves as a robust foundation for enterprise authentication and authorization needs.

Key advantages:

- **Security First**: OAuth 2.1 compliance with mandatory PKCE and removed vulnerable grants
- **Enterprise Ready**: Multi-tenant support with organization-scoped access control
- **Developer Friendly**: Laravel Passport-compatible API with comprehensive documentation
- **Production Tested**: Built with Rust for performance, reliability, and memory safety
- **Extensible**: Modular architecture supporting custom grants and middleware

For support and contributions, please refer to the project repository and community guidelines.

---

**Document Version**: 1.0
**Last Updated**: January 2025
**OAuth 2.1 Draft**: [draft-ietf-oauth-v2-1-13](https://datatracker.ietf.org/doc/draft-ietf-oauth-v2-1/)