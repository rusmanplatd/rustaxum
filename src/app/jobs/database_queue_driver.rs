use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgPool, FromRow};
use chrono::{DateTime, Utc};
use serde_json;
use crate::app::jobs::{QueueDriver, JobMetadata, JobStatus};

/// Database row representation for jobs table
#[derive(Debug, FromRow)]
struct JobRow {
    id: String,
    queue_name: String,
    job_name: String,
    payload: serde_json::Value,
    attempts: i32,
    max_attempts: i32,
    status: String,
    priority: i32,
    available_at: DateTime<Utc>,
    reserved_at: Option<DateTime<Utc>>,
    processed_at: Option<DateTime<Utc>>,
    failed_at: Option<DateTime<Utc>>,
    error_message: Option<String>,
    timeout_seconds: Option<i32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Database-backed queue driver using PostgreSQL
#[derive(Debug, Clone)]
pub struct DatabaseQueueDriver {
    pool: PgPool,
}

impl DatabaseQueueDriver {
    pub fn new(pool: PgPool) -> Self {
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
            payload: row.payload.to_string(),
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

        sqlx::query(
            r#"
            INSERT INTO jobs (
                id, queue_name, job_name, payload, attempts, max_attempts,
                status, priority, available_at, failed_at, error_message,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#
        )
        .bind(&metadata.id)
        .bind(&metadata.queue_name)
        .bind(&metadata.job_name)
        .bind(serde_json::from_str::<serde_json::Value>(&metadata.payload)?)
        .bind(metadata.attempts as i32)
        .bind(metadata.max_attempts as i32)
        .bind(status_str)
        .bind(metadata.priority)
        .bind(metadata.scheduled_at.unwrap_or(metadata.created_at))
        .bind(metadata.failed_at)
        .bind(&metadata.error_message)
        .bind(metadata.created_at)
        .bind(metadata.updated_at)
        .execute(&self.pool)
        .await?;

        tracing::info!("Job {} pushed to database queue '{}'", metadata.id, metadata.queue_name);
        Ok(())
    }

    async fn pop(&self, queue_name: &str) -> Result<Option<JobMetadata>> {
        // Start a transaction to ensure atomicity
        let mut tx = self.pool.begin().await?;

        // Find the next available job and reserve it
        let job_row = sqlx::query_as::<_, JobRow>(
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
        .bind(queue_name)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(row) = job_row {
            tx.commit().await?;
            let metadata = self.row_to_job_metadata(&row)?;
            tracing::info!("Job {} popped from queue '{}'", metadata.id, queue_name);
            Ok(Some(metadata))
        } else {
            tx.rollback().await?;
            Ok(None)
        }
    }

    async fn size(&self, queue_name: &str) -> Result<u64> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM jobs WHERE queue_name = $1 AND status = 'pending'"
        )
        .bind(queue_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0 as u64)
    }

    async fn delete(&self, job_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM jobs WHERE id = $1")
            .bind(job_id)
            .execute(&self.pool)
            .await?;

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

        sqlx::query(
            r#"
            UPDATE jobs
            SET status = $1,
                attempts = $2,
                failed_at = $3,
                error_message = $4,
                processed_at = CASE WHEN $1 = 'completed' THEN NOW() ELSE processed_at END,
                updated_at = NOW()
            WHERE id = $5
            "#
        )
        .bind(status_str)
        .bind(metadata.attempts as i32)
        .bind(metadata.failed_at)
        .bind(&metadata.error_message)
        .bind(&metadata.id)
        .execute(&self.pool)
        .await?;

        tracing::info!("Job {} updated in database queue", metadata.id);
        Ok(())
    }

    async fn failed_jobs(&self, limit: Option<u32>) -> Result<Vec<JobMetadata>> {
        let limit = limit.unwrap_or(100) as i64;

        let rows = sqlx::query_as::<_, JobRow>(
            r#"
            SELECT * FROM jobs
            WHERE status = 'failed'
            ORDER BY failed_at DESC
            LIMIT $1
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(self.row_to_job_metadata(&row)?);
        }

        Ok(jobs)
    }

    async fn retry_job(&self, job_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET status = 'pending',
                attempts = 0,
                failed_at = NULL,
                error_message = NULL,
                reserved_at = NULL,
                processed_at = NULL,
                available_at = NOW(),
                updated_at = NOW()
            WHERE id = $1 AND status = 'failed'
            "#
        )
        .bind(job_id)
        .execute(&self.pool)
        .await?;

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
        let limit = limit.unwrap_or(100) as i64;

        let status_str = status.map(|s| match s {
            JobStatus::Pending => "pending",
            JobStatus::Processing => "processing",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Retrying => "retrying",
        });

        let rows = sqlx::query_as::<_, JobRow>(
            "SELECT * FROM jobs WHERE queue_name = $1 AND ($2::text IS NULL OR status = $2) ORDER BY created_at DESC LIMIT $3"
        )
        .bind(queue_name)
        .bind(status_str)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(self.row_to_job_metadata(&row)?);
        }

        Ok(jobs)
    }

    /// Clean up old completed jobs
    pub async fn cleanup_completed_jobs(&self, older_than_days: u32) -> Result<u64> {
        let interval = format!("{} days", older_than_days);
        let result = sqlx::query(
            r#"
            DELETE FROM jobs
            WHERE status = 'completed'
              AND processed_at < NOW() - CAST($1 AS INTERVAL)
            "#
        )
        .bind(interval)
        .execute(&self.pool)
        .await?;

        let deleted_count = result.rows_affected();
        tracing::info!("Cleaned up {} completed jobs older than {} days", deleted_count, older_than_days);
        Ok(deleted_count)
    }

    /// Get queue statistics
    pub async fn get_queue_stats(&self, queue_name: &str) -> Result<QueueStats> {
        let stats: (i64, i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status = 'pending') as pending,
                COUNT(*) FILTER (WHERE status = 'processing') as processing,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COUNT(*) FILTER (WHERE status = 'failed') as failed,
                COUNT(*) FILTER (WHERE status = 'retrying') as retrying
            FROM jobs
            WHERE queue_name = $1
            "#
        )
        .bind(queue_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(QueueStats {
            queue_name: queue_name.to_string(),
            pending: stats.0 as u64,
            processing: stats.1 as u64,
            completed: stats.2 as u64,
            failed: stats.3 as u64,
            retrying: stats.4 as u64,
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