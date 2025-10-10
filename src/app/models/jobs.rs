use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::{HasModelType, activity_log::HasId};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::jobs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Job {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub queue_name: String,
    pub job_name: String,
    pub payload: serde_json::Value,
    pub attempts: i32,
    pub max_attempts: i32,
    pub status: String,
    pub priority: i32,
    pub available_at: DateTime<Utc>,
    pub reserved_at: Option<DateTime<Utc>>,
    pub processed_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub timeout_seconds: Option<i32>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Reserved,
    Processing,
    Completed,
    Failed,
    Retrying,
}

impl From<String> for JobStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "pending" => JobStatus::Pending,
            "reserved" => JobStatus::Reserved,
            "processing" => JobStatus::Processing,
            "completed" => JobStatus::Completed,
            "failed" => JobStatus::Failed,
            "retrying" => JobStatus::Retrying,
            _ => JobStatus::Pending,
        }
    }
}

impl From<JobStatus> for String {
    fn from(status: JobStatus) -> Self {
        match status {
            JobStatus::Pending => "pending".to_string(),
            JobStatus::Reserved => "reserved".to_string(),
            JobStatus::Processing => "processing".to_string(),
            JobStatus::Completed => "completed".to_string(),
            JobStatus::Failed => "failed".to_string(),
            JobStatus::Retrying => "retrying".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateJob {
    pub queue_name: String,
    pub job_name: String,
    pub payload: serde_json::Value,
    pub max_attempts: i32,
    pub priority: i32,
    pub available_at: Option<DateTime<Utc>>,
    pub timeout_seconds: Option<i32>,
}
impl Job {
    pub fn new(
        queue_name: String,
        job_name: String,
        payload: serde_json::Value,
        max_attempts: i32,
        priority: i32,
        available_at: Option<DateTime<Utc>>,
        timeout_seconds: Option<i32>,
    ) -> Self {
        let now = Utc::now();
        Job {
            id: DieselUlid::new(),
            queue_name,
            job_name,
            payload,
            attempts: 0,
            max_attempts,
            status: JobStatus::Pending.into(),
            priority,
            available_at: available_at.unwrap_or(now),
            reserved_at: None,
            processed_at: None,
            failed_at: None,
            error_message: None,
            timeout_seconds,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn status_enum(&self) -> JobStatus {
        self.status.clone().into()
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.status_enum(), JobStatus::Pending)
    }

    pub fn is_processing(&self) -> bool {
        matches!(self.status_enum(), JobStatus::Processing | JobStatus::Reserved)
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.status_enum(), JobStatus::Completed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.status_enum(), JobStatus::Failed)
    }

    pub fn can_retry(&self) -> bool {
        self.is_failed() && self.attempts < self.max_attempts
    }

    pub fn mark_reserved(&mut self) {
        self.status = JobStatus::Reserved.into();
        self.reserved_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn mark_processing(&mut self) {
        self.status = JobStatus::Processing.into();
        self.updated_at = Utc::now();
    }

    pub fn mark_completed(&mut self) {
        self.status = JobStatus::Completed.into();
        self.processed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn mark_failed(&mut self, error_message: String) {
        self.status = JobStatus::Failed.into();
        self.failed_at = Some(Utc::now());
        self.error_message = Some(error_message);
        self.attempts += 1;
        self.updated_at = Utc::now();
    }
}

impl HasId for Job {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for Job {
    fn table_name() -> &'static str {
        "jobs"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "queue_name",
            "job_name",
            "status",
            "priority",
            "attempts",
            "max_attempts",
            "available_at",
            "reserved_at",
            "processed_at",
            "failed_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "queue_name",
            "job_name",
            "status",
            "priority",
            "attempts",
            "available_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "queue_name",
            "job_name",
            "payload",
            "attempts",
            "max_attempts",
            "status",
            "priority",
            "available_at",
            "reserved_at",
            "processed_at",
            "failed_at",
            "error_message",
            "timeout_seconds",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("priority", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![]
    }
}

impl HasModelType for Job {
    fn model_type() -> &'static str {
        "Job"
    }
}

crate::impl_query_builder_service!(Job);