//! Activity logging macros and utilities
//!
//! This module provides convenient macros and functions for logging activities
//! similar to Spatie's Laravel ActivityLog package.

pub use crate::app::models::activity_log::*;
pub use crate::app::models::{HasModelType, HasRoles};
pub use crate::app::services::activity_log_service::*;
pub use crate::app::http::middleware::correlation_middleware::{CorrelationContext, extractors};

/// Log an activity with a simple description
///
/// # Example
/// ```rust
/// activity!("User logged in");
/// ```
#[macro_export]
macro_rules! activity {
    ($description:expr) => {{
        use $crate::app::activity_log::activity;
        activity($description).await
    }};
}

/// Log an activity with a specific log name
///
/// # Example
/// ```rust
/// activity_log!("auth", "User logged in successfully");
/// ```
#[macro_export]
macro_rules! activity_log {
    ($log_name:expr, $description:expr) => {{
        use $crate::app::activity_log::activity_for_log;
        activity_for_log($log_name, $description).await
    }};
}

/// Log an activity with correlation ID
///
/// # Example
/// ```rust
/// activity_with_correlation!("Processing payment", correlation_id);
/// ```
#[macro_export]
macro_rules! activity_with_correlation {
    ($description:expr, $correlation_id:expr) => {{
        use $crate::app::activity_log::activity_with_correlation;
        activity_with_correlation($description, $correlation_id).await
    }};
}

/// Log an activity with detailed context using the builder pattern
///
/// # Example
/// ```rust
/// activity_builder!()
///     .log_name("user_management")
///     .description("User profile updated")
///     .performed_on(&user)
///     .caused_by(&admin_user)
///     .with_property("fields_changed", vec!["email", "name"])
///     .event("updated")
///     .log()
///     .await?;
/// ```
#[macro_export]
macro_rules! activity_builder {
    () => {{
        use $crate::app::models::activity_log::ActivityLog;
        ActivityLog::builder()
    }};
}

/// Create activity logger for a specific model
///
/// # Example
/// ```rust
/// let logger = activity_on!(user);
/// logger.description("Profile updated").log().await?;
/// ```
#[macro_export]
macro_rules! activity_on {
    ($subject:expr) => {{
        use $crate::app::models::activity_log::ActivityLog;
        ActivityLog::performed_on($subject)
    }};
}

/// Create activity logger for a specific causer
///
/// # Example
/// ```rust
/// let logger = activity_by!(admin_user);
/// logger.description("User account locked").log().await?;
/// ```
#[macro_export]
macro_rules! activity_by {
    ($causer:expr) => {{
        use $crate::app::models::activity_log::ActivityLog;
        ActivityLog::caused_by($causer)
    }};
}

/// Log an activity with automatic correlation from request context
///
/// # Example
/// ```rust
/// // In a request handler where correlation_id is a DieselUlid
/// activity_correlated!(correlation_id, "User action performed");
/// ```
#[macro_export]
macro_rules! activity_correlated {
    ($correlation_id:expr, $description:expr) => {{
        use $crate::app::models::activity_log::ActivityLog;
        ActivityLog::with_correlation_id($correlation_id)
            .description($description)
            .log()
            .await
    }};
}

/// Create an activity log in a specific batch
///
/// # Example
/// ```rust
/// let batch_id = uuid::Uuid::new_v4().to_string();
/// activity_batch!(batch_id, "Bulk operation started");
/// ```
#[macro_export]
macro_rules! activity_batch {
    ($batch_uuid:expr, $description:expr) => {{
        use $crate::app::models::activity_log::ActivityLog;
        ActivityLog::in_batch($batch_uuid)
            .description($description)
            .log()
            .await
    }};
}

/// Helper trait for models to enable easy activity logging
pub trait LogsActivity: HasModelType + HasId
where
    Self: Sized,
{
    /// Log an activity performed on this model
    fn log_activity(&self, description: &str) -> ActivityLogBuilder {
        ActivityLog::performed_on(self).description(description)
    }

    /// Log an activity caused by this model
    fn log_activity_as_causer(&self, description: &str) -> ActivityLogBuilder {
        ActivityLog::caused_by(self).description(description)
    }

    /// Log an activity with both subject and causer being this model
    fn log_self_activity(&self, description: &str) -> ActivityLogBuilder {
        ActivityLog::performed_on(self)
            .caused_by(self)
            .description(description)
    }
}

// Blanket implementation for all models that have the required traits
impl<T> LogsActivity for T where T: HasModelType + HasId + Sized {}

/// Helper functions for common activity logging patterns
pub mod helpers {
    use super::*;
    use anyhow::Result;
    use crate::app::models::DieselUlid;

    /// Log a user authentication event
    pub async fn log_auth_event(
        user_id: &str,
        event: &str,
        correlation_id: Option<DieselUlid>
    ) -> Result<ActivityLog> {
        let mut builder = ActivityLog::for_log("auth")
            .description(&format!("User {}", event))
            .subject_type("User")
            .subject_id(user_id)
            .event(event);

        if let Some(correlation_id) = correlation_id {
            builder = builder.correlation_id(correlation_id);
        }

        builder.log().await
    }

    /// Log a model creation event
    pub async fn log_created<T: HasModelType + HasId>(
        model: &T,
        causer_id: Option<&str>,
        correlation_id: Option<DieselUlid>
    ) -> Result<ActivityLog> {
        let mut builder = ActivityLog::performed_on(model)
            .description(&format!("{} created", T::model_type()))
            .event("created");

        if let Some(causer_id) = causer_id {
            builder = builder.causer_type("User").causer_id(causer_id);
        }

        if let Some(correlation_id) = correlation_id {
            builder = builder.correlation_id(correlation_id);
        }

        builder.log().await
    }

    /// Log a model update event
    pub async fn log_updated<T: HasModelType + HasId>(
        model: &T,
        causer_id: Option<&str>,
        correlation_id: Option<DieselUlid>,
        changed_fields: Option<Vec<&str>>
    ) -> Result<ActivityLog> {
        let mut builder = ActivityLog::performed_on(model)
            .description(&format!("{} updated", T::model_type()))
            .event("updated");

        if let Some(causer_id) = causer_id {
            builder = builder.causer_type("User").causer_id(causer_id);
        }

        if let Some(correlation_id) = correlation_id {
            builder = builder.correlation_id(correlation_id);
        }

        if let Some(fields) = changed_fields {
            builder = builder.with_property("changed_fields", fields);
        }

        builder.log().await
    }

    /// Log a model deletion event
    pub async fn log_deleted<T: HasModelType + HasId>(
        model: &T,
        causer_id: Option<&str>,
        correlation_id: Option<DieselUlid>
    ) -> Result<ActivityLog> {
        let mut builder = ActivityLog::performed_on(model)
            .description(&format!("{} deleted", T::model_type()))
            .event("deleted");

        if let Some(causer_id) = causer_id {
            builder = builder.causer_type("User").causer_id(causer_id);
        }

        if let Some(correlation_id) = correlation_id {
            builder = builder.correlation_id(correlation_id);
        }

        builder.log().await
    }

    /// Log a batch operation
    pub async fn log_batch_operation(
        operation: &str,
        count: usize,
        correlation_id: Option<DieselUlid>
    ) -> Result<ActivityLog> {
        let batch_uuid = uuid::Uuid::new_v4().to_string();

        let mut builder = ActivityLog::in_batch(&batch_uuid)
            .description(&format!("Batch operation: {} ({} items)", operation, count))
            .event("batch_operation")
            .with_property("operation", operation)
            .with_property("count", count);

        if let Some(correlation_id) = correlation_id {
            builder = builder.correlation_id(correlation_id);
        }

        builder.log().await
    }
}

/// Re-export commonly used items
pub mod prelude {
    pub use super::{
        ActivityLog, ActivityLogBuilder,
        ActivityLogService, ActivityLogQueryBuilder,
        LogsActivity, helpers, HasId
    };
    pub use crate::{
        activity, activity_log, activity_with_correlation,
        activity_builder, activity_on, activity_by,
        activity_correlated, activity_batch
    };
}