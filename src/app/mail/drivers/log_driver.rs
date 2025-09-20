use anyhow::Result;
use async_trait::async_trait;
use crate::app::mail::{MailDriver, MailMessage};

#[derive(Debug, Clone)]
pub struct LogDriver {
    pub log_to_file: bool,
    pub file_path: Option<String>,
}

impl LogDriver {
    pub fn new() -> Self {
        Self {
            log_to_file: false,
            file_path: None,
        }
    }

    pub fn with_file(mut self, file_path: String) -> Self {
        self.log_to_file = true;
        self.file_path = Some(file_path);
        self
    }
}

impl Default for LogDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MailDriver for LogDriver {
    async fn send(&self, mut message: MailMessage) -> Result<()> {
        // Compile markdown if needed
        message.content.compile_markdown().await?;

        let log_content = format!(
            r#"===============================
MAIL LOG ENTRY
===============================
From: {:?}
To: {:?}
CC: {:?}
BCC: {:?}
Subject: {}
Priority: {:?}
Headers: {:?}

Content (Text):
{}

Content (HTML):
{}

Attachments: {}
===============================
"#,
            message.from,
            message.to,
            message.cc,
            message.bcc,
            message.subject,
            message.priority,
            message.headers,
            message.content.to_text(),
            message.content.to_html().await.unwrap_or_else(|_| "Error rendering HTML".to_string()),
            message.attachments.len()
        );

        if self.log_to_file {
            if let Some(ref file_path) = self.file_path {
                // In a real implementation, append to file
                println!("Would log to file: {}", file_path);
            }
        }

        println!("{}", log_content);

        Ok(())
    }

    fn driver_name(&self) -> &'static str {
        "log"
    }
}