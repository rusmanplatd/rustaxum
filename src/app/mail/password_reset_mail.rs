use anyhow::Result;
use async_trait::async_trait;
use crate::app::mail::{Mailable, MailMessage, MailContent};

#[derive(Debug, Clone)]
pub struct PasswordResetMail {
    pub to_email: String,
    pub user_name: String,
    pub reset_token: String,
    pub reset_url: Option<String>,
    pub expires_at: Option<String>,
}

impl PasswordResetMail {
    pub fn new(to_email: String, user_name: String, reset_token: String) -> Self {
        Self {
            to_email,
            user_name,
            reset_token,
            reset_url: None,
            expires_at: None,
        }
    }

    pub fn with_reset_url(mut self, url: String) -> Self {
        self.reset_url = Some(url);
        self
    }

    pub fn with_expiration(mut self, expires_at: String) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}

#[async_trait]
impl Mailable for PasswordResetMail {
    async fn build(&self) -> Result<MailMessage> {
        let reset_link = self.reset_url.clone().unwrap_or_else(|| {
            format!("https://app.example.com/reset-password?token={}", self.reset_token)
        });

        let expiration_text = if let Some(ref expires) = self.expires_at {
            format!("This link will expire on {}.", expires)
        } else {
            "This link will expire in 24 hours.".to_string()
        };

        let markdown_content = format!(
            r#"# Password Reset Request

Hello **{}**,

We received a request to reset your password for your account.

## Reset Your Password

Click the link below to reset your password:

[Reset Password]({})

{}

## Security Notice

If you didn't request this password reset, please ignore this email. Your password will remain unchanged.

For security reasons, we recommend:
- Using a strong, unique password
- Enabling two-factor authentication if available
- Not sharing your password with anyone

If you have any concerns about your account security, please contact our support team immediately.

Best regards,
The Security Team
"#,
            self.user_name, reset_link, expiration_text
        );

        Ok(MailMessage::new()
            .to(self.to_email.clone())
            .subject("Password Reset Request".to_string())
            .content(MailContent::Markdown {
                markdown: markdown_content,
                compiled_html: None,
            }))
    }

    fn to(&self) -> Vec<String> {
        vec![self.to_email.clone()]
    }

    fn subject(&self) -> String {
        "Password Reset Request".to_string()
    }

    fn should_queue(&self) -> bool {
        false // Password reset emails should be sent immediately for security
    }

    fn queue_name(&self) -> Option<&str> {
        Some("security") // Use dedicated security queue for password resets
    }
}