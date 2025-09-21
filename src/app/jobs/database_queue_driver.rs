use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use diesel::prelude::*;
use crate::database::DbPool;
use crate::schema::jobs;
use crate::app::jobs::{QueueDriver, JobMetadata, JobStatus};

/// Database row representation for jobs table
#[derive(Debug, Queryable, Identifiable, QueryableByName)]
#[diesel(table_name = jobs)]
struct JobRow {
    id: String,          // Bpchar maps to String
    queue_name: String,  // Varchar maps to String
    job_name: String,    // Varchar maps to String
    payload: serde_json::Value, // Jsonb maps to serde_json::Value
    attempts: i32,       // Int4 maps to i32
    max_attempts: i32,   // Int4 maps to i32
    status: String,      // Varchar maps to String
    priority: i32,       // Int4 maps to i32
    available_at: DateTime<Utc>,        // Timestamptz maps to DateTime<Utc>
    reserved_at: Option<DateTime<Utc>>,  // Nullable<Timestamptz> maps to Option<DateTime<Utc>>
    processed_at: Option<DateTime<Utc>>, // Nullable<Timestamptz> maps to Option<DateTime<Utc>>
    failed_at: Option<DateTime<Utc>>,    // Nullable<Timestamptz> maps to Option<DateTime<Utc>>
    error_message: Option<String>,       // Nullable<Text> maps to Option<String>
    timeout_seconds: Option<i32>,        // Nullable<Int4> maps to Option<i32>
    created_at: DateTime<Utc>,           // Timestamptz maps to DateTime<Utc>
    updated_at: DateTime<Utc>,           // Timestamptz maps to DateTime<Utc>
}

/// Database-backed queue driver using PostgreSQL
#[derive(Debug, Clone)]
pub struct DatabaseQueueDriver {
    pool: DbPool,
}

impl DatabaseQueueDriver {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Convert database row to JobMetadata
    fn row_to_job_metadata(&self, row: &JobRow) -> Result<JobMetadata> {
        let status = match row.status.as_str() {
            "pending" => JobStatus::Pending,
            "processing" => JobStatus::Processing,
            "completed" => JobStatus::Completed,
            "failed" => JobStatus::Failed,
            "retrying" => JobStatus::Retrying,
            _ => JobStatus::Pending,
        };

        Ok(JobMetadata {
            id: row.id.clone(),
            job_name: row.job_name.clone(),
            queue_name: row.queue_name.clone(),
            status,
            payload: serde_json::to_string(&row.payload)?,
            attempts: row.attempts as u32,
            max_attempts: row.max_attempts as u32,
            priority: row.priority,
            created_at: row.created_at,
            updated_at: row.updated_at,
            scheduled_at: Some(row.available_at),
            failed_at: row.failed_at,
            error_message: row.error_message.clone(),
        })
    }
}

#[async_trait]
impl QueueDriver for DatabaseQueueDriver {
    async fn push(&self, metadata: JobMetadata) -> Result<()> {
        let status_str = match metadata.status {
            JobStatus::Pending => "pending",
            JobStatus::Processing => "processing",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Retrying => "retrying",
        };

        let mut conn = self.pool.get()?;

        diesel::insert_into(jobs::table)
            .values((
                jobs::id.eq(&metadata.id),
                jobs::queue_name.eq(&metadata.queue_name),
                jobs::job_name.eq(&metadata.job_name),
                jobs::payload.eq(serde_json::from_str::<serde_json::Value>(&metadata.payload)?),
                jobs::attempts.eq(metadata.attempts as i32),
                jobs::max_attempts.eq(metadata.max_attempts as i32),
                jobs::status.eq(status_str),
                jobs::priority.eq(metadata.priority),
                jobs::available_at.eq(metadata.scheduled_at.unwrap_or(metadata.created_at)),
                jobs::failed_at.eq(metadata.failed_at),
                jobs::error_message.eq(&metadata.error_message),
                jobs::created_at.eq(metadata.created_at),
                jobs::updated_at.eq(metadata.updated_at),
            ))
            .execute(&mut conn)?;

        tracing::info!("Job {} pushed to database queue '{}'", metadata.id, metadata.queue_name);
        Ok(())
    }

    async fn pop(&self, queue_name: &str) -> Result<Option<JobMetadata>> {
        let mut conn = self.pool.get()?;

        // For this complex query with FOR UPDATE SKIP LOCKED, we use raw SQL
        // since Diesel doesn't directly support these PostgreSQL-specific features
        use diesel::sql_query;

        let job_row: Option<JobRow> = sql_query(
            r#"
            UPDATE jobs
            SET status = 'processing',
                reserved_at = NOW(),
                updated_at = NOW()
            WHERE id = (
                SELECT id
                FROM jobs
                WHERE queue_name = $1
                  AND status = 'pending'
                  AND available_at <= NOW()
                ORDER BY priority ASC, created_at ASC
                FOR UPDATE SKIP LOCKED
                LIMIT 1
            )
            RETURNING *
            "#
        )
        .bind::<diesel::sql_types::Text, _>(queue_name)
        .get_result(&mut conn)
        .optional()?;

        if let Some(row) = job_row {
            let metadata = self.row_to_job_metadata(&row)?;
            tracing::info!("Job {} popped from queue '{}'", metadata.id, queue_name);
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    async fn size(&self, queue_name: &str) -> Result<u64> {
        let mut conn = self.pool.get()?;

        let count = jobs::table
            .filter(jobs::queue_name.eq(queue_name))
            .filter(jobs::status.eq("pending"))
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(count as u64)
    }

    async fn delete(&self, job_id: &str) -> Result<()> {
        let mut conn = self.pool.get()?;

        diesel::delete(jobs::table.filter(jobs::id.eq(job_id)))
            .execute(&mut conn)?;

        tracing::info!("Job {} deleted from database queue", job_id);
        Ok(())
    }

    async fn update(&self, metadata: &JobMetadata) -> Result<()> {
        let status_str = match metadata.status {
            JobStatus::Pending => "pending",
            JobStatus::Processing => "processing",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Retrying => "retrying",
        };

        let mut conn = self.pool.get()?;
        let now = chrono::Utc::now();

        let processed_at = if status_str == "completed" {
            Some(now)
        } else {
            None
        };

        diesel::update(jobs::table.filter(jobs::id.eq(&metadata.id)))
            .set((
                jobs::status.eq(status_str),
                jobs::attempts.eq(metadata.attempts as i32),
                jobs::failed_at.eq(metadata.failed_at),
                jobs::error_message.eq(&metadata.error_message),
                jobs::processed_at.eq(processed_at),
                jobs::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        tracing::info!("Job {} updated in database queue", metadata.id);
        Ok(())
    }

    async fn failed_jobs(&self, limit: Option<u32>) -> Result<Vec<JobMetadata>> {
        let limit_val = limit.unwrap_or(100) as i64;

        let mut conn = self.pool.get()?;

        let rows = jobs::table
            .filter(jobs::status.eq("failed"))
            .order(jobs::failed_at.desc())
            .limit(limit_val)
            .load::<JobRow>(&mut conn)?;

        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(self.row_to_job_metadata(&row)?);
        }

        Ok(jobs)
    }

    async fn retry_job(&self, job_id: &str) -> Result<()> {
        let mut conn = self.pool.get()?;
        let now = chrono::Utc::now();

        diesel::update(jobs::table
            .filter(jobs::id.eq(job_id))
            .filter(jobs::status.eq("failed")))
            .set((
                jobs::status.eq("pending"),
                jobs::attempts.eq(0),
                jobs::failed_at.eq::<Option<DateTime<Utc>>>(None),
                jobs::error_message.eq::<Option<String>>(None),
                jobs::reserved_at.eq::<Option<DateTime<Utc>>>(None),
                jobs::processed_at.eq::<Option<DateTime<Utc>>>(None),
                jobs::available_at.eq(now),
                jobs::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        tracing::info!("Job {} retried in database queue", job_id);
        Ok(())
    }

    fn driver_name(&self) -> &'static str {
        "database"
    }
}

/// Additional methods specific to the database driver
impl DatabaseQueueDriver {
    /// Get all jobs in a specific queue
    pub async fn get_queue_jobs(&self, queue_name: &str, status: Option<JobStatus>, limit: Option<u32>) -> Result<Vec<JobMetadata>> {
        let limit_val = limit.unwrap_or(100) as i64;

        let status_str = status.map(|s| match s {
            JobStatus::Pending => "pending",
            JobStatus::Processing => "processing",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Retrying => "retrying",
        });

        let mut conn = self.pool.get()?;

        let mut query = jobs::table
            .filter(jobs::queue_name.eq(queue_name))
            .into_boxed();

        if let Some(status_filter) = status_str {
            query = query.filter(jobs::status.eq(status_filter));
        }

        let rows = query
            .order(jobs::created_at.desc())
            .limit(limit_val)
            .load::<JobRow>(&mut conn)?;

        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(self.row_to_job_metadata(&row)?);
        }

        Ok(jobs)
    }

    /// Clean up old completed jobs
    pub async fn cleanup_completed_jobs(&self, older_than_days: u32) -> Result<u64> {
        let mut conn = self.pool.get()?;
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(older_than_days as i64);

        let deleted_count = diesel::delete(jobs::table
            .filter(jobs::status.eq("completed"))
            .filter(jobs::processed_at.lt(cutoff_date)))
            .execute(&mut conn)?;

        tracing::info!("Cleaned up {} completed jobs older than {} days", deleted_count, older_than_days);
        Ok(deleted_count as u64)
    }

    /// Get queue statistics
    pub async fn get_queue_stats(&self, queue_name: &str) -> Result<QueueStats> {
        let mut conn = self.pool.get()?;

        let pending = jobs::table
            .filter(jobs::queue_name.eq(queue_name))
            .filter(jobs::status.eq("pending"))
            .count()
            .get_result::<i64>(&mut conn)?;

        let processing = jobs::table
            .filter(jobs::queue_name.eq(queue_name))
            .filter(jobs::status.eq("processing"))
            .count()
            .get_result::<i64>(&mut conn)?;

        let completed = jobs::table
            .filter(jobs::queue_name.eq(queue_name))
            .filter(jobs::status.eq("completed"))
            .count()
            .get_result::<i64>(&mut conn)?;

        let failed = jobs::table
            .filter(jobs::queue_name.eq(queue_name))
            .filter(jobs::status.eq("failed"))
            .count()
            .get_result::<i64>(&mut conn)?;

        let retrying = jobs::table
            .filter(jobs::queue_name.eq(queue_name))
            .filter(jobs::status.eq("retrying"))
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(QueueStats {
            queue_name: queue_name.to_string(),
            pending: pending as u64,
            processing: processing as u64,
            completed: completed as u64,
            failed: failed as u64,
            retrying: retrying as u64,
        })
    }
}

#[derive(Debug, Clone)]
pub struct QueueStats {
    pub queue_name: String,
    pub pending: u64,
    pub processing: u64,
    pub completed: u64,
    pub failed: u64,
    pub retrying: u64,
}