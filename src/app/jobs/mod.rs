pub mod process_payment_job;
pub mod send_email_job;
pub mod database_queue_driver;
pub mod queue_worker;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use chrono::{DateTime, Utc};

/// Base trait that all jobs must implement
#[async_trait]
pub trait Job: Send + Sync + std::fmt::Debug {
    /// Get the job name for identification
    fn job_name(&self) -> &'static str;

    /// Execute the job
    async fn handle(&self) -> Result<()>;

    /// Get the maximum number of retry attempts
    fn max_attempts(&self) -> u32 {
        3
    }

    /// Get the delay between retries in seconds
    fn retry_delay(&self) -> u64 {
        60
    }

    /// Determine if this job should be queued
    fn should_queue(&self) -> bool {
        true
    }

    /// Get the queue name for this job
    fn queue_name(&self) -> &str {
        "default"
    }

    /// Get job priority (lower numbers = higher priority)
    fn priority(&self) -> i32 {
        0
    }

    /// Get job timeout in seconds
    fn timeout(&self) -> Option<u64> {
        Some(300) // 5 minutes default
    }

    /// Serialize job data for queue storage
    fn serialize(&self) -> Result<String>;

    /// Called when job fails after all retries
    async fn failed(&self, error: &anyhow::Error) {
        tracing::error!("Job {} failed permanently: {}", self.job_name(), error);
    }
}

/// Job status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Retrying,
}

/// Job metadata for tracking and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    pub id: String,
    pub job_name: String,
    pub queue_name: String,
    pub status: JobStatus,
    pub payload: String,
    pub attempts: u32,
    pub max_attempts: u32,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

impl JobMetadata {
    pub fn new(job_name: String, queue_name: String, payload: String, priority: i32, max_attempts: u32) -> Self {
        let now = Utc::now();
        Self {
            id: ulid::Ulid::new().to_string(),
            job_name,
            queue_name,
            status: JobStatus::Pending,
            payload,
            attempts: 0,
            max_attempts,
            priority,
            created_at: now,
            updated_at: now,
            scheduled_at: None,
            failed_at: None,
            error_message: None,
        }
    }

    pub fn mark_processing(&mut self) {
        self.status = JobStatus::Processing;
        self.updated_at = Utc::now();
    }

    pub fn mark_completed(&mut self) {
        self.status = JobStatus::Completed;
        self.updated_at = Utc::now();
    }

    pub fn mark_failed(&mut self, error: &str) {
        self.status = JobStatus::Failed;
        self.error_message = Some(error.to_string());
        self.failed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn mark_retrying(&mut self) {
        self.status = JobStatus::Retrying;
        self.attempts += 1;
        self.updated_at = Utc::now();
    }

    pub fn can_retry(&self) -> bool {
        self.attempts < self.max_attempts
    }
}

/// Queue driver trait for different queue implementations
#[async_trait]
pub trait QueueDriver: Send + Sync {
    /// Push a job to the queue
    async fn push(&self, metadata: JobMetadata) -> Result<()>;

    /// Pop a job from the queue
    async fn pop(&self, queue_name: &str) -> Result<Option<JobMetadata>>;

    /// Get queue size
    async fn size(&self, queue_name: &str) -> Result<u64>;

    /// Delete a job from the queue
    async fn delete(&self, job_id: &str) -> Result<()>;

    /// Update job metadata
    async fn update(&self, metadata: &JobMetadata) -> Result<()>;

    /// Get failed jobs
    async fn failed_jobs(&self, limit: Option<u32>) -> Result<Vec<JobMetadata>>;

    /// Retry a failed job
    async fn retry_job(&self, job_id: &str) -> Result<()>;

    /// Get driver name
    fn driver_name(&self) -> &'static str;
}

/// In-memory queue driver for development and testing
#[derive(Debug)]
pub struct MemoryQueueDriver {
    queues: Arc<RwLock<HashMap<String, Vec<JobMetadata>>>>,
    failed_jobs: Arc<RwLock<Vec<JobMetadata>>>,
}

impl MemoryQueueDriver {
    pub fn new() -> Self {
        Self {
            queues: Arc::new(RwLock::new(HashMap::new())),
            failed_jobs: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Default for MemoryQueueDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl QueueDriver for MemoryQueueDriver {
    async fn push(&self, metadata: JobMetadata) -> Result<()> {
        let mut queues = self.queues.write().await;
        let queue = queues.entry(metadata.queue_name.clone()).or_insert_with(Vec::new);

        // Insert in priority order (lower priority number = higher priority)
        let insert_pos = queue.iter().position(|job| job.priority > metadata.priority)
            .unwrap_or(queue.len());

        queue.insert(insert_pos, metadata);
        Ok(())
    }

    async fn pop(&self, queue_name: &str) -> Result<Option<JobMetadata>> {
        let mut queues = self.queues.write().await;
        let queue = queues.entry(queue_name.to_string()).or_insert_with(Vec::new);

        // Pop the highest priority job (first in the list)
        Ok(if queue.is_empty() { None } else { Some(queue.remove(0)) })
    }

    async fn size(&self, queue_name: &str) -> Result<u64> {
        let queues = self.queues.read().await;
        Ok(queues.get(queue_name).map(|q| q.len() as u64).unwrap_or(0))
    }

    async fn delete(&self, job_id: &str) -> Result<()> {
        let mut queues = self.queues.write().await;
        for queue in queues.values_mut() {
            queue.retain(|job| job.id != job_id);
        }
        Ok(())
    }

    async fn update(&self, metadata: &JobMetadata) -> Result<()> {
        if metadata.status == JobStatus::Failed {
            let mut failed_jobs = self.failed_jobs.write().await;
            failed_jobs.push(metadata.clone());
        }
        Ok(())
    }

    async fn failed_jobs(&self, limit: Option<u32>) -> Result<Vec<JobMetadata>> {
        let failed_jobs = self.failed_jobs.read().await;
        let limit = limit.unwrap_or(100) as usize;
        Ok(failed_jobs.iter().take(limit).cloned().collect())
    }

    async fn retry_job(&self, job_id: &str) -> Result<()> {
        let mut failed_jobs = self.failed_jobs.write().await;
        if let Some(pos) = failed_jobs.iter().position(|job| job.id == job_id) {
            let mut job = failed_jobs.remove(pos);
            job.status = JobStatus::Pending;
            job.attempts = 0;
            job.error_message = None;
            job.failed_at = None;
            job.updated_at = Utc::now();

            // Re-queue the job
            drop(failed_jobs);
            self.push(job).await?;
        }
        Ok(())
    }

    fn driver_name(&self) -> &'static str {
        "memory"
    }
}

/// Job dispatcher for managing job execution
pub struct JobDispatcher {
    driver: Box<dyn QueueDriver>,
    workers: HashMap<String, QueueWorker>,
}

impl JobDispatcher {
    pub fn new(driver: Box<dyn QueueDriver>) -> Self {
        Self {
            driver,
            workers: HashMap::new(),
        }
    }

    /// Dispatch a job to the queue
    pub async fn dispatch(&self, job: &dyn Job) -> Result<String> {
        let payload = job.serialize()?;
        let metadata = JobMetadata::new(
            job.job_name().to_string(),
            job.queue_name().to_string(),
            payload,
            job.priority(),
            job.max_attempts(),
        );

        let job_id = metadata.id.clone();
        self.driver.push(metadata).await?;

        tracing::info!("Job {} dispatched to queue '{}'", job_id, job.queue_name());
        Ok(job_id)
    }

    /// Start a worker for a specific queue
    pub async fn start_worker(&mut self, queue_name: String, concurrency: usize) -> Result<()> {
        let worker = QueueWorker::new(queue_name.clone(), concurrency, self.driver.as_ref()).await?;
        self.workers.insert(queue_name.clone(), worker);

        tracing::info!("Started worker for queue '{}' with concurrency {}", queue_name, concurrency);
        Ok(())
    }

    /// Stop a worker
    pub async fn stop_worker(&mut self, queue_name: &str) -> Result<()> {
        if let Some(worker) = self.workers.remove(queue_name) {
            worker.stop().await?;
            tracing::info!("Stopped worker for queue '{}'", queue_name);
        }
        Ok(())
    }

    /// Get queue statistics
    pub async fn stats(&self, queue_name: &str) -> Result<QueueStats> {
        let size = self.driver.size(queue_name).await?;
        let failed_count = self.driver.failed_jobs(None).await?.len() as u64;

        Ok(QueueStats {
            queue_name: queue_name.to_string(),
            pending_jobs: size,
            failed_jobs: failed_count,
            workers_count: self.workers.get(queue_name).map(|w| w.concurrency).unwrap_or(0),
        })
    }
}

/// Queue worker for processing jobs
pub struct QueueWorker {
    queue_name: String,
    concurrency: usize,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl QueueWorker {
    pub async fn new(queue_name: String, concurrency: usize, _driver: &dyn QueueDriver) -> Result<Self> {
        Ok(Self {
            queue_name,
            concurrency,
            shutdown_tx: None,
        })
    }

    pub async fn stop(mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        Ok(())
    }
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub queue_name: String,
    pub pending_jobs: u64,
    pub failed_jobs: u64,
    pub workers_count: usize,
}

/// Global job dispatcher instance
static JOB_DISPATCHER: tokio::sync::OnceCell<Arc<RwLock<JobDispatcher>>> = tokio::sync::OnceCell::const_new();

/// Initialize the global job dispatcher
pub async fn init_job_dispatcher(driver: Box<dyn QueueDriver>) -> Arc<RwLock<JobDispatcher>> {
    JOB_DISPATCHER.get_or_init(|| async {
        Arc::new(RwLock::new(JobDispatcher::new(driver)))
    }).await.clone()
}

/// Get the global job dispatcher
pub async fn job_dispatcher() -> Arc<RwLock<JobDispatcher>> {
    JOB_DISPATCHER.get_or_init(|| async {
        Arc::new(RwLock::new(JobDispatcher::new(Box::new(MemoryQueueDriver::new()))))
    }).await.clone()
}

/// Dispatch a job using the global dispatcher
pub async fn dispatch_job(job: &dyn Job) -> Result<String> {
    let dispatcher = job_dispatcher().await;
    let dispatcher = dispatcher.read().await;
    dispatcher.dispatch(job).await
}

/// Helper macro to dispatch jobs
#[macro_export]
macro_rules! dispatch {
    ($job:expr_2021) => {
        $crate::app::jobs::dispatch_job(&$job).await
    };
}