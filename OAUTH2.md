# OAuth2 / Passport Implementation

This Laravel Passport-inspired OAuth2 implementation provides a complete OAuth2 server for your Rust Axum application with improvements over the original PHP version.

## Features

### Core OAuth2 Features
- **Authorization Code Flow** - Full OAuth2 authorization code flow with PKCE support
- **Client Credentials Flow** - For server-to-server authentication
- **Refresh Tokens** - Automatic token refresh capabilities
- **Personal Access Tokens** - GitHub-style personal access tokens
- **Scope-based Authorization** - Fine-grained permission control
- **Token Introspection** - RFC 7662 compliant token introspection
- **Token Revocation** - RFC 7009 compliant token revocation

### Improvements Over Laravel Passport
- **Type Safety** - Full Rust type safety throughout the implementation
- **Performance** - Built on high-performance Axum and SQLx
- **Security** - Secure by default with modern cryptographic standards
- **Memory Safety** - Rust's memory safety guarantees
- **Concurrent** - Built for high-concurrency workloads
- **ULID Support** - Uses ULIDs instead of incremental IDs for better security
- **Query Builder Integration** - Seamless integration with the custom query builder

## Database Schema

The implementation uses the following database tables:

- `oauth_clients` - OAuth2 client applications
- `oauth_access_tokens` - Issued access tokens
- `oauth_refresh_tokens` - Refresh tokens for token renewal
- `oauth_auth_codes` - Authorization codes for the auth code flow
- `oauth_scopes` - Available permission scopes
- `oauth_personal_access_clients` - Special clients for personal access tokens

## Installation

### 1. Run Migrations

```bash
cargo run --bin artisan migrate
```

This will create all necessary OAuth2 tables.

### 2. Install Passport

```bash
cargo run --bin artisan passport install
```

This command will:
- Verify OAuth2 tables exist
- Create a personal access client
- Display setup instructions

## Client Management

### Create a Client

```bash
# Standard OAuth2 client
cargo run --bin artisan passport client --name "My App" --redirect-uris "http://localhost:3000/callback,http://localhost:3000/auth/callback"

# Personal access client
cargo run --bin artisan passport client --name "Personal Access" --redirect-uris "http://localhost" --personal

# Password grant client
cargo run --bin artisan passport client --name "Mobile App" --redirect-uris "http://localhost" --password
```

### List Clients

```bash
cargo run --bin artisan passport client:list
```

### Revoke a Client

```bash
cargo run --bin artisan passport client:revoke <client-id>
```

### Delete a Client

```bash
cargo run --bin artisan passport client:delete <client-id>
```

### Regenerate Client Secret

```bash
cargo run --bin artisan passport client:secret <client-id>
```

## Scope Management

### Create a Scope

```bash
cargo run --bin artisan passport scope:create read --description "Read access to resources"
cargo run --bin artisan passport scope:create write --description "Write access to resources"
cargo run --bin artisan passport scope:create admin --description "Administrative access" --default
```

### List Scopes

```bash
cargo run --bin artisan passport scope:list
```

### Delete a Scope

```bash
cargo run --bin artisan passport scope:delete <scope-name-or-id>
```

## Token Management

### List Tokens

```bash
# List tokens for a specific user
cargo run --bin artisan passport token:list --user-id <user-id>
```

### Revoke a Token

```bash
cargo run --bin artisan passport token:revoke <token-id>
```

### Revoke All User Tokens

```bash
cargo run --bin artisan passport token:revoke-all <user-id>
```

## API Endpoints

### Authorization Endpoints

- `GET /oauth/authorize` - Authorization endpoint
- `POST /oauth/token` - Token endpoint
- `POST /oauth/introspect` - Token introspection
- `POST /oauth/revoke` - Token revocation

### Client Management Endpoints

- `POST /oauth/clients` - Create client
- `GET /oauth/clients` - List clients
- `GET /oauth/clients/{id}` - Get client
- `PUT /oauth/clients/{id}` - Update client
- `DELETE /oauth/clients/{id}` - Delete client
- `POST /oauth/clients/{id}/regenerate-secret` - Regenerate secret

### Personal Access Token Endpoints

- `POST /oauth/personal-access-tokens` - Create personal access token
- `GET /oauth/personal-access-tokens` - List user's tokens
- `DELETE /oauth/personal-access-tokens/{id}` - Revoke token

## Authorization Code Flow

### 1. Authorization Request

Direct users to:
```
GET /oauth/authorize?response_type=code&client_id={CLIENT_ID}&redirect_uri={REDIRECT_URI}&scope={SCOPES}&state={STATE}
```

Optional PKCE parameters:
- `code_challenge` - PKCE code challenge
- `code_challenge_method` - Should be "S256"

### 2. Token Exchange

```bash
curl -X POST http://localhost:3000/oauth/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code&client_id={CLIENT_ID}&client_secret={CLIENT_SECRET}&code={CODE}&redirect_uri={REDIRECT_URI}"
```

With PKCE:
```bash
curl -X POST http://localhost:3000/oauth/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code&client_id={CLIENT_ID}&code={CODE}&redirect_uri={REDIRECT_URI}&code_verifier={CODE_VERIFIER}"
```

## Client Credentials Flow

```bash
curl -X POST http://localhost:3000/oauth/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=client_credentials&client_id={CLIENT_ID}&client_secret={CLIENT_SECRET}&scope={SCOPES}"
```

## Personal Access Tokens

### Create Token Programmatically

```rust
use rustaxum::app::services::oauth::TokenService;

let token_response = TokenService::create_personal_access_token(
    &pool,
    user_id,
    "My Token".to_string(),
    vec!["read".to_string(), "write".to_string()],
    Some(86400), // Expires in 1 day
).await?;

println!("Access Token: {}", token_response.access_token);
```

### Create via API

```bash
curl -X POST http://localhost:3000/oauth/personal-access-tokens \
  -H "Authorization: Bearer {USER_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"name": "My Token", "scopes": ["read", "write"], "expires_in_seconds": 86400}'
```

## Middleware and Route Protection

### Using OAuth Middleware

```rust
use axum::{Router, routing::get};
use rustaxum::app::middleware::oauth::{oauth_middleware, require_scope};

let app = Router::new()
    .route("/api/protected", get(protected_handler))
    .layer(axum::middleware::from_fn_with_state(pool.clone(), oauth_middleware))
    .route("/api/admin", get(admin_handler))
    .layer(axum::middleware::from_fn_with_state(pool.clone(), require_scope("admin")));
```

### Using Scope Macros

```rust
use rustaxum::{require_read_scope, require_write_scope, require_admin_scope, require_custom_scopes};

let app = Router::new()
    .route("/api/read", get(read_handler))
    .layer(axum::middleware::from_fn_with_state(pool.clone(), require_read_scope!()))
    .route("/api/write", post(write_handler))
    .layer(axum::middleware::from_fn_with_state(pool.clone(), require_write_scope!()))
    .route("/api/admin", delete(admin_handler))
    .layer(axum::middleware::from_fn_with_state(pool.clone(), require_admin_scope!()))
    .route("/api/custom", put(custom_handler))
    .layer(axum::middleware::from_fn_with_state(pool.clone(), require_custom_scopes!("custom", "special")));
```

### Accessing Token Information in Handlers

```rust
use axum::extract::Request;
use rustaxum::app::models::oauth::AccessToken;
use rustaxum::app::services::oauth::TokenClaims;

async fn protected_handler(req: Request) -> impl IntoResponse {
    let access_token = req.extensions().get::<AccessToken>().unwrap();
    let claims = req.extensions().get::<TokenClaims>().unwrap();

    println!("Token ID: {}", access_token.id);
    println!("User ID: {:?}", access_token.user_id);
    println!("Client ID: {}", claims.aud);
    println!("Scopes: {:?}", claims.scopes);

    Json(json!({"message": "Protected resource accessed"}))
}
```

## Token Introspection

```bash
curl -X POST http://localhost:3000/oauth/introspect \
  -H "Content-Type: application/json" \
  -d '{"token": "{ACCESS_TOKEN}"}'
```

Response:
```json
{
    "active": true,
    "scope": "read write",
    "client_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "username": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "exp": 1640995200,
    "iat": 1640908800,
    "sub": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "aud": "01ARZ3NDEKTSV4RRFFQ69G5FAV"
}
```

## Token Revocation

```bash
curl -X POST http://localhost:3000/oauth/revoke \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "token={ACCESS_TOKEN}"
```

## Security Best Practices

### 1. Use HTTPS in Production
Always use HTTPS for OAuth2 endpoints in production.

### 2. Validate Redirect URIs
The system validates redirect URIs against registered URIs for security.

### 3. Use PKCE for Public Clients
Always use PKCE (Proof Key for Code Exchange) for public clients like SPAs and mobile apps.

### 4. Short-lived Access Tokens
Use short-lived access tokens (1 hour default) with refresh tokens for better security.

### 5. Scope Principle of Least Privilege
Grant the minimum required scopes for each client.

### 6. Secure Client Secrets
Store client secrets securely and rotate them regularly.

## Configuration

Set these environment variables:

```env
# OAuth2 JWT signing secret (use a strong random string)
OAUTH_JWT_SECRET=your-jwt-secret-here

# Token expiration times (in seconds)
OAUTH_ACCESS_TOKEN_TTL=3600      # 1 hour
OAUTH_REFRESH_TOKEN_TTL=604800   # 7 days
OAUTH_AUTH_CODE_TTL=600          # 10 minutes
```

## Testing

Run OAuth2 tests:

```bash
cargo test oauth
```

## Troubleshooting

### Common Issues

1. **"No personal access client found"**
   - Run `cargo run --bin artisan passport install`

2. **"Invalid client credentials"**
   - Verify client ID and secret
   - Check if client is revoked

3. **"Insufficient scope"**
   - Token doesn't have required scope
   - Grant additional scopes to the client

4. **"Token expired"**
   - Refresh the access token using refresh token
   - Request new tokens through authorization flow

### Debugging

Enable debug logging:
```env
RUST_LOG=debug
```

This will show detailed OAuth2 flow information in the logs.

## Advanced Usage

### Custom Token Claims

You can extend the `TokenClaims` struct to include additional claims:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTokenClaims {
    // Standard claims
    pub sub: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
    pub scopes: Vec<String>,

    // Custom claims
    pub role: String,
    pub permissions: Vec<String>,
}
```

### Custom Scopes

Create application-specific scopes:

```bash
cargo run --bin artisan passport scope:create "user:read" --description "Read user data"
cargo run --bin artisan passport scope:create "user:write" --description "Modify user data"
cargo run --bin artisan passport scope:create "billing:read" --description "Read billing information"
```

### Rate Limiting

Consider implementing rate limiting on OAuth2 endpoints:

```rust
use tower::limit::RateLimitLayer;

let app = Router::new()
    .route("/oauth/token", post(token_handler))
    .layer(RateLimitLayer::new(10, Duration::from_secs(60))); // 10 requests per minute
```

This comprehensive OAuth2/Passport implementation provides a solid foundation for API authentication and authorization in your Rust Axum application.