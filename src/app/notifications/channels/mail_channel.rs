use anyhow::Result;
use async_trait::async_trait;
use crate::app::notifications::channels::Channel;
use crate::app::notifications::notification::{Notification, Notifiable, NotificationChannel};
use crate::app::mail::{mail_manager, MailMessage, MailContent as NewMailContent};

pub struct MailChannel;

impl MailChannel {
    pub fn new() -> Self {
        Self
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
        let old_mail_message = notification.to_mail(notifiable).await?;

        // Convert to new mail system format
        let new_content = match old_mail_message.content {
            crate::app::notifications::notification::MailContent::Text(text) => {
                NewMailContent::Text(text)
            },
            crate::app::notifications::notification::MailContent::Html(html) => {
                NewMailContent::Html(html)
            },
            crate::app::notifications::notification::MailContent::Markdown(md) => {
                NewMailContent::Markdown {
                    markdown: md,
                    compiled_html: None,
                }
            },
        };

        let mut new_message = MailMessage::new()
            .to(old_mail_message.to)
            .subject(old_mail_message.subject)
            .content(new_content);

        if let Some(from) = old_mail_message.from {
            new_message = new_message.from(from);
        }

        // Add attachments
        for attachment_path in old_mail_message.attachments {
            new_message = new_message.attach(
                crate::app::mail::Attachment::from_path(
                    attachment_path.clone(),
                    attachment_path,
                    None
                )
            );
        }

        // Send using the mail manager
        let manager = mail_manager().await;
        let manager = manager.read().await;

        match manager.send_message(new_message).await {
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
                Err(e)
            }
        }
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Mail
    }
}