use anyhow::Result;
use serde_json::json;
use crate::app::models::HasModelType;
use crate::app::models::activity_log::HasId;
use crate::app::http::middleware::activity_logging_middleware::ActivityLogger as MiddlewareActivityLogger;

/// Trait for service-level activity logging using the existing middleware ActivityLogger
pub trait ServiceActivityLogger {
    /// Get the activity logger instance
    fn get_activity_logger(&self) -> MiddlewareActivityLogger {
        MiddlewareActivityLogger::new("service_operation")
    }

    /// Get the activity logger with correlation ID
    fn get_activity_logger_with_correlation(&self, correlation_id: crate::app::models::DieselUlid) -> MiddlewareActivityLogger {
        MiddlewareActivityLogger::new("service_operation").with_correlation_id(correlation_id)
    }

    /// Get the activity logger with causer information
    fn get_activity_logger_with_causer(&self, causer_type: &str, causer_id: &str) -> MiddlewareActivityLogger {
        MiddlewareActivityLogger::new("service_operation").with_causer(causer_type, causer_id)
    }

    /// Log a create operation using the middleware ActivityLogger
    async fn log_created<T: HasModelType + HasId>(
        &self,
        entity: &T,
        causer_id: Option<&str>,
        properties: Option<serde_json::Value>,
    ) -> Result<()> {
        let mut logger = self.get_activity_logger();

        if let Some(id) = causer_id {
            logger = logger.with_causer("User", id);
        }

        let mut props = properties.unwrap_or_else(|| json!({}));
        props["model_type"] = json!(T::model_type());
        props["action"] = json!("create");

        logger.log_create(T::model_type(), &entity.id(), Some(props)).await
            .map_err(anyhow::Error::from)
    }

    /// Log an update operation
    async fn log_updated<T: HasModelType + HasId>(
        &self,
        entity: &T,
        changes: serde_json::Value,
        causer_id: Option<&str>,
    ) -> Result<()> {
        let mut logger = self.get_activity_logger();

        if let Some(id) = causer_id {
            logger = logger.with_causer("User", id);
        }

        let props = json!({
            "model_type": T::model_type(),
            "action": "update",
            "changes": changes
        });

        logger.log_update(T::model_type(), &entity.id(), Some(props)).await
            .map_err(anyhow::Error::from)
    }

    /// Log a delete operation
    async fn log_deleted<T: HasModelType + HasId>(
        &self,
        entity: &T,
        causer_id: Option<&str>,
    ) -> Result<()> {
        let mut logger = self.get_activity_logger();

        if let Some(id) = causer_id {
            logger = logger.with_causer("User", id);
        }

        let props = json!({
            "model_type": T::model_type(),
            "action": "delete"
        });

        logger.log_delete(T::model_type(), &entity.id(), Some(props)).await
            .map_err(anyhow::Error::from)
    }

    /// Log a view operation
    async fn log_viewed<T: HasModelType + HasId>(
        &self,
        entity: &T,
        causer_id: Option<&str>,
    ) -> Result<()> {
        let mut logger = self.get_activity_logger();

        if let Some(id) = causer_id {
            logger = logger.with_causer("User", id);
        }

        let props = json!({
            "model_type": T::model_type(),
            "action": "view"
        });

        logger.log_view(T::model_type(), &entity.id(), Some(props)).await
            .map_err(anyhow::Error::from)
    }

    /// Log authentication events using the middleware ActivityLogger
    async fn log_authentication(
        &self,
        event: &str,
        user_id: Option<&str>,
        success: bool,
        properties: Option<serde_json::Value>,
    ) -> Result<()> {
        let logger = MiddlewareActivityLogger::new("authentication");

        match (event, success) {
            ("login", true) => {
                if let Some(id) = user_id {
                    logger.log_login(id, properties).await
                        .map_err(anyhow::Error::from)
                } else {
                    Ok(())
                }
            },
            ("logout", _) => {
                if let Some(id) = user_id {
                    logger.log_logout(id, properties).await
                        .map_err(anyhow::Error::from)
                } else {
                    Ok(())
                }
            },
            ("login", false) => {
                if let Some(email) = user_id {
                    let reason = properties
                        .as_ref()
                        .and_then(|p| p.get("reason"))
                        .and_then(|r| r.as_str())
                        .unwrap_or("Unknown reason");
                    logger.log_failed_login(email, reason, properties.clone()).await
                        .map_err(anyhow::Error::from)
                } else {
                    Ok(())
                }
            },
            _ => {
                let description = format!("Authentication event: {} (success: {})", event, success);
                logger.log_custom(&description, Some(&format!("auth.{}", event)), properties).await
                    .map_err(anyhow::Error::from)
            }
        }
    }

    /// Log system events
    fn log_system_event(
        &self,
        event: &str,
        description: &str,
        properties: Option<serde_json::Value>,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        let event = event.to_string();
        let description = description.to_string();
        async move {
            let logger = MiddlewareActivityLogger::new("system");
            logger.log_custom(&description, Some(&event), properties).await
                .map_err(anyhow::Error::from)
        }
    }
}

/// Helper struct that implements ServiceActivityLogger for easy use
pub struct DefaultServiceActivityLogger;

impl ServiceActivityLogger for DefaultServiceActivityLogger {}