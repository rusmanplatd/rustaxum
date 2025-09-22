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

    /// Write to rotating log file with size-based rotation
    async fn write_to_rotating_log(&self, file_path: &str, content: &str) -> Result<()> {
        use std::fs::{OpenOptions, metadata};
        use std::io::{Write, BufWriter};
        use std::path::Path;

        // Configuration for log rotation
        const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB
        const MAX_BACKUP_FILES: usize = 5;

        let path = Path::new(file_path);

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Check if file needs rotation
        if path.exists() {
            let file_size = metadata(path)?.len();
            if file_size >= MAX_FILE_SIZE {
                self.rotate_log_files(file_path, MAX_BACKUP_FILES)?;
            }
        }

        // Write to the main log file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        let mut writer = BufWriter::new(file);

        // Add timestamp to log entry
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        writeln!(writer, "[{}]", timestamp)?;
        writeln!(writer, "{}", content)?;
        let _ = writeln!(writer, ""); // Extra newline for readability

        writer.flush()?;

        Ok(())
    }

    /// Rotate log files by renaming them with incremental numbers
    fn rotate_log_files(&self, file_path: &str, max_backups: usize) -> Result<()> {
        use std::fs;
        use std::path::Path;

        let path = Path::new(file_path);
        let file_stem = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?;
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let parent = path.parent().unwrap_or_else(|| Path::new("."));

        // Remove the oldest backup if we're at the limit
        let oldest_backup = if extension.is_empty() {
            parent.join(format!("{}.{}", file_stem, max_backups))
        } else {
            parent.join(format!("{}.{}.{}", file_stem, max_backups, extension))
        };

        if oldest_backup.exists() {
            fs::remove_file(&oldest_backup)?;
        }

        // Shift existing backups
        for i in (1..max_backups).rev() {
            let current_backup = if extension.is_empty() {
                parent.join(format!("{}.{}", file_stem, i))
            } else {
                parent.join(format!("{}.{}.{}", file_stem, i, extension))
            };

            let next_backup = if extension.is_empty() {
                parent.join(format!("{}.{}", file_stem, i + 1))
            } else {
                parent.join(format!("{}.{}.{}", file_stem, i + 1, extension))
            };

            if current_backup.exists() {
                fs::rename(&current_backup, &next_backup)?;
            }
        }

        // Move current log to .1 backup
        let first_backup = if extension.is_empty() {
            parent.join(format!("{}.1", file_stem))
        } else {
            parent.join(format!("{}.1.{}", file_stem, extension))
        };

        if path.exists() {
            fs::rename(path, &first_backup)?;
        }

        tracing::info!("Rotated log file: {} -> {}", file_path, first_backup.display());

        Ok(())
    }

    /// Cleanup old log files based on age
    pub async fn cleanup_old_logs(&self, max_age_days: u32) -> Result<()> {
        if let Some(ref file_path) = self.file_path {
            use std::fs;
            use std::time::{SystemTime, UNIX_EPOCH, Duration};
            use std::path::Path;

            let path = Path::new(file_path);
            let parent = path.parent().unwrap_or_else(|| Path::new("."));
            let file_stem = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("mail");

            let cutoff_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .saturating_sub(Duration::from_secs(max_age_days as u64 * 24 * 3600));

            // Find and remove old backup files
            if let Ok(entries) = fs::read_dir(parent) {
                for entry in entries.flatten() {
                    if let Ok(filename) = entry.file_name().into_string() {
                        if filename.starts_with(file_stem) && filename.contains('.') {
                            if let Ok(metadata) = entry.metadata() {
                                if let Ok(modified) = metadata.modified() {
                                    if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                                        if duration < cutoff_time {
                                            if let Err(e) = fs::remove_file(entry.path()) {
                                                tracing::warn!("Failed to remove old log file {:?}: {}", entry.path(), e);
                                            } else {
                                                tracing::info!("Removed old log file: {:?}", entry.path());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
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
                if let Err(e) = self.write_to_rotating_log(file_path, &log_content).await {
                    tracing::error!("Failed to write to log file: {}", e);
                    println!("Failed to write to log file: {}", e);
                }
            }
        }

        println!("{}", log_content);

        Ok(())
    }

    fn driver_name(&self) -> &'static str {
        "log"
    }
}