use anyhow::Result;
use std::env;

/// Configuration for the activity log system
#[derive(Debug, Clone)]
pub struct ActivityLogConfig {
    /// Whether activity logging is enabled globally
    pub enabled: bool,

    /// Default log name to use when none is specified
    pub default_log_name: Option<String>,

    /// Whether to automatically log model events (created, updated, deleted)
    pub auto_log_model_events: bool,

    /// Whether to automatically include correlation IDs in activity logs
    pub auto_correlation: bool,

    /// Whether to include the causer information in activity logs
    pub include_causer: bool,

    /// Whether to include the subject information in activity logs
    pub include_subject: bool,

    /// Maximum number of activity logs to keep in database (0 = unlimited)
    pub max_log_count: u64,

    /// Number of days to keep activity logs (0 = unlimited)
    pub retention_days: u32,

    /// Whether to automatically clean old activity logs
    pub auto_cleanup: bool,

    /// List of events to exclude from logging
    pub excluded_events: Vec<String>,

    /// List of model types to exclude from logging
    pub excluded_models: Vec<String>,

    /// Whether to log properties changes in model updates
    pub log_properties: bool,

    /// Maximum size of properties JSON (in bytes)
    pub max_properties_size: usize,
}

impl ActivityLogConfig {
    pub fn from_env() -> Result<Self> {
        let enabled = env::var("ACTIVITY_LOG_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let default_log_name = env::var("ACTIVITY_LOG_DEFAULT_NAME").ok();

        let auto_log_model_events = env::var("ACTIVITY_LOG_AUTO_MODEL_EVENTS")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let auto_correlation = env::var("ACTIVITY_LOG_AUTO_CORRELATION")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let include_causer = env::var("ACTIVITY_LOG_INCLUDE_CAUSER")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let include_subject = env::var("ACTIVITY_LOG_INCLUDE_SUBJECT")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let max_log_count = env::var("ACTIVITY_LOG_MAX_COUNT")
            .unwrap_or_else(|_| "0".to_string())
            .parse::<u64>()
            .unwrap_or(0);

        let retention_days = env::var("ACTIVITY_LOG_RETENTION_DAYS")
            .unwrap_or_else(|_| "0".to_string())
            .parse::<u32>()
            .unwrap_or(0);

        let auto_cleanup = env::var("ACTIVITY_LOG_AUTO_CLEANUP")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let excluded_events = env::var("ACTIVITY_LOG_EXCLUDED_EVENTS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let excluded_models = env::var("ACTIVITY_LOG_EXCLUDED_MODELS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let log_properties = env::var("ACTIVITY_LOG_PROPERTIES")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let max_properties_size = env::var("ACTIVITY_LOG_MAX_PROPERTIES_SIZE")
            .unwrap_or_else(|_| "65536".to_string()) // 64KB default
            .parse::<usize>()
            .unwrap_or(65536);

        Ok(ActivityLogConfig {
            enabled,
            default_log_name,
            auto_log_model_events,
            auto_correlation,
            include_causer,
            include_subject,
            max_log_count,
            retention_days,
            auto_cleanup,
            excluded_events,
            excluded_models,
            log_properties,
            max_properties_size,
        })
    }

    /// Check if activity logging is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if an event should be logged
    pub fn should_log_event(&self, event: &str) -> bool {
        self.enabled && !self.excluded_events.contains(&event.to_string())
    }

    /// Check if a model type should be logged
    pub fn should_log_model(&self, model_type: &str) -> bool {
        self.enabled && !self.excluded_models.contains(&model_type.to_string())
    }

    /// Get the default log name
    pub fn get_default_log_name(&self) -> Option<&str> {
        self.default_log_name.as_deref()
    }

    /// Check if properties should be included in logs
    pub fn should_log_properties(&self) -> bool {
        self.enabled && self.log_properties
    }

    /// Validate properties size
    pub fn is_properties_size_valid(&self, size: usize) -> bool {
        size <= self.max_properties_size
    }

    /// Check if cleanup is needed based on retention settings
    pub fn should_cleanup(&self) -> bool {
        self.enabled && self.auto_cleanup && (self.retention_days > 0 || self.max_log_count > 0)
    }

    /// Check if correlation tracking should be automatic
    pub fn should_auto_correlate(&self) -> bool {
        self.enabled && self.auto_correlation
    }

    /// Check if model events should be automatically logged
    pub fn should_auto_log_model_events(&self) -> bool {
        self.enabled && self.auto_log_model_events
    }
}

impl Default for ActivityLogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_log_name: None,
            auto_log_model_events: false,
            auto_correlation: true,
            include_causer: true,
            include_subject: true,
            max_log_count: 0,
            retention_days: 0,
            auto_cleanup: false,
            excluded_events: Vec::new(),
            excluded_models: Vec::new(),
            log_properties: true,
            max_properties_size: 65536,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_activity_log_config_default() {
        let config = ActivityLogConfig::default();
        assert!(config.enabled);
        assert!(config.auto_correlation);
        assert!(config.include_causer);
        assert!(config.include_subject);
        assert_eq!(config.max_log_count, 0);
        assert_eq!(config.retention_days, 0);
        assert!(!config.auto_cleanup);
    }

    #[test]
    fn test_should_log_event() {
        let mut config = ActivityLogConfig::default();
        config.excluded_events = vec!["test_event".to_string()];

        assert!(config.should_log_event("normal_event"));
        assert!(!config.should_log_event("test_event"));
    }

    #[test]
    fn test_should_log_model() {
        let mut config = ActivityLogConfig::default();
        config.excluded_models = vec!["TestModel".to_string()];

        assert!(config.should_log_model("User"));
        assert!(!config.should_log_model("TestModel"));
    }

    #[test]
    fn test_properties_size_validation() {
        let config = ActivityLogConfig::default();
        assert!(config.is_properties_size_valid(1000));
        assert!(config.is_properties_size_valid(65536));
        assert!(!config.is_properties_size_valid(100000));
    }
}