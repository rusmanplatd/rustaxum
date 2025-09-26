# Authentication System Demo

This document demonstrates how to use both JWT and Session-based authentication in the RustAxum application.

## 🔐 Authentication Methods Supported

The system now supports **dual authentication methods**:

1. **JWT Bearer Token Authentication** (stateless)
2. **Session-based Authentication** (stateful)

## 🚀 Starting the Server

```bash
# Build the project
cargo build

# Start with Docker Compose (recommended)
docker-compose up -d

# Or start directly
cargo run
```

The server will be available at `http://localhost:3000`

## 📋 API Endpoints

### JWT Authentication (Stateless)

#### Register with JWT

```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "John Doe",
    "email": "john@example.com",
    "password": "password123"
  }'
```

Response:

```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "refresh_token": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "user": {
    "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "name": "John Doe",
    "email": "john@example.com"
  },
  "expires_at": "2024-01-02T00:00:00Z",
  "refresh_expires_at": "2024-01-08T00:00:00Z"
}
```

#### Login with JWT

```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john@example.com",
    "password": "password123"
  }'
```

#### Access Protected Route with JWT

```bash
curl -X GET http://localhost:3000/api/users \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### Session Authentication (Stateful)

#### Register with Session

```bash
curl -X POST http://localhost:3000/api/auth/session/register \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{
    "name": "Jane Doe",
    "email": "jane@example.com",
    "password": "password123"
  }'
```

Response:

```json
{
  "message": "Registration successful",
  "user": {
    "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "name": "Jane Doe",
    "email": "jane@example.com"
  },
  "session_id": "abc123..."
}
```

#### Login with Session

```bash
curl -X POST http://localhost:3000/api/auth/session/login \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{
    "email": "jane@example.com",
    "password": "password123"
  }'
```

#### Get Current User (Session)

```bash
curl -X GET http://localhost:3000/api/auth/session/user \
  -b cookies.txt
```

#### Access Protected Route with Session

```bash
curl -X GET http://localhost:3000/api/users \
  -b cookies.txt
```

#### Logout (Session)

```bash
curl -X POST http://localhost:3000/api/auth/session/logout \
  -b cookies.txt
```

## 🛡️ Middleware Protection

### Route Protection Levels

1. **Public Routes** (no authentication required):

   - `POST /api/auth/register`
   - `POST /api/auth/login`
   - `POST /api/auth/session/register`
   - `POST /api/auth/session/login`
   - `POST /api/auth/forgot-password`
   - `POST /api/auth/reset-password`
   - `GET /api/docs/*`

2. **Protected Routes** (authentication required):
   - `GET /api/users/*`
   - `POST /api/countries/*`
   - `GET /api/roles/*`
   - `PUT /api/auth/change-password`
   - `POST /api/auth/session/logout`
   - `GET /api/auth/session/user`
   - All other CRUD endpoints

### Middleware Features

- **Unified Authentication**: Single middleware supports both JWT and Session
- **Automatic Detection**: Tries JWT first, falls back to session
- **Laravel-like Guards**: `auth`, `guest`, and `optional` middleware
- **User Context**: Authenticated user info available in request extensions

## 🔧 Configuration

### Session Configuration

Sessions are configured via environment variables:

```env
SESSION_DRIVER=database
SESSION_LIFETIME=120
SESSION_ENCRYPT=false
SESSION_COOKIE=rustaxum_session
SESSION_SECURE=false
SESSION_HTTP_ONLY=true
SESSION_SAME_SITE=lax
```

### JWT Configuration

JWT tokens are configured via:

```env
JWT_SECRET=your-jwt-secret-key
```

## 📊 Testing Both Methods

### Test JWT Authentication

```bash
# Register and get JWT
JWT_RESPONSE=$(curl -s -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"name":"JWT User","email":"jwt@test.com","password":"test123"}')

JWT_TOKEN=$(echo $JWT_RESPONSE | jq -r '.access_token')

# Access protected endpoint
curl -X GET http://localhost:3000/api/users \
  -H "Authorization: Bearer $JWT_TOKEN"
```

### Test Session Authentication

```bash
# Register and get session cookie
curl -X POST http://localhost:3000/api/auth/session/register \
  -H "Content-Type: application/json" \
  -c session_cookies.txt \
  -d '{"name":"Session User","email":"session@test.com","password":"test123"}'

# Access protected endpoint
curl -X GET http://localhost:3000/api/users \
  -b session_cookies.txt
```

## 🔄 Mixed Authentication Support

The same protected routes work with **both** authentication methods:

```bash
# This works with JWT
curl -X GET http://localhost:3000/api/users \
  -H "Authorization: Bearer $JWT_TOKEN"

# This also works with Session
curl -X GET http://localhost:3000/api/users \
  -b session_cookies.txt
```

## 🚨 Error Handling

### Authentication Errors

```json
{
  "error": "Unauthorized",
  "message": "Authentication required. Please provide a valid Bearer token or valid session."
}
```

### Already Authenticated (Guest Routes)

```json
{
  "error": "Forbidden",
  "message": "Already authenticated via JWT. This endpoint is for guests only."
}
```

## 🎯 Laravel Similarity

The authentication system mimics Laravel's auth system:

- **Routes**: Similar to Laravel's `auth:api` and `auth:web` guards
- **Middleware**: `auth`, `guest`, `optional` middleware types
- **Session Management**: Laravel-like session store with flash data
- **User Context**: Available via request extensions (similar to `$request->user()`)

## 🔍 Debugging

Enable debug logging to see authentication flow:

```bash
RUST_LOG=debug cargo run
```

This will show:

- Session creation/retrieval
- JWT validation attempts
- Authentication success/failure
- Route protection status
