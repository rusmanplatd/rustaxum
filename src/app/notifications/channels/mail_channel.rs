use anyhow::Result;
use async_trait::async_trait;
use lettre::{Message, AsyncTransport};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::AsyncSmtpTransport;
use crate::app::notifications::channels::Channel;
use crate::app::notifications::notification::{Notification, Notifiable, NotificationChannel, MailContent};
use crate::config::Config;

pub struct MailChannel;

impl MailChannel {
    pub fn new() -> Self {
        Self
    }

    async fn get_mailer() -> Result<AsyncSmtpTransport<lettre::Tokio1Executor>> {
        let config = Config::load()?;

        let mut transport = AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(&config.mail.host)
            .port(config.mail.port);

        // Add authentication if credentials are provided
        if !config.mail.username.is_empty() {
            let creds = Credentials::new(config.mail.username.clone(), config.mail.password.clone());
            transport = transport.credentials(creds);
        }

        // Configure TLS if enabled
        if config.mail.use_tls() {
            let tls_params = TlsParameters::new(config.mail.host.clone())?;
            transport = transport.tls(Tls::Required(tls_params));
        } else {
            transport = transport.tls(Tls::None);
        }

        Ok(transport.build())
    }

    fn convert_content_to_html(content: &MailContent) -> String {
        match content {
            MailContent::Html(html) => html.clone(),
            MailContent::Text(text) => {
                // Convert plain text to HTML
                format!("<pre style=\"font-family: inherit; white-space: pre-wrap;\">{}</pre>", text)
            }
            MailContent::Markdown(md) => {
                // For basic markdown to HTML conversion
                // In a real implementation, you'd use a markdown parser like pulldown-cmark
                md.replace('\n', "<br>")
                    .replace("**", "<strong>")
                    .replace("*", "<em>")
            }
        }
    }
}

#[async_trait]
impl Channel for MailChannel {
    async fn send(&self, notification: &dyn Notification, notifiable: &dyn Notifiable) -> Result<()> {
        // Get email address for the notifiable entity
        let email_address = match notifiable.route_notification_for(&NotificationChannel::Mail).await {
            Some(email) => email,
            None => {
                tracing::warn!("No email address found for notifiable entity: {}", notifiable.get_key());
                return Ok(());
            }
        };

        // Get mail message from notification
        let mail_message = notification.to_mail(notifiable).await?;
        let config = Config::load()?;

        // Convert content to HTML
        let html_content = Self::convert_content_to_html(&mail_message.content);

        // Build email message
        let from_address = mail_message.from
            .unwrap_or_else(|| format!("{} <{}>", config.mail.from_name, config.mail.from_address));

        let email = Message::builder()
            .from(from_address.parse()?)
            .to(mail_message.to.parse()?)
            .subject(&mail_message.subject)
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(html_content)?;

        // Send email
        let mailer = Self::get_mailer().await?;

        match mailer.send(email).await {
            Ok(_) => {
                tracing::info!(
                    "Notification email sent successfully to: {} (type: {})",
                    email_address,
                    notification.notification_type()
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    "Failed to send notification email to {}: {}",
                    email_address,
                    e
                );
                Err(anyhow::anyhow!("Failed to send email: {}", e))
            }
        }
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Mail
    }
}