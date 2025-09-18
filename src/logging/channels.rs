use anyhow::Result;
use std::collections::HashMap;
use serde_json::Value;
use crate::config::logging::ChannelConfig;
use crate::logging::writers::{FileWriter, StderrWriter, DailyFileWriter};
use crate::logging::formatters::{Formatter, DefaultFormatter, JsonFormatter};

pub trait Channel {
    fn log(&mut self, level: &str, message: &str, context: Option<HashMap<String, Value>>) -> Result<()>;
}

pub struct SingleChannel {
    writer: Box<dyn Writer>,
    formatter: Box<dyn Formatter>,
    level: String,
}

pub struct DailyChannel {
    writer: Box<dyn Writer>,
    formatter: Box<dyn Formatter>,
    level: String,
}

pub struct StderrChannel {
    writer: Box<dyn Writer>,
    formatter: Box<dyn Formatter>,
    level: String,
}

pub struct StackChannel {
    channels: Vec<Box<dyn Channel + Send + Sync>>,
    level: String,
}

pub trait Writer: Send + Sync {
    fn write(&mut self, formatted_message: &str) -> Result<()>;
}

impl Channel for SingleChannel {
    fn log(&mut self, level: &str, message: &str, context: Option<HashMap<String, Value>>) -> Result<()> {
        if should_log(&self.level, level) {
            let formatted = self.formatter.format(level, message, context)?;
            self.writer.write(&formatted)?;
        }
        Ok(())
    }
}

impl Channel for DailyChannel {
    fn log(&mut self, level: &str, message: &str, context: Option<HashMap<String, Value>>) -> Result<()> {
        if should_log(&self.level, level) {
            let formatted = self.formatter.format(level, message, context)?;
            self.writer.write(&formatted)?;
        }
        Ok(())
    }
}

impl Channel for StderrChannel {
    fn log(&mut self, level: &str, message: &str, context: Option<HashMap<String, Value>>) -> Result<()> {
        if should_log(&self.level, level) {
            let formatted = self.formatter.format(level, message, context)?;
            self.writer.write(&formatted)?;
        }
        Ok(())
    }
}

impl Channel for StackChannel {
    fn log(&mut self, level: &str, message: &str, context: Option<HashMap<String, Value>>) -> Result<()> {
        if should_log(&self.level, level) {
            for channel in &mut self.channels {
                channel.log(level, message, context.clone())?;
            }
        }
        Ok(())
    }
}

pub struct ChannelManager;

impl ChannelManager {
    pub fn new() -> Self {
        Self
    }

    pub fn create_channel(&self, config: &ChannelConfig) -> Result<Box<dyn Channel + Send + Sync>> {
        let formatter: Box<dyn Formatter> = match config.format.as_deref().unwrap_or("default") {
            "json" => Box::new(JsonFormatter::new(
                config.date_format.as_deref().unwrap_or("%Y-%m-%d %H:%M:%S")
            )),
            _ => Box::new(DefaultFormatter::new(
                config.date_format.as_deref().unwrap_or("%Y-%m-%d %H:%M:%S")
            )),
        };

        match config.driver.as_str() {
            "single" => {
                let writer: Box<dyn Writer> = Box::new(FileWriter::new(
                    config.path.as_deref().unwrap_or("storage/logs/app.log")
                )?);
                Ok(Box::new(SingleChannel {
                    writer,
                    formatter,
                    level: config.level.clone(),
                }))
            },
            "daily" => {
                let writer: Box<dyn Writer> = Box::new(DailyFileWriter::new(
                    config.path.as_deref().unwrap_or("storage/logs/app"),
                    config.max_files.unwrap_or(7)
                )?);
                Ok(Box::new(DailyChannel {
                    writer,
                    formatter,
                    level: config.level.clone(),
                }))
            },
            "stderr" => {
                let writer: Box<dyn Writer> = Box::new(StderrWriter::new());
                Ok(Box::new(StderrChannel {
                    writer,
                    formatter,
                    level: config.level.clone(),
                }))
            },
            "stack" => {
                // For stack channel, we'll create stderr + daily by default
                let stderr_channel = Box::new(StderrChannel {
                    writer: Box::new(StderrWriter::new()),
                    formatter: Box::new(DefaultFormatter::new(
                        config.date_format.as_deref().unwrap_or("%Y-%m-%d %H:%M:%S")
                    )),
                    level: config.level.clone(),
                });

                let daily_channel = Box::new(DailyChannel {
                    writer: Box::new(DailyFileWriter::new(
                        "storage/logs/app",
                        7
                    )?),
                    formatter: Box::new(DefaultFormatter::new(
                        config.date_format.as_deref().unwrap_or("%Y-%m-%d %H:%M:%S")
                    )),
                    level: config.level.clone(),
                });

                Ok(Box::new(StackChannel {
                    channels: vec![stderr_channel, daily_channel],
                    level: config.level.clone(),
                }))
            },
            _ => Err(anyhow::anyhow!("Unknown channel driver: {}", config.driver))
        }
    }
}

fn should_log(channel_level: &str, message_level: &str) -> bool {
    let channel_priority = log_level_priority(channel_level);
    let message_priority = log_level_priority(message_level);
    message_priority <= channel_priority
}

fn log_level_priority(level: &str) -> u8 {
    match level.to_lowercase().as_str() {
        "emergency" => 0,
        "alert" => 1,
        "critical" => 2,
        "error" => 3,
        "warning" => 4,
        "notice" => 5,
        "info" => 6,
        "debug" => 7,
        _ => 6, // default to info
    }
}