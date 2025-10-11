use axum::response::IntoResponse;
use serde_json::json;
use crate::app::http::responses::template_response::TemplateResponse;

pub async fn index() -> impl IntoResponse {
    let data = json!({
        "title": "Home",
        "description": "Welcome to RustAxum - A modern Rust web framework",
        "features": [
            {
                "name": "Authentication",
                "description": "Secure JWT-based authentication system",
                "icon": "fas fa-shield-alt"
            },
            {
                "name": "Database ORM",
                "description": "Diesel ORM with PostgreSQL support",
                "icon": "fas fa-database"
            },
            {
                "name": "Template Engine",
                "description": "Handlebars templating with layout support",
                "icon": "fas fa-code"
            },
            {
                "name": "OAuth 2.1",
                "description": "Complete OAuth 2.1 authorization server",
                "icon": "fas fa-key"
            },
            {
                "name": "Email System",
                "description": "Rich email templates and SMTP support",
                "icon": "fas fa-envelope"
            },
            {
                "name": "Background Jobs",
                "description": "Async job processing with Redis",
                "icon": "fas fa-tasks"
            }
        ]
    });

    TemplateResponse::new("pages/home", &data)
}