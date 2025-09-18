use anyhow::Result;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{Write as IoWrite, stderr};
use std::path::PathBuf;
use chrono::{Utc, DateTime};
use crate::logging::channels::Writer;

pub struct FileWriter {
    file_path: PathBuf,
}

pub struct DailyFileWriter {
    base_path: PathBuf,
    max_files: u32,
    current_date: String,
    current_file: Option<File>,
}

pub struct StderrWriter;

impl FileWriter {
    pub fn new(path: &str) -> Result<Self> {
        let file_path = PathBuf::from(path);

        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            create_dir_all(parent)?;
        }

        Ok(Self { file_path })
    }
}

impl Writer for FileWriter {
    fn write(&mut self, formatted_message: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;

        file.write_all(formatted_message.as_bytes())?;
        file.flush()?;
        Ok(())
    }
}

impl DailyFileWriter {
    pub fn new(base_path: &str, max_files: u32) -> Result<Self> {
        let base_path = PathBuf::from(base_path);

        // Create parent directories if they don't exist
        if let Some(parent) = base_path.parent() {
            create_dir_all(parent)?;
        }

        Ok(Self {
            base_path,
            max_files,
            current_date: String::new(),
            current_file: None,
        })
    }

    fn get_current_file_path(&self) -> PathBuf {
        let now: DateTime<Utc> = Utc::now();
        let date_str = now.format("%Y-%m-%d").to_string();

        let filename = format!("{}-{}.log",
            self.base_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("app"),
            date_str
        );

        if let Some(parent) = self.base_path.parent() {
            parent.join(filename)
        } else {
            PathBuf::from(filename)
        }
    }

    fn cleanup_old_files(&self) -> Result<()> {
        if let Some(parent_dir) = self.base_path.parent() {
            let mut log_files = Vec::new();

            if parent_dir.exists() {
                for entry in std::fs::read_dir(parent_dir)? {
                    let entry = entry?;
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();

                    let base_name = self.base_path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("app");

                    if file_name_str.starts_with(base_name) && file_name_str.ends_with(".log") {
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                log_files.push((entry.path(), modified));
                            }
                        }
                    }
                }

                // Sort by modification time (newest first)
                log_files.sort_by(|a, b| b.1.cmp(&a.1));

                // Remove files exceeding max_files limit
                if log_files.len() > self.max_files as usize {
                    for (file_path, _) in log_files.iter().skip(self.max_files as usize) {
                        std::fs::remove_file(file_path).ok();
                    }
                }
            }
        }

        Ok(())
    }
}

impl Writer for DailyFileWriter {
    fn write(&mut self, formatted_message: &str) -> Result<()> {
        let now: DateTime<Utc> = Utc::now();
        let current_date = now.format("%Y-%m-%d").to_string();

        // Check if we need to rotate to a new file
        if self.current_date != current_date || self.current_file.is_none() {
            self.current_date = current_date;
            let file_path = self.get_current_file_path();

            // Create parent directories if they don't exist
            if let Some(parent) = file_path.parent() {
                create_dir_all(parent)?;
            }

            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)?;

            self.current_file = Some(file);

            // Cleanup old files
            self.cleanup_old_files().ok(); // Don't fail if cleanup fails
        }

        if let Some(ref mut file) = self.current_file {
            file.write_all(formatted_message.as_bytes())?;
            file.flush()?;
        }

        Ok(())
    }
}

impl StderrWriter {
    pub fn new() -> Self {
        Self
    }
}

impl Writer for StderrWriter {
    fn write(&mut self, formatted_message: &str) -> Result<()> {
        let mut stderr = stderr();
        stderr.write_all(formatted_message.as_bytes())?;
        stderr.flush()?;
        Ok(())
    }
}

// Safe because FileWriter only contains a PathBuf which is Send + Sync
unsafe impl Send for FileWriter {}
unsafe impl Sync for FileWriter {}

// Safe because DailyFileWriter contains only Send + Sync types
unsafe impl Send for DailyFileWriter {}
unsafe impl Sync for DailyFileWriter {}

// Safe because StderrWriter has no fields
unsafe impl Send for StderrWriter {}
unsafe impl Sync for StderrWriter {}