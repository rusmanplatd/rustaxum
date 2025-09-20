use anyhow::Result;
use async_trait::async_trait;
use lettre::{Message, AsyncTransport, AsyncSmtpTransport, Tokio1Executor};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::message::{header::ContentType, MultiPart, SinglePart};
use crate::app::mail::{MailDriver, MailMessage, MailContent, Attachment, AttachmentData};

#[derive(Debug, Clone)]
pub struct SmtpDriver {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub encryption: SmtpEncryption,
    pub from_name: String,
    pub from_address: String,
}

#[derive(Debug, Clone)]
pub enum SmtpEncryption {
    None,
    Tls,
    StartTls,
}

impl SmtpDriver {
    pub fn new(host: String, port: u16, from_name: String, from_address: String) -> Self {
        Self {
            host,
            port,
            username: None,
            password: None,
            encryption: SmtpEncryption::StartTls,
            from_name,
            from_address,
        }
    }

    pub fn with_credentials(mut self, username: String, password: String) -> Self {
        self.username = Some(username);
        self.password = Some(password);
        self
    }

    pub fn with_encryption(mut self, encryption: SmtpEncryption) -> Self {
        self.encryption = encryption;
        self
    }

    async fn build_transport(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
        let mut transport_builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.host)
            .port(self.port);

        // Add authentication if credentials are provided
        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            let creds = Credentials::new(username.clone(), password.clone());
            transport_builder = transport_builder.credentials(creds);
        }

        // Configure TLS
        transport_builder = match self.encryption {
            SmtpEncryption::None => transport_builder.tls(Tls::None),
            SmtpEncryption::Tls => {
                let tls_params = TlsParameters::new(self.host.clone())?;
                transport_builder.tls(Tls::Required(tls_params))
            },
            SmtpEncryption::StartTls => {
                let tls_params = TlsParameters::new(self.host.clone())?;
                transport_builder.tls(Tls::Opportunistic(tls_params))
            },
        };

        Ok(transport_builder.build())
    }

    async fn build_email(&self, mut mail_message: MailMessage) -> Result<Message> {
        // Compile markdown if needed
        mail_message.content.compile_markdown().await?;

        // Get from address
        let from_address = mail_message.from.unwrap_or_else(|| {
            format!("{} <{}>", self.from_name, self.from_address)
        });

        // Start building the message
        let mut message_builder = Message::builder()
            .from(from_address.parse()?)
            .subject(&mail_message.subject);

        // Add recipients
        for to in &mail_message.to {
            message_builder = message_builder.to(to.parse()?);
        }

        for cc in &mail_message.cc {
            message_builder = message_builder.cc(cc.parse()?);
        }

        for bcc in &mail_message.bcc {
            message_builder = message_builder.bcc(bcc.parse()?);
        }

        // Add reply-to if provided
        if let Some(reply_to) = mail_message.reply_to {
            message_builder = message_builder.reply_to(reply_to.parse()?);
        }

        // Add custom headers
        for (key, value) in mail_message.headers {
            message_builder = message_builder.header((key.as_str(), value.as_str()));
        }

        // Build content
        let html_content = mail_message.content.to_html().await?;
        let text_content = mail_message.content.to_text();

        // Create multipart message with text and HTML
        let mut multipart = MultiPart::alternative()
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_PLAIN)
                    .body(text_content)
            )
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(html_content)
            );

        // Add attachments
        for attachment in mail_message.attachments {
            multipart = self.add_attachment(multipart, attachment).await?;
        }

        let message = message_builder.multipart(multipart)?;

        Ok(message)
    }

    async fn add_attachment(&self, mut multipart: MultiPart, attachment: Attachment) -> Result<MultiPart> {
        let content_type: ContentType = attachment.content_type.parse()
            .unwrap_or(ContentType::parse("application/octet-stream").unwrap());

        let body = match attachment.data {
            AttachmentData::Path(path) => {
                tokio::fs::read(&path).await
                    .map_err(|e| anyhow::anyhow!("Failed to read attachment file {}: {}", path, e))?
            },
            AttachmentData::Bytes(bytes) => bytes,
            AttachmentData::Base64(base64_str) => {
                base64::decode(&base64_str)
                    .map_err(|e| anyhow::anyhow!("Failed to decode base64 attachment: {}", e))?
            },
        };

        let attachment_part = lettre::message::Attachment::new(attachment.filename)
            .body(body, content_type);

        multipart = multipart.singlepart(attachment_part);

        Ok(multipart)
    }
}

#[async_trait]
impl MailDriver for SmtpDriver {
    async fn send(&self, mail_message: MailMessage) -> Result<()> {
        tracing::info!("SMTP Driver: Sending email via {}:{}", self.host, self.port);

        // Build the transport
        let mailer = self.build_transport().await?;

        // Build the email message
        let email = self.build_email(mail_message).await?;

        // Send the email
        match mailer.send(email).await {
            Ok(_) => {
                tracing::info!("Email sent successfully via SMTP");
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to send email via SMTP: {}", e);
                Err(anyhow::anyhow!("SMTP send failed: {}", e))
            }
        }
    }

    fn driver_name(&self) -> &'static str {
        "smtp"
    }
}