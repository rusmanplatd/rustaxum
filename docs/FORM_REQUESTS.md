# FormRequest System

This document describes the FormRequest system implemented in RustAxum, which provides Laravel-like form validation for HTTP requests.

## Overview

The FormRequest system allows you to define validation rules, custom messages, and authorization logic for HTTP requests in a clean, reusable way. It automatically validates incoming JSON requests and returns structured error responses when validation fails.

## Features

- **Automatic Validation**: Requests are validated before reaching your controller methods
- **Structured Error Responses**: Validation errors are returned in a consistent JSON format
- **Custom Messages**: Define custom error messages for validation rules
- **Authorization**: Implement authorization logic directly in the request
- **Axum Integration**: Works seamlessly as an Axum extractor
- **All Laravel Validation Rules**: Supports all common validation rules (required, email, min, max, etc.)

## Basic Usage

### 1. Define a FormRequest

```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use crate::app::http::form_request::FormRequest;
use crate::app::validation::{Rule, required, email, min, confirmed};
use crate::impl_form_request_extractor;

#[derive(Deserialize, Serialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub password_confirmation: String,
}

#[async_trait]
impl FormRequest for RegisterRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("name", vec![required(), min(2)]);
        rules.insert("email", vec![required(), email()]);
        rules.insert("password", vec![required(), min(8)]);
        rules.insert("password_confirmation", vec![required(), confirmed()]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("name.required", "Name is required");
        messages.insert("email.email", "Please enter a valid email address");
        messages.insert("password.min", "Password must be at least 8 characters");
        messages
    }
}

// This macro generates the Axum FromRequest implementation
impl_form_request_extractor!(RegisterRequest);
```

### 2. Use in Controllers

```rust
use axum::{response::IntoResponse, Json, http::StatusCode};
use serde_json::json;

pub async fn register(request: RegisterRequest) -> impl IntoResponse {
    // Request is automatically validated before reaching this point
    // If validation fails, a 422 response is returned automatically

    println!("Registering user: {}", request.email);

    // Your business logic here
    // ...

    (StatusCode::CREATED, Json(json!({
        "message": "User registered successfully",
        "user": {
            "name": request.name,
            "email": request.email
        }
    })))
}
```

### 3. Add Routes

```rust
use axum::{routing::post, Router};

let app = Router::new()
    .route("/register", post(register));
```

## Available Validation Rules

The FormRequest system supports all validation rules from the existing validator:

### Basic Rules
- `required()` - Field must be present and not empty
- `string()` - Field must be a string
- `numeric()` - Field must be numeric
- `integer()` - Field must be an integer
- `boolean()` - Field must be boolean
- `email()` - Field must be a valid email
- `url()` - Field must be a valid URL

### Size Rules
- `min(value)` - Minimum length/value
- `max(value)` - Maximum length/value
- `between(min, max)` - Value between min and max
- `size(value)` - Exact size/length

### Comparison Rules
- `confirmed()` - Field must match field_confirmation
- `same(field)` - Field must match another field
- `different(field)` - Field must be different from another field

### List Rules
- `in_list(values)` - Field must be in the given list
- `not_in(values)` - Field must not be in the given list

### Pattern Rules
- `alpha()` - Only alphabetic characters
- `alpha_num()` - Only alphanumeric characters
- `alpha_dash()` - Only alphanumeric, dash, and underscore
- `regex(pattern)` - Must match regex pattern

### Date Rules
- `date()` - Must be a valid date
- `date_format(format)` - Must match date format
- `before(date)` - Must be before given date
- `after(date)` - Must be after given date

## Advanced Features

### Custom Authorization

```rust
impl FormRequest for AdminRequest {
    fn authorize(&self) -> bool {
        // Implement your authorization logic
        // Access to headers, user info, etc. would need to be passed in
        true // or false to deny
    }

    // ... other methods
}
```

### Data Preparation

```rust
impl FormRequest for SearchRequest {
    fn prepare_for_validation(&mut self) {
        // Modify data before validation
        if let Some(ref mut email) = self.email {
            *email = email.to_lowercase();
        }
    }

    // ... other methods
}
```

### Custom Error Handling

```rust
impl FormRequest for CustomRequest {
    fn failed_validation(&self, errors: ValidationErrors) -> ValidationErrorResponse {
        ValidationErrorResponse {
            message: "Custom validation failed".to_string(),
            errors: errors.get_messages(),
        }
    }

    // ... other methods
}
```

## Response Format

When validation fails, the FormRequest system returns a `422 Unprocessable Entity` response with the following JSON structure:

```json
{
  "message": "The given data was invalid.",
  "errors": {
    "email": [
      "Email is required"
    ],
    "password": [
      "Password must be at least 8 characters"
    ]
  }
}
```

## Generating FormRequests with Artisan

You can generate new FormRequest files using the Artisan CLI:

```bash
cargo run --bin artisan -- make request CreateUserRequest
```

This will create a new file in `src/app/http/requests/` with a basic FormRequest template.

## Example: Complete User Management

```rust
// src/app/http/requests/user_requests.rs
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use crate::app::http::form_request::FormRequest;
use crate::app::validation::{Rule, required, email, min, max, string, numeric};
use crate::impl_form_request_extractor;

#[derive(Deserialize, Serialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub age: Option<u32>,
}

#[async_trait]
impl FormRequest for CreateUserRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("name", vec![required(), string(), min(2), max(100)]);
        rules.insert("email", vec![required(), email()]);
        rules.insert("password", vec![required(), string(), min(8)]);
        rules.insert("age", vec![numeric(), min(18), max(120)]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("name.required", "Name is required");
        messages.insert("name.min", "Name must be at least 2 characters");
        messages.insert("email.email", "Please enter a valid email address");
        messages.insert("password.min", "Password must be at least 8 characters");
        messages.insert("age.min", "You must be at least 18 years old");
        messages
    }

    fn prepare_for_validation(&mut self) {
        // Normalize email to lowercase
        self.email = self.email.to_lowercase();
    }
}

impl_form_request_extractor!(CreateUserRequest);

// Controller usage
pub async fn create_user(request: CreateUserRequest) -> impl IntoResponse {
    // Request is automatically validated
    // Business logic here...

    (StatusCode::CREATED, Json(json!({
        "message": "User created successfully",
        "user": {
            "name": request.name,
            "email": request.email,
            "age": request.age
        }
    })))
}
```

## Best Practices

1. **Organize by Feature**: Group related FormRequests in modules (e.g., `auth_requests.rs`, `user_requests.rs`)

2. **Use Descriptive Names**: Name your FormRequests clearly (`CreateUserRequest`, `UpdatePostRequest`)

3. **Custom Messages**: Always provide user-friendly error messages

4. **Authorization**: Implement authorization in the FormRequest when possible

5. **Data Preparation**: Use `prepare_for_validation` to normalize data (lowercase emails, trim strings, etc.)

6. **Optional Fields**: Use `Option<T>` for optional fields and handle them appropriately in validation rules

7. **Reusable Rules**: Extract common validation patterns into reusable functions

## Migration from Manual Validation

If you're currently using manual validation, you can easily migrate to FormRequests:

### Before (Manual Validation)
```rust
pub async fn create_user(Json(payload): Json<CreateUserPayload>) -> impl IntoResponse {
    // Manual validation
    if payload.name.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Name is required"})));
    }

    if !payload.email.contains('@') {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid email"})));
    }

    // Business logic...
}
```

### After (FormRequest)
```rust
pub async fn create_user(request: CreateUserRequest) -> impl IntoResponse {
    // Validation is automatic!
    // Business logic...
}
```

The FormRequest system provides a much cleaner, more maintainable approach to request validation while maintaining Laravel-like familiarity.