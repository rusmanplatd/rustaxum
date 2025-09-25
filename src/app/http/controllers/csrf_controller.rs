use axum::{
    extract::Request,
    response::{Html, Json},
    Extension,
    http::StatusCode,
};
use serde_json::{json, Value};

use crate::app::services::csrf_service::CSRFService;
use crate::app::services::session::SessionStore;
use crate::app::helpers::csrf_helpers::CSRFHelpers;

pub struct CSRFController;

impl CSRFController {
    /// Get CSRF token for the current session
    pub async fn token(
        Extension(session_store): Extension<SessionStore>,
    ) -> Result<Json<Value>, StatusCode> {
        let csrf_service = CSRFService::new();

        match csrf_service.token(&session_store).await {
            Ok(token) => Ok(Json(json!({
                "token": token,
                "token_name": csrf_service.token_name(),
                "header_name": csrf_service.header_name()
            }))),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    /// Show a test form with CSRF protection
    pub async fn form(
        Extension(session_store): Extension<SessionStore>,
    ) -> Result<Html<String>, StatusCode> {
        match CSRFHelpers::csrf_field(&session_store).await {
            Ok(csrf_field) => {
                let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>CSRF Protection Test</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .form-container {{ max-width: 600px; margin: 0 auto; }}
        form {{ background: #f9f9f9; padding: 20px; border-radius: 8px; }}
        input, textarea {{ width: 100%; padding: 8px; margin: 5px 0; }}
        button {{ background: #007bff; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; }}
        button:hover {{ background: #0056b3; }}
        .success {{ background: #d4edda; color: #155724; padding: 10px; border-radius: 4px; margin: 10px 0; }}
        .error {{ background: #f8d7da; color: #721c24; padding: 10px; border-radius: 4px; margin: 10px 0; }}
    </style>
</head>
<body>
    <div class="form-container">
        <h1>CSRF Protection Test</h1>

        <h2>Test Form (Protected by CSRF)</h2>
        <form action="/csrf/test" method="POST">
            {}
            <label>Name:</label>
            <input type="text" name="name" placeholder="Enter your name" required>

            <label>Message:</label>
            <textarea name="message" placeholder="Enter a message" required></textarea>

            <button type="submit">Submit Form</button>
        </form>

        <h2>AJAX Test</h2>
        <button onclick="testAjaxRequest()">Test AJAX Request</button>
        <div id="ajax-result"></div>

        <h2>API Information</h2>
        <p><strong>Token Endpoint:</strong> GET /csrf/token</p>
        <p><strong>Test Form:</strong> POST /csrf/test</p>
        <p><strong>Protected API:</strong> POST /csrf/api-test</p>
    </div>

    <script>
        async function testAjaxRequest() {{
            try {{
                // Get CSRF token
                const tokenResponse = await fetch('/csrf/token');
                const tokenData = await tokenResponse.json();

                // Make protected request
                const response = await fetch('/csrf/api-test', {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/json',
                        [tokenData.header_name]: tokenData.token
                    }},
                    body: JSON.stringify({{
                        message: 'Hello from AJAX with CSRF protection!'
                    }})
                }});

                const result = await response.json();
                document.getElementById('ajax-result').innerHTML =
                    '<div class="success">AJAX request successful: ' + result.message + '</div>';
            }} catch (error) {{
                document.getElementById('ajax-result').innerHTML =
                    '<div class="error">AJAX request failed: ' + error.message + '</div>';
            }}
        }}
    </script>
</body>
</html>
                "#, csrf_field);

                Ok(Html(html))
            },
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    /// Handle test form submission
    pub async fn test_form(
        Extension(_session_store): Extension<SessionStore>,
        _request: Request,
    ) -> Result<Html<String>, StatusCode> {
        // If we reach here, CSRF validation passed
        let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>CSRF Test Success</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; text-align: center; }
        .success { background: #d4edda; color: #155724; padding: 20px; border-radius: 8px; margin: 20px 0; }
    </style>
</head>
<body>
    <h1>CSRF Protection Test</h1>
    <div class="success">
        ✅ Form submission successful! CSRF protection is working correctly.
    </div>
    <p><a href="/csrf/form">← Back to Test Form</a></p>
</body>
</html>
        "#;

        Ok(Html(html.to_string()))
    }

    /// Test API endpoint with CSRF protection
    pub async fn test_api(
        Extension(_session_store): Extension<SessionStore>,
    ) -> Result<Json<Value>, StatusCode> {
        // If we reach here, CSRF validation passed
        Ok(Json(json!({
            "success": true,
            "message": "API request successful! CSRF protection is working.",
            "timestamp": chrono::Utc::now().to_rfc3339()
        })))
    }

    /// Regenerate CSRF token (useful after login/logout)
    pub async fn regenerate(
        Extension(session_store): Extension<SessionStore>,
    ) -> Result<Json<Value>, StatusCode> {
        let csrf_service = CSRFService::new();

        match csrf_service.regenerate_token(&session_store).await {
            Ok(new_token) => Ok(Json(json!({
                "success": true,
                "token": new_token,
                "message": "CSRF token regenerated successfully"
            }))),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}