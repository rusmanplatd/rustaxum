use std::sync::{OnceLock, Mutex};
use std::collections::HashMap;
use anyhow::Result;
use serde_json::Value;
use crate::config::logging::LoggingConfig;
use crate::logging::channels::{Channel, ChannelManager};

static LOG_MANAGER: OnceLock<Mutex<LogManager>> = OnceLock::new();

pub struct LogManager {
    config: LoggingConfig,
    channels: HashMap<String, Box<dyn Channel + Send + Sync>>,
    channel_manager: ChannelManager,
}

impl LogManager {
    pub fn new(config: LoggingConfig) -> Result<Self> {
        let channel_manager = ChannelManager::new();
        let channels = HashMap::new();

        Ok(LogManager {
            config,
            channels,
            channel_manager,
        })
    }

    pub fn channel(&mut self, name: &str) -> Result<&mut Box<dyn Channel + Send + Sync>> {
        if !self.channels.contains_key(name) {
            if let Some(channel_config) = self.config.get_channel(name) {
                let channel = self.channel_manager.create_channel(channel_config)?;
                self.channels.insert(name.to_string(), channel);
            } else {
                return Err(anyhow::anyhow!("Channel '{}' not found in configuration", name));
            }
        }

        Ok(self.channels.get_mut(name).unwrap())
    }

    pub fn default_channel(&mut self) -> Result<&mut Box<dyn Channel + Send + Sync>> {
        let default_name = self.config.default.clone();
        self.channel(&default_name)
    }

    pub fn log(&mut self, level: &str, message: &str, context: Option<HashMap<String, Value>>) -> Result<()> {
        if let Ok(channel) = self.default_channel() {
            channel.log(level, message, context)?;
        }
        Ok(())
    }

    pub fn log_to_channel(&mut self, channel_name: &str, level: &str, message: &str, context: Option<HashMap<String, Value>>) -> Result<()> {
        if let Ok(channel) = self.channel(channel_name) {
            channel.log(level, message, context)?;
        }
        Ok(())
    }
}

pub struct Log;

impl Log {
    pub fn init(config: LoggingConfig) -> Result<()> {
        let manager = LogManager::new(config)?;
        LOG_MANAGER.set(Mutex::new(manager)).map_err(|_| anyhow::anyhow!("Log manager already initialized"))?;
        Ok(())
    }

    pub fn emergency(message: &str) {
        Self::log("emergency", message, None);
    }

    pub fn emergency_with_context(message: &str, context: HashMap<String, Value>) {
        Self::log("emergency", message, Some(context));
    }

    pub fn alert(message: &str) {
        Self::log("alert", message, None);
    }

    pub fn alert_with_context(message: &str, context: HashMap<String, Value>) {
        Self::log("alert", message, Some(context));
    }

    pub fn critical(message: &str) {
        Self::log("critical", message, None);
    }

    pub fn critical_with_context(message: &str, context: HashMap<String, Value>) {
        Self::log("critical", message, Some(context));
    }

    pub fn error(message: &str) {
        Self::log("error", message, None);
    }

    pub fn error_with_context(message: &str, context: HashMap<String, Value>) {
        Self::log("error", message, Some(context));
    }

    pub fn warning(message: &str) {
        Self::log("warning", message, None);
    }

    pub fn warning_with_context(message: &str, context: HashMap<String, Value>) {
        Self::log("warning", message, Some(context));
    }

    pub fn notice(message: &str) {
        Self::log("notice", message, None);
    }

    pub fn notice_with_context(message: &str, context: HashMap<String, Value>) {
        Self::log("notice", message, Some(context));
    }

    pub fn info(message: &str) {
        Self::log("info", message, None);
    }

    pub fn info_with_context(message: &str, context: HashMap<String, Value>) {
        Self::log("info", message, Some(context));
    }

    pub fn debug(message: &str) {
        Self::log("debug", message, None);
    }

    pub fn debug_with_context(message: &str, context: HashMap<String, Value>) {
        Self::log("debug", message, Some(context));
    }

    pub fn channel(name: &str) -> ChannelLogger {
        ChannelLogger::new(name.to_string())
    }

    fn log(level: &str, message: &str, context: Option<HashMap<String, Value>>) {
        if let Some(manager_mutex) = LOG_MANAGER.get() {
            if let Ok(mut manager) = manager_mutex.lock() {
                if let Err(_) = manager.log(level, message, context.clone()) {
                    // Fallback to tracing if our logging fails
                    Self::fallback_log(level, message, context);
                }
                return;
            }
        }

        // Fallback to tracing if manager not available
        Self::fallback_log(level, message, context);
    }

    fn fallback_log(level: &str, message: &str, context: Option<HashMap<String, Value>>) {
        match level {
            "emergency" | "alert" | "critical" | "error" => tracing::error!("{}", message),
            "warning" => tracing::warn!("{}", message),
            "notice" | "info" => tracing::info!("{}", message),
            "debug" => tracing::debug!("{}", message),
            _ => tracing::info!("{}", message),
        }

        if let Some(ctx) = context {
            tracing::info!("Context: {:?}", ctx);
        }
    }
}

pub struct ChannelLogger {
    channel_name: String,
}

impl ChannelLogger {
    pub fn new(channel_name: String) -> Self {
        Self { channel_name }
    }

    pub fn emergency(&self, message: &str) {
        self.log("emergency", message, None);
    }

    pub fn emergency_with_context(&self, message: &str, context: HashMap<String, Value>) {
        self.log("emergency", message, Some(context));
    }

    pub fn alert(&self, message: &str) {
        self.log("alert", message, None);
    }

    pub fn alert_with_context(&self, message: &str, context: HashMap<String, Value>) {
        self.log("alert", message, Some(context));
    }

    pub fn critical(&self, message: &str) {
        self.log("critical", message, None);
    }

    pub fn critical_with_context(&self, message: &str, context: HashMap<String, Value>) {
        self.log("critical", message, Some(context));
    }

    pub fn error(&self, message: &str) {
        self.log("error", message, None);
    }

    pub fn error_with_context(&self, message: &str, context: HashMap<String, Value>) {
        self.log("error", message, Some(context));
    }

    pub fn warning(&self, message: &str) {
        self.log("warning", message, None);
    }

    pub fn warning_with_context(&self, message: &str, context: HashMap<String, Value>) {
        self.log("warning", message, Some(context));
    }

    pub fn notice(&self, message: &str) {
        self.log("notice", message, None);
    }

    pub fn notice_with_context(&self, message: &str, context: HashMap<String, Value>) {
        self.log("notice", message, Some(context));
    }

    pub fn info(&self, message: &str) {
        self.log("info", message, None);
    }

    pub fn info_with_context(&self, message: &str, context: HashMap<String, Value>) {
        self.log("info", message, Some(context));
    }

    pub fn debug(&self, message: &str) {
        self.log("debug", message, None);
    }

    pub fn debug_with_context(&self, message: &str, context: HashMap<String, Value>) {
        self.log("debug", message, Some(context));
    }

    fn log(&self, level: &str, message: &str, context: Option<HashMap<String, Value>>) {
        if let Some(manager_mutex) = LOG_MANAGER.get() {
            if let Ok(mut manager) = manager_mutex.lock() {
                if let Err(_) = manager.log_to_channel(&self.channel_name, level, message, context.clone()) {
                    // Fallback to tracing if our logging fails
                    self.fallback_log(level, message, context);
                }
                return;
            }
        }

        // Fallback to tracing if manager not available
        self.fallback_log(level, message, context);
    }

    fn fallback_log(&self, level: &str, message: &str, context: Option<HashMap<String, Value>>) {
        let channel_message = format!("[{}] {}", self.channel_name, message);
        match level {
            "emergency" | "alert" | "critical" | "error" => tracing::error!("{}", channel_message),
            "warning" => tracing::warn!("{}", channel_message),
            "notice" | "info" => tracing::info!("{}", channel_message),
            "debug" => tracing::debug!("{}", channel_message),
            _ => tracing::info!("{}", channel_message),
        }

        if let Some(ctx) = context {
            tracing::info!("[{}] Context: {:?}", self.channel_name, ctx);
        }
    }
}