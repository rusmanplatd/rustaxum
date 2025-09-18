use axum::{
    routing::post,
    Router,
    response::{IntoResponse, Json},
    http::StatusCode,
};
use serde_json::json;

use rustaxum::app::http::requests::{RegisterRequest, LoginRequest, UpdateUserRequest, ContactFormRequest};

/// Example controller function using FormRequest
pub async fn register_user(request: RegisterRequest) -> impl IntoResponse {
    // The request is automatically validated before reaching this point
    // If validation fails, a 422 response with validation errors is returned automatically

    println!("Register request received:");
    println!("Name: {}", request.name);
    println!("Email: {}", request.email);

    // Here you would typically:
    // 1. Create the user in the database
    // 2. Send verification email
    // 3. Return success response

    (StatusCode::CREATED, Json(json!({
        "message": "User registered successfully",
        "user": {
            "name": request.name,
            "email": request.email
        }
    })))
}

/// Example login controller
pub async fn login_user(request: LoginRequest) -> impl IntoResponse {
    println!("Login attempt for: {}", request.email);

    // Here you would typically:
    // 1. Verify credentials
    // 2. Generate JWT token
    // 3. Return token response

    (StatusCode::OK, Json(json!({
        "message": "Login successful",
        "token": "example-jwt-token",
        "remember": request.remember
    })))
}

/// Example update user profile controller
pub async fn update_user_profile(request: UpdateUserRequest) -> impl IntoResponse {
    println!("Updating user profile");

    // The FormRequest handles all validation automatically
    // Optional fields are properly handled

    (StatusCode::OK, Json(json!({
        "message": "Profile updated successfully",
        "user": {
            "name": request.name,
            "email": request.email,
            "bio": request.bio,
            "phone": request.phone,
            "age": request.age
        }
    })))
}

/// Example contact form controller
pub async fn submit_contact_form(request: ContactFormRequest) -> impl IntoResponse {
    println!("Contact form submitted by: {}", request.name);

    // Here you would typically:
    // 1. Save to database
    // 2. Send notification email
    // 3. Return confirmation

    (StatusCode::CREATED, Json(json!({
        "message": "Thank you for your message. We'll get back to you soon!",
        "id": "contact-123"
    })))
}

/// Create example router with FormRequest endpoints
pub fn create_example_router() -> Router {
    Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user))
        .route("/profile", post(update_user_profile))
        .route("/contact", post(submit_contact_form))
}

#[tokio::main]
async fn main() {
    let app = create_example_router();

    println!("FormRequest example server running on http://0.0.0.0:3001");
    println!("\nTry these endpoints:");
    println!("POST /register - Register a new user");
    println!("POST /login - Login user");
    println!("POST /profile - Update user profile");
    println!("POST /contact - Submit contact form");
    println!("\nExample request body for /register:");
    println!(r#"{{
  "name": "John Doe",
  "email": "john@example.com",
  "password": "secretpassword123",
  "password_confirmation": "secretpassword123"
}}"#);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}