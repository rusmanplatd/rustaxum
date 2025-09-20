use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use chrono::{DateTime, Utc};
use crate::app::jobs::{QueueDriver, JobMetadata, JobStatus};

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
    fn row_to_job_metadata(&self, row: &sqlx::postgres::PgRow) -> Result<JobMetadata> {
        let status_str: &str = row.try_get("status")?;
        let status = match status_str {
            "pending" => JobStatus::Pending,
            "processing" => JobStatus::Processing,
            "completed" => JobStatus::Completed,
            "failed" => JobStatus::Failed,
            "retrying" => JobStatus::Retrying,
            _ => JobStatus::Pending,
        };

        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;
        let failed_at: Option<DateTime<Utc>> = row.try_get("failed_at")?;

        Ok(JobMetadata {
            id: row.try_get("id")?,
            job_name: row.try_get("job_name")?,
            queue_name: row.try_get("queue_name")?,
            status,
            payload: row.try_get("payload")?,
            attempts: row.try_get::<i32, _>("attempts")? as u32,
            max_attempts: row.try_get::<i32, _>("max_attempts")? as u32,
            priority: row.try_get("priority")?,
            created_at,
            updated_at,
            scheduled_at: row.try_get("available_at")?,
            failed_at,
            error_message: row.try_get("error_message")?,
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

        sqlx::query!(
            r#"
            INSERT INTO jobs (
                id, queue_name, job_name, payload, attempts, max_attempts,
                status, priority, available_at, failed_at, error_message,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            metadata.id,
            metadata.queue_name,
            metadata.job_name,
            metadata.payload,
            metadata.attempts as i32,
            metadata.max_attempts as i32,
            status_str,
            metadata.priority,
            metadata.scheduled_at.unwrap_or(metadata.created_at),
            metadata.failed_at,
            metadata.error_message,
            metadata.created_at,
            metadata.updated_at
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("Job {} pushed to database queue '{}'", metadata.id, metadata.queue_name);
        Ok(())
    }

    async fn pop(&self, queue_name: &str) -> Result<Option<JobMetadata>> {
        // Start a transaction to ensure atomicity
        let mut tx = self.pool.begin().await?;

        // Find the next available job and reserve it
        let job_row = sqlx::query!(
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
            "#,
            queue_name
        )
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
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM jobs WHERE queue_name = $1 AND status = 'pending'",
            queue_name
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or(0) as u64)
    }

    async fn delete(&self, job_id: &str) -> Result<()> {
        sqlx::query!("DELETE FROM jobs WHERE id = $1", job_id)
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

        sqlx::query!(
            r#"
            UPDATE jobs
            SET status = $1,
                attempts = $2,
                failed_at = $3,
                error_message = $4,
                processed_at = CASE WHEN $1 = 'completed' THEN NOW() ELSE processed_at END,
                updated_at = NOW()
            WHERE id = $5
            "#,
            status_str,
            metadata.attempts as i32,
            metadata.failed_at,
            metadata.error_message,
            metadata.id
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("Job {} updated in database queue", metadata.id);
        Ok(())
    }

    async fn failed_jobs(&self, limit: Option<u32>) -> Result<Vec<JobMetadata>> {
        let limit = limit.unwrap_or(100) as i64;

        let rows = sqlx::query!(
            r#"
            SELECT * FROM jobs
            WHERE status = 'failed'
            ORDER BY failed_at DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(self.row_to_job_metadata(&row)?);
        }

        Ok(jobs)
    }

    async fn retry_job(&self, job_id: &str) -> Result<()> {
        sqlx::query!(
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
            "#,
            job_id
        )
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

        let rows = if let Some(status) = status {
            let status_str = match status {
                JobStatus::Pending => "pending",
                JobStatus::Processing => "processing",
                JobStatus::Completed => "completed",
                JobStatus::Failed => "failed",
                JobStatus::Retrying => "retrying",
            };

            sqlx::query!(
                "SELECT * FROM jobs WHERE queue_name = $1 AND status = $2 ORDER BY created_at DESC LIMIT $3",
                queue_name,
                status_str,
                limit
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query!(
                "SELECT * FROM jobs WHERE queue_name = $1 ORDER BY created_at DESC LIMIT $2",
                queue_name,
                limit
            )
            .fetch_all(&self.pool)
            .await?
        };

        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(self.row_to_job_metadata(&row)?);
        }

        Ok(jobs)
    }

    /// Clean up old completed jobs
    pub async fn cleanup_completed_jobs(&self, older_than_days: u32) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM jobs
            WHERE status = 'completed'
              AND processed_at < NOW() - INTERVAL '%s days'
            "#,
            older_than_days.to_string()
        )
        .execute(&self.pool)
        .await?;

        let deleted_count = result.rows_affected();
        tracing::info!("Cleaned up {} completed jobs older than {} days", deleted_count, older_than_days);
        Ok(deleted_count)
    }

    /// Get queue statistics
    pub async fn get_queue_stats(&self, queue_name: &str) -> Result<QueueStats> {
        let stats = sqlx::query!(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status = 'pending') as pending,
                COUNT(*) FILTER (WHERE status = 'processing') as processing,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COUNT(*) FILTER (WHERE status = 'failed') as failed,
                COUNT(*) FILTER (WHERE status = 'retrying') as retrying
            FROM jobs
            WHERE queue_name = $1
            "#,
            queue_name
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(QueueStats {
            queue_name: queue_name.to_string(),
            pending: stats.pending.unwrap_or(0) as u64,
            processing: stats.processing.unwrap_or(0) as u64,
            completed: stats.completed.unwrap_or(0) as u64,
            failed: stats.failed.unwrap_or(0) as u64,
            retrying: stats.retrying.unwrap_or(0) as u64,
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