use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, sleep};
use tracing::{info, warn, error};

use crate::app::jobs::{QueueDriver, JobMetadata, JobStatus, Job};

/// Worker configuration
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub queue_name: String,
    pub concurrency: usize,
    pub max_runtime: Duration,
    pub sleep_duration: Duration,
    pub retry_delay: Duration,
    pub max_memory_usage: Option<u64>, // In bytes
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            queue_name: "default".to_string(),
            concurrency: 1,
            max_runtime: Duration::from_secs(300), // 5 minutes
            sleep_duration: Duration::from_secs(1),
            retry_delay: Duration::from_secs(60),
            max_memory_usage: Some(500 * 1024 * 1024), // 500MB
        }
    }
}

/// Queue worker that processes jobs from a queue
pub struct QueueWorker {
    config: WorkerConfig,
    driver: Arc<dyn QueueDriver>,
    job_registry: Arc<RwLock<HashMap<String, Box<dyn JobFactory>>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    is_running: Arc<RwLock<bool>>,
    stats: Arc<RwLock<WorkerStats>>,
}

/// Factory trait for creating job instances from metadata
pub trait JobFactory: Send + Sync {
    fn create_job(&self, payload: &str) -> Result<Box<dyn Job>>;
}

/// Worker statistics
#[derive(Debug, Clone, Default)]
pub struct WorkerStats {
    pub jobs_processed: u64,
    pub jobs_succeeded: u64,
    pub jobs_failed: u64,
    pub jobs_retried: u64,
    pub total_processing_time: Duration,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_job_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl QueueWorker {
    pub fn new(config: WorkerConfig, driver: Arc<dyn QueueDriver>) -> Self {
        Self {
            config,
            driver,
            job_registry: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx: None,
            is_running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(WorkerStats::default())),
        }
    }

    /// Register a job factory for a specific job type
    pub async fn register_job<T: Job + 'static>(&self, job_name: &str, factory: impl JobFactory + 'static) {
        let mut registry = self.job_registry.write().await;
        registry.insert(job_name.to_string(), Box::new(factory));
        info!("Registered job factory for: {}", job_name);
    }

    /// Start the worker
    pub async fn start(&mut self) -> Result<()> {
        if *self.is_running.read().await {
            return Err(anyhow::anyhow!("Worker is already running"));
        }

        *self.is_running.write().await = true;
        *self.stats.write().await = WorkerStats {
            started_at: Some(chrono::Utc::now()),
            ..Default::default()
        };

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        info!("Starting queue worker for '{}' with {} concurrent workers",
            self.config.queue_name, self.config.concurrency);

        // Spawn worker tasks
        let mut handles = Vec::new();
        for worker_id in 0..self.config.concurrency {
            let handle = self.spawn_worker_task(worker_id).await;
            handles.push(handle);
        }

        // Spawn stats reporting task
        let stats_handle = self.spawn_stats_task().await;
        handles.push(stats_handle);

        // Wait for shutdown signal or all workers to complete
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received for worker '{}'", self.config.queue_name);
            }
            _ = futures::future::join_all(handles) => {
                warn!("All worker tasks completed unexpectedly");
            }
        }

        *self.is_running.write().await = false;
        info!("Queue worker '{}' stopped", self.config.queue_name);
        Ok(())
    }

    /// Stop the worker
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        // Wait for worker to stop
        while *self.is_running.read().await {
            sleep(Duration::from_millis(100)).await;
        }

        info!("Queue worker '{}' stopped gracefully", self.config.queue_name);
        Ok(())
    }

    /// Get worker statistics
    pub async fn get_stats(&self) -> WorkerStats {
        self.stats.read().await.clone()
    }

    /// Check if worker is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    async fn spawn_worker_task(&self, worker_id: usize) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let driver = self.driver.clone();
        let job_registry = self.job_registry.clone();
        let is_running = self.is_running.clone();
        let stats = self.stats.clone();

        tokio::spawn(async move {
            info!("Worker {}-{} started", config.queue_name, worker_id);

            while *is_running.read().await {
                match Self::process_next_job(&config, &driver, &job_registry, &stats).await {
                    Ok(processed) => {
                        if !processed {
                            // No job available, sleep before checking again
                            sleep(config.sleep_duration).await;
                        }
                    }
                    Err(e) => {
                        error!("Worker {}-{} error: {}", config.queue_name, worker_id, e);
                        sleep(config.sleep_duration).await;
                    }
                }
            }

            info!("Worker {}-{} stopped", config.queue_name, worker_id);
        })
    }

    async fn spawn_stats_task(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let is_running = self.is_running.clone();
        let stats = self.stats.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Report every minute

            while *is_running.read().await {
                interval.tick().await;

                let current_stats = stats.read().await.clone();
                info!(
                    "Worker '{}' stats: processed={}, succeeded={}, failed={}, retried={}",
                    config.queue_name,
                    current_stats.jobs_processed,
                    current_stats.jobs_succeeded,
                    current_stats.jobs_failed,
                    current_stats.jobs_retried
                );
            }
        })
    }

    async fn process_next_job(
        config: &WorkerConfig,
        driver: &Arc<dyn QueueDriver>,
        job_registry: &Arc<RwLock<HashMap<String, Box<dyn JobFactory>>>>,
        stats: &Arc<RwLock<WorkerStats>>,
    ) -> Result<bool> {
        // Try to get next job from queue
        let mut job_metadata = match driver.pop(&config.queue_name).await? {
            Some(metadata) => metadata,
            None => return Ok(false), // No jobs available
        };

        let start_time = std::time::Instant::now();

        // Update stats
        {
            let mut stats_guard = stats.write().await;
            stats_guard.jobs_processed += 1;
            stats_guard.last_job_at = Some(chrono::Utc::now());
        }

        info!("Processing job {} ({})", job_metadata.id, job_metadata.job_name);

        // Create job instance from metadata
        let job_result = {
            let registry = job_registry.read().await;
            match registry.get(&job_metadata.job_name) {
                Some(factory) => factory.create_job(&job_metadata.payload),
                None => Err(anyhow::anyhow!("No factory registered for job type: {}", job_metadata.job_name)),
            }
        };

        let job = match job_result {
            Ok(job) => job,
            Err(e) => {
                error!("Failed to create job {}: {}", job_metadata.id, e);
                job_metadata.mark_failed(&e.to_string());
                driver.update(&job_metadata).await?;
                stats.write().await.jobs_failed += 1;
                return Ok(true);
            }
        };

        // Execute the job with timeout
        let execution_result = tokio::time::timeout(
            config.max_runtime,
            job.handle()
        ).await;

        let processing_time = start_time.elapsed();

        match execution_result {
            Ok(Ok(())) => {
                // Job succeeded
                job_metadata.mark_completed();
                driver.update(&job_metadata).await?;
                stats.write().await.jobs_succeeded += 1;
                info!("Job {} completed successfully in {:?}", job_metadata.id, processing_time);
            }
            Ok(Err(e)) => {
                // Job failed
                error!("Job {} failed: {}", job_metadata.id, e);

                if job_metadata.can_retry() {
                    // Retry the job
                    job_metadata.mark_retrying();
                    job_metadata.scheduled_at = Some(chrono::Utc::now() + chrono::Duration::seconds(config.retry_delay.as_secs() as i64));
                    driver.update(&job_metadata).await?;
                    driver.push(job_metadata).await?; // Re-queue for retry
                    stats.write().await.jobs_retried += 1;
                    info!("Job {} will be retried (attempt {}/{})", job_metadata.id, job_metadata.attempts, job_metadata.max_attempts);
                } else {
                    // Max retries exceeded
                    job_metadata.mark_failed(&e.to_string());
                    driver.update(&job_metadata).await?;
                    job.failed(&e).await;
                    stats.write().await.jobs_failed += 1;
                    error!("Job {} failed permanently after {} attempts", job_metadata.id, job_metadata.attempts);
                }
            }
            Err(_) => {
                // Job timed out
                let timeout_error = format!("Job timed out after {:?}", config.max_runtime);
                error!("Job {} {}", job_metadata.id, timeout_error);

                if job_metadata.can_retry() {
                    job_metadata.mark_retrying();
                    job_metadata.scheduled_at = Some(chrono::Utc::now() + chrono::Duration::seconds(config.retry_delay.as_secs() as i64));
                    driver.update(&job_metadata).await?;
                    driver.push(job_metadata).await?;
                    stats.write().await.jobs_retried += 1;
                } else {
                    job_metadata.mark_failed(&timeout_error);
                    driver.update(&job_metadata).await?;
                    stats.write().await.jobs_failed += 1;
                }
            }
        }

        // Update total processing time
        stats.write().await.total_processing_time += processing_time;

        Ok(true)
    }
}

/// Simple job factory implementation
pub struct SimpleJobFactory<T: Job + Clone + 'static> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Job + Clone + 'static> SimpleJobFactory<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Job + Clone + serde::de::DeserializeOwned + 'static> JobFactory for SimpleJobFactory<T> {
    fn create_job(&self, payload: &str) -> Result<Box<dyn Job>> {
        let job: T = serde_json::from_str(payload)?;
        Ok(Box::new(job))
    }
}