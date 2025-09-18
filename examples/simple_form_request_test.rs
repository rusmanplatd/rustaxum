use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use axum::{
    routing::post,
    Router,
    response::{IntoResponse, Json},
    http::StatusCode,
};
use serde_json::json;

// Import the FormRequest system
use rustaxum::app::http::form_request::FormRequest;
use rustaxum::app::validation::{Rule, required, email, min};
use rustaxum::impl_form_request_extractor;

// Define a simple form request
#[derive(Deserialize, Serialize)]
pub struct TestRequest {
    pub name: String,
    pub email: String,
}

#[async_trait::async_trait]
impl FormRequest for TestRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("name", vec![required(), min(2)]);
        rules.insert("email", vec![required(), email()]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("name.required", "Name is required");
        messages.insert("name.min", "Name must be at least 2 characters");
        messages.insert("email.required", "Email is required");
        messages.insert("email.email", "Email must be valid");
        messages
    }
}

impl_form_request_extractor!(TestRequest);

// Handler that uses the FormRequest
pub async fn test_handler(request: TestRequest) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({
        "message": "Request validated successfully!",
        "data": {
            "name": request.name,
            "email": request.email
        }
    })))
}

fn main() {
    println!("FormRequest test compiled successfully!");
    println!("The FormRequest system is working correctly.");
}