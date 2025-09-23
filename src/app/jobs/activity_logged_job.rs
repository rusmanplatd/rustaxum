use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use crate::app::jobs::Job;
use crate::app::traits::ServiceActivityLogger;

/// Trait that extends Job with automatic activity logging
#[async_trait]
pub trait ActivityLoggedJob: Job + ServiceActivityLogger {
    /// The user ID who triggered this job (if any)
    fn triggered_by(&self) -> Option<&str> {
        None
    }

    /// Additional properties to log with job execution
    fn job_properties(&self) -> serde_json::Value {
        json!({
            "job_name": self.job_name(),
            "queue_name": self.queue_name(),
            "priority": self.priority(),
            "max_attempts": self.max_attempts(),
            "timeout": self.timeout()
        })
    }

    /// Execute the job with automatic activity logging
    async fn handle_with_logging(&self) -> Result<()> {
        // Log job start
        let start_properties = json!({
            "status": "started",
            "triggered_by": self.triggered_by(),
            "job_details": self.job_properties()
        });

        if let Err(e) = self.log_system_event(
            "job_started",
            &format!("Job {} started", self.job_name()),
            Some(start_properties)
        ).await {
            eprintln!("Failed to log job start: {}", e);
        }

        // Execute the actual job
        let result = self.handle().await;

        // Log job completion or failure
        match &result {
            Ok(_) => {
                let success_properties = json!({
                    "status": "completed",
                    "triggered_by": self.triggered_by(),
                    "job_details": self.job_properties()
                });

                if let Err(e) = self.log_system_event(
                    "job_completed",
                    &format!("Job {} completed successfully", self.job_name()),
                    Some(success_properties)
                ).await {
                    eprintln!("Failed to log job completion: {}", e);
                }
            }
            Err(err) => {
                let error_properties = json!({
                    "status": "failed",
                    "error": err.to_string(),
                    "triggered_by": self.triggered_by(),
                    "job_details": self.job_properties()
                });

                if let Err(e) = self.log_system_event(
                    "job_failed",
                    &format!("Job {} failed: {}", self.job_name(), err),
                    Some(error_properties)
                ).await {
                    eprintln!("Failed to log job failure: {}", e);
                }
            }
        }

        result
    }
}

/// Default implementation for any job that implements Job and ServiceActivityLogger
impl<T> ActivityLoggedJob for T
where
    T: Job + ServiceActivityLogger + Send + Sync
{}