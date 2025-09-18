use anyhow::Result;
use chrono::{Utc, DateTime};
use std::collections::HashMap;
use serde_json::Value;

pub trait Formatter: Send + Sync {
    fn format(&self, level: &str, message: &str, context: Option<HashMap<String, Value>>) -> Result<String>;
}

pub struct DefaultFormatter {
    date_format: String,
}

pub struct JsonFormatter {
    date_format: String,
}

impl DefaultFormatter {
    pub fn new(date_format: &str) -> Self {
        Self {
            date_format: date_format.to_string(),
        }
    }
}

impl Formatter for DefaultFormatter {
    fn format(&self, level: &str, message: &str, context: Option<HashMap<String, Value>>) -> Result<String> {
        let now: DateTime<Utc> = Utc::now();
        let timestamp = now.format(&self.date_format).to_string();

        let mut formatted = format!("[{}] {}: {}", timestamp, level.to_uppercase(), message);

        if let Some(ctx) = context {
            if !ctx.is_empty() {
                formatted.push_str(&format!(" {:?}", ctx));
            }
        }

        formatted.push('\n');
        Ok(formatted)
    }
}

impl JsonFormatter {
    pub fn new(date_format: &str) -> Self {
        Self {
            date_format: date_format.to_string(),
        }
    }
}

impl Formatter for JsonFormatter {
    fn format(&self, level: &str, message: &str, context: Option<HashMap<String, Value>>) -> Result<String> {
        let now: DateTime<Utc> = Utc::now();
        let timestamp = now.format(&self.date_format).to_string();

        let mut log_entry = serde_json::Map::new();
        log_entry.insert("timestamp".to_string(), Value::String(timestamp));
        log_entry.insert("level".to_string(), Value::String(level.to_uppercase()));
        log_entry.insert("message".to_string(), Value::String(message.to_string()));

        if let Some(ctx) = context {
            log_entry.insert("context".to_string(), Value::Object(
                ctx.into_iter().collect()
            ));
        }

        let json_str = serde_json::to_string(&log_entry)?;
        Ok(format!("{}\n", json_str))
    }
}