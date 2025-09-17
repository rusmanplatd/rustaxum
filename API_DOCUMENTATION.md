# RustAxum Authentication API Documentation

## Overview

This API provides a complete authentication system with user registration, login, password management, and token handling.

## Base URL

```
http://localhost:3000/api
```

## Authentication

Most endpoints require a valid JWT token in the Authorization header:

```
Authorization: Bearer <your-jwt-token>
```

## Endpoints

### User Registration

**POST** `/auth/register`

Register a new user account.

**Request Body:**
```json
{
  "name": "John Doe",
  "email": "john@example.com",
  "password": "SecurePass123!"
}
```

**Response (201 Created):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "name": "John Doe",
    "email": "john@example.com",
    "email_verified_at": null,
    "last_login_at": "2024-01-01T10:00:00Z",
    "created_at": "2024-01-01T10:00:00Z",
    "updated_at": "2024-01-01T10:00:00Z"
  },
  "expires_at": "2024-01-02T10:00:00Z"
}
```

### User Login

**POST** `/auth/login`

Authenticate a user and receive a JWT token.

**Request Body:**
```json
{
  "email": "john@example.com",
  "password": "SecurePass123!"
}
```

**Response (200 OK):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "name": "John Doe",
    "email": "john@example.com",
    "email_verified_at": null,
    "last_login_at": "2024-01-01T10:00:00Z",
    "created_at": "2024-01-01T10:00:00Z",
    "updated_at": "2024-01-01T10:00:00Z"
  },
  "expires_at": "2024-01-02T10:00:00Z"
}
```

### Forgot Password

**POST** `/auth/forgot-password`

Request a password reset token to be sent via email.

**Request Body:**
```json
{
  "email": "john@example.com"
}
```

**Response (200 OK):**
```json
{
  "message": "If an account with that email exists, we have sent a password reset link."
}
```

### Reset Password

**POST** `/auth/reset-password`

Reset password using the token received via email.

**Request Body:**
```json
{
  "token": "reset-token-from-email",
  "password": "NewSecurePass123!",
  "password_confirmation": "NewSecurePass123!"
}
```

**Response (200 OK):**
```json
{
  "message": "Password has been reset successfully."
}
```

### Change Password

**PUT** `/auth/change-password`

Change password for the authenticated user.

**Headers:**
```
Authorization: Bearer <your-jwt-token>
```

**Request Body:**
```json
{
  "current_password": "OldPassword123!",
  "new_password": "NewSecurePass123!",
  "password_confirmation": "NewSecurePass123!"
}
```

**Response (200 OK):**
```json
{
  "message": "Password changed successfully."
}
```

### Logout

**POST** `/auth/logout`

Logout and revoke the current JWT token.

**Headers:**
```
Authorization: Bearer <your-jwt-token>
```

**Response (200 OK):**
```json
{
  "message": "Token revoked successfully."
}
```

### Revoke Token

**DELETE** `/auth/revoke-token`

Manually revoke the current JWT token.

**Headers:**
```
Authorization: Bearer <your-jwt-token>
```

**Response (200 OK):**
```json
{
  "message": "Token revoked successfully."
}
```

### Get Users

**GET** `/users`

Get a list of all users (requires authentication).

**Headers:**
```
Authorization: Bearer <your-jwt-token>
```

**Response (200 OK):**
```json
[
  {
    "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "name": "John Doe",
    "email": "john@example.com"
  },
  {
    "id": "01ARZ3NDEKTSV4RRFFQ69G5FB0",
    "name": "Jane Smith",
    "email": "jane@example.com"
  }
]
```

### Get User by ID

**GET** `/users/{id}`

Get a specific user by their ID (requires authentication).

**Headers:**
```
Authorization: Bearer <your-jwt-token>
```

**Response (200 OK):**
```json
{
  "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "name": "John Doe",
  "email": "john@example.com"
}
```

## Error Responses

All endpoints may return error responses in the following format:

**400 Bad Request / 401 Unauthorized / 404 Not Found:**
```json
{
  "error": "Error message describing what went wrong"
}
```

## Password Requirements

Passwords must meet the following criteria:
- At least 8 characters long
- Maximum 128 characters
- At least one lowercase letter
- At least one uppercase letter
- At least one number
- At least one special character (!@#$%^&*()_+-=[]{}|;:,.<>?)

## Account Lockout

Accounts are temporarily locked after 5 failed login attempts for 30 minutes.

## Token Management

- JWT tokens expire after 24 hours
- Revoked tokens are stored in a blacklist
- Expired blacklist entries are periodically cleaned up

## Email Notifications

The system sends email notifications for:
- Password reset requests
- Welcome messages for new users
- Password change confirmations

*Note: Email sending is currently logged to console. Implement actual SMTP configuration for production use.*