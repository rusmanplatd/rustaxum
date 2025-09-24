pub mod password_reset_mail;
pub mod welcome_mail;
pub mod drivers;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Base trait that all mail implementations must implement
#[async_trait]
pub trait Mailable: Send + Sync {
    /// Build the mail message
    async fn build(&self) -> Result<MailMessage>;

    /// Get the recipients for this mail
    fn to(&self) -> Vec<String>;

    /// Get the sender (optional, falls back to default)
    fn from(&self) -> Option<String> {
        None
    }

    /// Get the reply-to address (optional)
    fn reply_to(&self) -> Option<String> {
        None
    }

    /// Get the subject line
    fn subject(&self) -> String;

    /// Get additional headers
    fn headers(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Get the priority level (1-5, where 1 is highest)
    fn priority(&self) -> Option<u8> {
        None
    }

    /// Get attachments
    fn attachments(&self) -> Vec<Attachment> {
        Vec::new()
    }

    /// Determine if this mail should be queued
    fn should_queue(&self) -> bool {
        false
    }

    /// Get the queue name for this mail (if queued)
    fn queue_name(&self) -> Option<&str> {
        None
    }
}

/// Mail message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailMessage {
    pub from: Option<String>,
    pub reply_to: Option<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub content: MailContent,
    pub headers: HashMap<String, String>,
    pub priority: Option<u8>,
    pub attachments: Vec<Attachment>,
}

/// Mail content types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MailContent {
    Text(String),
    Html(String),
    Markdown {
        markdown: String,
        compiled_html: Option<String>,
    },
    Template {
        name: String,
        data: serde_json::Value,
    },
    Multipart {
        text: String,
        html: String,
    },
}

/// Mail attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub content_type: String,
    pub data: AttachmentData,
}

/// Attachment data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttachmentData {
    Path(String),
    Bytes(Vec<u8>),
    Base64(String),
}

/// Mail driver trait for different email providers
#[async_trait]
pub trait MailDriver: Send + Sync {
    async fn send(&self, message: MailMessage) -> Result<()>;
    fn driver_name(&self) -> &'static str;
}

/// Mail manager that handles different drivers
pub struct MailManager {
    drivers: HashMap<String, Box<dyn MailDriver>>,
    default_driver: String,
}

impl MailManager {
    pub fn new(default_driver: String) -> Self {
        Self {
            drivers: HashMap::new(),
            default_driver,
        }
    }

    pub fn register_driver(&mut self, name: String, driver: Box<dyn MailDriver>) {
        self.drivers.insert(name, driver);
    }

    pub async fn send(&self, mailable: &dyn Mailable) -> Result<()> {
        let message = mailable.build().await?;
        self.send_message(message).await
    }

    pub async fn send_message(&self, message: MailMessage) -> Result<()> {
        let driver = self.drivers.get(&self.default_driver)
            .ok_or_else(|| anyhow::anyhow!("Mail driver '{}' not found", self.default_driver))?;

        driver.send(message).await
    }

    pub async fn send_with_driver(&self, mailable: &dyn Mailable, driver_name: &str) -> Result<()> {
        let message = mailable.build().await?;
        let driver = self.drivers.get(driver_name)
            .ok_or_else(|| anyhow::anyhow!("Mail driver '{}' not found", driver_name))?;

        driver.send(message).await
    }
}

impl MailMessage {
    pub fn new() -> Self {
        Self {
            from: None,
            reply_to: None,
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: String::new(),
            content: MailContent::Text(String::new()),
            headers: HashMap::new(),
            priority: None,
            attachments: Vec::new(),
        }
    }

    pub fn to(mut self, email: String) -> Self {
        self.to.push(email);
        self
    }

    pub fn from(mut self, email: String) -> Self {
        self.from = Some(email);
        self
    }

    pub fn reply_to(mut self, email: String) -> Self {
        self.reply_to = Some(email);
        self
    }

    pub fn cc(mut self, email: String) -> Self {
        self.cc.push(email);
        self
    }

    pub fn bcc(mut self, email: String) -> Self {
        self.bcc.push(email);
        self
    }

    pub fn subject(mut self, subject: String) -> Self {
        self.subject = subject;
        self
    }

    pub fn content(mut self, content: MailContent) -> Self {
        self.content = content;
        self
    }

    pub fn header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority.min(5).max(1));
        self
    }

    pub fn attach(mut self, attachment: Attachment) -> Self {
        self.attachments.push(attachment);
        self
    }
}

impl Attachment {
    pub fn from_path(filename: String, path: String, content_type: Option<String>) -> Self {
        Self {
            filename,
            content_type: content_type.unwrap_or_else(|| "application/octet-stream".to_string()),
            data: AttachmentData::Path(path),
        }
    }

    pub fn from_bytes(filename: String, bytes: Vec<u8>, content_type: Option<String>) -> Self {
        Self {
            filename,
            content_type: content_type.unwrap_or_else(|| "application/octet-stream".to_string()),
            data: AttachmentData::Bytes(bytes),
        }
    }

    pub fn from_base64(filename: String, base64: String, content_type: Option<String>) -> Self {
        Self {
            filename,
            content_type: content_type.unwrap_or_else(|| "application/octet-stream".to_string()),
            data: AttachmentData::Base64(base64),
        }
    }
}

impl MailContent {
    /// Compile markdown to HTML if needed
    pub async fn compile_markdown(&mut self) -> Result<()> {
        if let MailContent::Markdown { markdown, compiled_html } = self {
            if compiled_html.is_none() {
                // Use a markdown parser to convert to HTML
                // This is a simple implementation - you might want to use a more sophisticated parser
                let html = markdown_to_html(markdown);
                *compiled_html = Some(html);
            }
        }
        Ok(())
    }

    /// Get HTML content (compiling markdown if necessary)
    pub async fn to_html(&mut self) -> Result<String> {
        match self {
            MailContent::Html(html) => Ok(html.clone()),
            MailContent::Markdown { compiled_html, .. } => {
                if compiled_html.is_none() {
                    self.compile_markdown().await?;
                }
                match self {
                    MailContent::Markdown { compiled_html, .. } => {
                        Ok(compiled_html.as_ref().unwrap().clone())
                    },
                    _ => unreachable!()
                }
            },
            MailContent::Text(text) => {
                // Simple text to HTML conversion
                Ok(format!("<pre style=\"font-family: inherit; white-space: pre-wrap;\">{}</pre>",
                    html_escape::encode_text(text)))
            },
            MailContent::Multipart { html, .. } => Ok(html.clone()),
            MailContent::Template { .. } => {
                // Template rendering would be implemented here
                Err(anyhow::anyhow!("Template rendering not implemented"))
            }
        }
    }

    /// Get text content
    pub fn to_text(&self) -> String {
        match self {
            MailContent::Text(text) => text.clone(),
            MailContent::Html(html) => {
                // (TODO:  use proper HTML parser in production)
                html.clone()
            },
            MailContent::Markdown { markdown, .. } => markdown.clone(),
            MailContent::Multipart { text, .. } => text.clone(),
            MailContent::Template { .. } => "Template content".to_string(),
        }
    }
}

/// Convert markdown to HTML using pulldown-cmark
fn markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{Parser, Options, html};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

/// Global mail manager instance
static MAIL_MANAGER: tokio::sync::OnceCell<std::sync::Arc<tokio::sync::RwLock<MailManager>>> = tokio::sync::OnceCell::const_new();

/// Initialize the global mail manager
pub async fn init_mail_manager(default_driver: String) -> std::sync::Arc<tokio::sync::RwLock<MailManager>> {
    MAIL_MANAGER.get_or_init(|| async {
        std::sync::Arc::new(tokio::sync::RwLock::new(MailManager::new(default_driver)))
    }).await.clone()
}

/// Get the global mail manager
pub async fn mail_manager() -> std::sync::Arc<tokio::sync::RwLock<MailManager>> {
    MAIL_MANAGER.get_or_init(|| async {
        std::sync::Arc::new(tokio::sync::RwLock::new(MailManager::new("smtp".to_string())))
    }).await.clone()
}

/// Send mail using the global manager
pub async fn mail(mailable: &dyn Mailable) -> Result<()> {
    let manager = mail_manager().await;
    let manager = manager.read().await;
    manager.send(mailable).await
}