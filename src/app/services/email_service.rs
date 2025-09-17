use anyhow::Result;
use lettre::{Message, AsyncTransport};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::AsyncSmtpTransport;
use crate::config::Config;

pub struct EmailService;

impl EmailService {
    async fn get_mailer() -> Result<AsyncSmtpTransport<lettre::Tokio1Executor>> {
        let config = Config::from_env()?;

        let mut transport = AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(&config.smtp_host)
            .port(config.smtp_port);

        // Add authentication if credentials are provided
        if !config.smtp_username.is_empty() {
            let creds = Credentials::new(config.smtp_username, config.smtp_password);
            transport = transport.credentials(creds);
        }

        // Configure TLS if enabled
        if config.smtp_use_tls {
            let tls_params = TlsParameters::new(config.smtp_host.clone())?;
            transport = transport.tls(Tls::Required(tls_params));
        } else {
            transport = transport.tls(Tls::None);
        }

        Ok(transport.build())
    }

    pub async fn send_password_reset_email(to_email: &str, name: &str, reset_token: &str) -> Result<()> {
        let config = Config::from_env()?;

        let reset_url = format!("{}/reset-password?token={}", config.app_url, reset_token);

        let email_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Password Reset Request</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h2 style="color: #2c3e50;">Password Reset Request</h2>

        <p>Hello <strong>{}</strong>,</p>

        <p>You have requested a password reset for your RustAxum account. Please click the button below to reset your password:</p>

        <div style="text-align: center; margin: 30px 0;">
            <a href="{}" style="background-color: #3498db; color: white; padding: 12px 24px; text-decoration: none; border-radius: 5px; display: inline-block;">Reset Password</a>
        </div>

        <p>If the button doesn't work, you can copy and paste this link into your browser:</p>
        <p style="word-break: break-all; background-color: #f8f9fa; padding: 10px; border-left: 3px solid #3498db;">{}</p>

        <p>This link will expire in 24 hours for security reasons.</p>

        <p>If you did not request this password reset, please ignore this email and your password will remain unchanged.</p>

        <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">

        <p style="font-size: 12px; color: #666;">
            Best regards,<br>
            The RustAxum Team
        </p>
    </div>
</body>
</html>"#,
            name, reset_url, reset_url
        );

        let email = Message::builder()
            .from(format!("{} <{}>", config.smtp_from_name, config.smtp_from_email).parse()?)
            .to(format!("{} <{}>", name, to_email).parse()?)
            .subject("Password Reset Request - RustAxum")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(email_body)?;

        let mailer = Self::get_mailer().await?;

        match mailer.send(email).await {
            Ok(_) => {
                tracing::info!("Password reset email sent successfully to: {}", to_email);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to send password reset email to {}: {}", to_email, e);
                Err(anyhow::anyhow!("Failed to send email: {}", e))
            }
        }
    }

    pub async fn send_welcome_email(to_email: &str, name: &str) -> Result<()> {
        let config = Config::from_env()?;

        let email_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Welcome to RustAxum</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h2 style="color: #2c3e50;">Welcome to RustAxum!</h2>

        <p>Hello <strong>{}</strong>,</p>

        <p>Welcome to RustAxum! Your account has been successfully created and you're now ready to explore our platform.</p>

        <div style="background-color: #f8f9fa; padding: 20px; border-radius: 5px; margin: 20px 0;">
            <h3 style="margin-top: 0; color: #2c3e50;">Getting Started</h3>
            <ul>
                <li>Complete your profile setup</li>
                <li>Explore the dashboard</li>
                <li>Check out our documentation</li>
            </ul>
        </div>

        <p>If you have any questions or need assistance, please don't hesitate to contact our support team.</p>

        <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">

        <p style="font-size: 12px; color: #666;">
            Thank you for joining us!<br>
            The RustAxum Team
        </p>
    </div>
</body>
</html>"#,
            name
        );

        let email = Message::builder()
            .from(format!("{} <{}>", config.smtp_from_name, config.smtp_from_email).parse()?)
            .to(format!("{} <{}>", name, to_email).parse()?)
            .subject("Welcome to RustAxum!")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(email_body)?;

        let mailer = Self::get_mailer().await?;

        match mailer.send(email).await {
            Ok(_) => {
                tracing::info!("Welcome email sent successfully to: {}", to_email);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to send welcome email to {}: {}", to_email, e);
                Err(anyhow::anyhow!("Failed to send email: {}", e))
            }
        }
    }

    pub async fn send_password_changed_notification(to_email: &str, name: &str) -> Result<()> {
        let config = Config::from_env()?;

        let email_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Password Changed Successfully</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h2 style="color: #27ae60;">Password Changed Successfully</h2>

        <p>Hello <strong>{}</strong>,</p>

        <p>This is a confirmation that your password has been successfully changed for your RustAxum account.</p>

        <div style="background-color: #d4edda; color: #155724; padding: 15px; border-radius: 5px; margin: 20px 0; border-left: 4px solid #27ae60;">
            <strong>Security Notice:</strong> If you did not make this change, please contact our support team immediately.
        </div>

        <p>For your security:</p>
        <ul>
            <li>This change was made on: <strong>{}</strong></li>
            <li>All existing sessions have been logged out</li>
            <li>You may need to log in again on your devices</li>
        </ul>

        <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">

        <p style="font-size: 12px; color: #666;">
            Stay secure,<br>
            The RustAxum Team
        </p>
    </div>
</body>
</html>"#,
            name,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        let email = Message::builder()
            .from(format!("{} <{}>", config.smtp_from_name, config.smtp_from_email).parse()?)
            .to(format!("{} <{}>", name, to_email).parse()?)
            .subject("Password Changed - RustAxum")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(email_body)?;

        let mailer = Self::get_mailer().await?;

        match mailer.send(email).await {
            Ok(_) => {
                tracing::info!("Password changed notification sent successfully to: {}", to_email);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to send password changed notification to {}: {}", to_email, e);
                Err(anyhow::anyhow!("Failed to send email: {}", e))
            }
        }
    }
}