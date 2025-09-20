use anyhow::Result;
use async_trait::async_trait;
use crate::app::mail::{Mailable, MailMessage, MailContent};

#[derive(Debug, Clone)]
pub struct WelcomeMail {
    pub to_email: String,
    pub user_name: String,
    pub activation_link: Option<String>,
}

impl WelcomeMail {
    pub fn new(to_email: String, user_name: String) -> Self {
        Self {
            to_email,
            user_name,
            activation_link: None,
        }
    }

    pub fn with_activation_link(mut self, link: String) -> Self {
        self.activation_link = Some(link);
        self
    }
}

#[async_trait]
impl Mailable for WelcomeMail {
    async fn build(&self) -> Result<MailMessage> {
        let markdown_content = if let Some(ref link) = self.activation_link {
            format!(
                r#"# Welcome to Our Platform!

Hello **{}**,

Welcome to our platform! We're excited to have you on board.

## Getting Started

To activate your account, please click the link below:

[Activate Your Account]({})

If you have any questions, feel free to reach out to our support team.

Best regards,
The Team
"#,
                self.user_name, link
            )
        } else {
            format!(
                r#"# Welcome to Our Platform!

Hello **{}**,

Welcome to our platform! We're excited to have you on board.

Your account is now active and ready to use.

If you have any questions, feel free to reach out to our support team.

Best regards,
The Team
"#,
                self.user_name
            )
        };

        Ok(MailMessage::new()
            .to(self.to_email.clone())
            .subject(format!("Welcome, {}!", self.user_name))
            .content(MailContent::Markdown {
                markdown: markdown_content,
                compiled_html: None,
            }))
    }

    fn to(&self) -> Vec<String> {
        vec![self.to_email.clone()]
    }

    fn subject(&self) -> String {
        format!("Welcome, {}!", self.user_name)
    }

    fn should_queue(&self) -> bool {
        true
    }

    fn queue_name(&self) -> Option<&str> {
        Some("emails")
    }
}