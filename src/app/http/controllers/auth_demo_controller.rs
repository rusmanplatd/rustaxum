use axum::response::IntoResponse;
use serde_json::json;
use crate::app::http::responses::template_response::TemplateResponse;

pub struct AuthDemoController;

impl AuthDemoController {
    pub async fn login() -> impl IntoResponse {
        let data = json!({
            "title": "Sign In",
            "page_title": "Welcome Back",
            "subtitle": "Sign in to your account to continue",
            "header_icon": "fas fa-sign-in-alt",
            "registration_enabled": true,
            "social_login_enabled": true,
            "google_login": true,
            "github_login": true,
            "microsoft_login": false,
            "captcha_enabled": false,
            "is_development": true,
            "demo_email": "admin@example.com",
            "demo_password": "password",
            "footer_content": r#"
                <p class="text-muted mb-0">
                    <small>
                        By signing in, you agree to our
                        <a href="/legal/terms" class="text-auth">Terms of Service</a> and
                        <a href="/legal/privacy" class="text-auth">Privacy Policy</a>
                    </small>
                </p>
            "#,
            "css": r#"
                <style>
                .demo-badge {
                    position: absolute;
                    top: 10px;
                    right: 10px;
                    background: #28a745;
                    color: white;
                    padding: 5px 10px;
                    border-radius: 15px;
                    font-size: 12px;
                    font-weight: bold;
                }
                </style>
            "#,
            "js": r#"
                <script>
                // Add demo badge
                document.body.innerHTML += '<div class="demo-badge">DEMO</div>';
                </script>
            "#
        });

        TemplateResponse::new("auth/login", &data).with_layout("layouts/auth")
    }

    pub async fn register() -> impl IntoResponse {
        let data = json!({
            "title": "Create Account",
            "page_title": "Join Us Today",
            "subtitle": "Create your account to get started",
            "header_icon": "fas fa-user-plus",
            "organization_registration": true,
            "organizations": [
                {"id": "1", "name": "Acme Corporation"},
                {"id": "2", "name": "Tech Innovations Inc."},
                {"id": "3", "name": "Global Solutions Ltd."}
            ],
            "phone_required": false,
            "newsletter_opt_in": true,
            "captcha_enabled": false,
            "footer_content": r#"
                <div class="text-center">
                    <h6 class="mb-3">Why Join Us?</h6>
                    <div class="row text-start">
                        <div class="col-6">
                            <small>
                                <i class="fas fa-check text-success me-2"></i>Secure & Private<br>
                                <i class="fas fa-check text-success me-2"></i>24/7 Support<br>
                            </small>
                        </div>
                        <div class="col-6">
                            <small>
                                <i class="fas fa-check text-success me-2"></i>Free Forever<br>
                                <i class="fas fa-check text-success me-2"></i>No Spam
                            </small>
                        </div>
                    </div>
                </div>
            "#
        });

        TemplateResponse::new("auth/register", &data).with_layout("layouts/auth")
    }

    pub async fn forgot_password() -> impl IntoResponse {
        let data = json!({
            "title": "Reset Password",
            "page_title": "Reset Your Password",
            "subtitle": "We'll send you a secure reset link",
            "header_icon": "fas fa-key",
            "captcha_enabled": false,
            "rate_limit_info": null,
            "show_success": false
        });

        TemplateResponse::new("auth/forgot-password", &data).with_layout("layouts/auth")
    }

    pub async fn forgot_password_success() -> impl IntoResponse {
        let data = json!({
            "title": "Reset Password",
            "page_title": "Check Your Email",
            "subtitle": "We've sent you a reset link",
            "header_icon": "fas fa-envelope-open",
            "captcha_enabled": false,
            "show_success": true,
            "flash_message": "If an account with that email exists, you'll receive a password reset link within the next few minutes.",
            "flash_type": "success",
            "flash_icon": "fas fa-check-circle"
        });

        TemplateResponse::new("auth/forgot-password", &data).with_layout("layouts/auth")
    }
}