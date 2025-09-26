# Job System Documentation

## Overview

The Rustaxum framework includes a comprehensive Laravel-inspired job system built with Rust and Axum. This system provides asynchronous background job processing with database persistence, retry logic, priority queues, and comprehensive error handling.

## Table of Contents

- [Core Components](#core-components)
- [Job Trait](#job-trait)
- [Queue Drivers](#queue-drivers)
- [Job Model](#job-model)
- [Creating Jobs](#creating-jobs)
- [Dispatching Jobs](#dispatching-jobs)
- [Queue Management](#queue-management)
- [Error Handling](#error-handling)
- [Activity Logging](#activity-logging)
- [Database Schema](#database-schema)
- [Configuration](#configuration)
- [Examples](#examples)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Core Components

### Job Trait

All jobs must implement the `Job` trait defined in `src/app/jobs/mod.rs`:

```rust
#[async_trait]
pub trait Job: Send + Sync + std::fmt::Debug {
    /// Get the job name for identification
    fn job_name(&self) -> &'static str;

    /// Execute the job
    async fn handle(&self) -> Result<()>;

    /// Get the maximum number of retry attempts (default: 3)
    fn max_attempts(&self) -> u32 { 3 }

    /// Get the delay between retries in seconds (default: 60)
    fn retry_delay(&self) -> u64 { 60 }

    /// Determine if this job should be queued (default: true)
    fn should_queue(&self) -> bool { true }

    /// Get the queue name for this job (default: "default")
    fn queue_name(&self) -> &str { "default" }

    /// Get job priority - lower numbers = higher priority (default: 0)
    fn priority(&self) -> i32 { 0 }

    /// Get job timeout in seconds (default: 300 - 5 minutes)
    fn timeout(&self) -> Option<u64> { Some(300) }

    /// Serialize job data for queue storage
    fn serialize(&self) -> Result<String>;

    /// Called when job fails after all retries
    async fn failed(&self, error: &anyhow::Error);
}
```

### Job Status

Jobs can have the following statuses:

```rust
pub enum JobStatus {
    Pending,    // Waiting to be processed
    Processing, // Currently being executed
    Completed,  // Successfully finished
    Failed,     // Failed permanently (exceeded max attempts)
    Retrying,   // Failed but will retry
}
```

## Queue Drivers

### Memory Driver

For development and testing:

```rust
use crate::app::jobs::MemoryQueueDriver;

let driver = Box::new(MemoryQueueDriver::new());
```

### Database Driver

For production with PostgreSQL persistence:

```rust
use crate::app::jobs::database_queue_driver::DatabaseQueueDriver;

let driver = Box::new(DatabaseQueueDriver::new(pool.clone()));
```

## Job Model

The `Job` model (`src/app/models/jobs.rs`) provides database persistence:

```rust
pub struct Job {
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## Creating Jobs

### Using Artisan Command

Create a new asynchronous job:

```bash
cargo run --bin artisan -- make job ProcessPaymentJob
```

Create a synchronous job:

```bash
cargo run --bin artisan -- make job ProcessFileJob --sync
```

### Manual Job Creation

1. Create a new file in `src/app/jobs/`:

```rust
// src/app/jobs/process_payment_job.rs
use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::app::jobs::Job;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessPaymentJob {
    pub payment_id: String,
    pub amount: f64,
    pub currency: String,
    pub customer_id: String,
}

impl ProcessPaymentJob {
    pub fn new(payment_id: String, amount: f64, currency: String, customer_id: String) -> Self {
        Self {
            payment_id,
            amount,
            currency,
            customer_id,
        }
    }
}

#[async_trait]
impl Job for ProcessPaymentJob {
    fn job_name(&self) -> &'static str {
        "ProcessPaymentJob"
    }

    async fn handle(&self) -> Result<()> {
        tracing::info!("Processing payment {} for ${}", self.payment_id, self.amount);

        // Your payment processing logic here
        self.process_payment().await?;

        tracing::info!("Payment {} processed successfully", self.payment_id);
        Ok(())
    }

    fn max_attempts(&self) -> u32 {
        5 // Payment jobs should retry more often
    }

    fn retry_delay(&self) -> u64 {
        120 // 2 minutes between retries
    }

    fn queue_name(&self) -> &str {
        "payments"
    }

    fn priority(&self) -> i32 {
        -5 // High priority for payments
    }

    fn timeout(&self) -> Option<u64> {
        Some(600) // 10 minutes timeout
    }

    fn serialize(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    async fn failed(&self, error: &anyhow::Error) {
        tracing::error!(
            "Payment job {} failed permanently: {}",
            self.payment_id,
            error
        );

        // Handle payment failure (refund, notify customer, etc.)
        self.handle_payment_failure(error).await;
    }
}

impl ProcessPaymentJob {
    async fn process_payment(&self) -> Result<()> {
        // Implement payment processing logic
        Ok(())
    }

    async fn handle_payment_failure(&self, _error: &anyhow::Error) {
        // Handle payment failure
    }
}
```

2. Add the module to `src/app/jobs/mod.rs`:

```rust
pub mod process_payment_job;
```

## Dispatching Jobs

### Using Global Dispatcher

```rust
use crate::app::jobs::{dispatch_job, ProcessPaymentJob};

// Create job instance
let payment_job = ProcessPaymentJob::new(
    "payment_123".to_string(),
    99.99,
    "USD".to_string(),
    "customer_456".to_string(),
);

// Dispatch the job
let job_id = dispatch_job(&payment_job).await?;
tracing::info!("Payment job dispatched with ID: {}", job_id);
```

### Using Dispatch Macro

```rust
use crate::dispatch;

let payment_job = ProcessPaymentJob::new(/* ... */);
let job_id = dispatch!(payment_job)?;
```

### Manual Dispatcher Usage

```rust
use crate::app::jobs::{JobDispatcher, MemoryQueueDriver};

// Create dispatcher with specific driver
let driver = Box::new(MemoryQueueDriver::new());
let dispatcher = JobDispatcher::new(driver);

// Dispatch job
let job_id = dispatcher.dispatch(&payment_job).await?;
```

## Queue Management

### Starting Workers

```rust
use crate::app::jobs::{job_dispatcher, JobDispatcher};

// Get global dispatcher
let dispatcher = job_dispatcher().await;
let mut dispatcher = dispatcher.write().await;

// Start workers for different queues
dispatcher.start_worker("default".to_string(), 5).await?;    // 5 concurrent workers
dispatcher.start_worker("payments".to_string(), 10).await?;  // 10 concurrent workers
dispatcher.start_worker("emails".to_string(), 3).await?;     // 3 concurrent workers
```

### Queue Statistics

```rust
let stats = dispatcher.stats("payments").await?;
println!("Queue: {}", stats.queue_name);
println!("Pending jobs: {}", stats.pending_jobs);
println!("Failed jobs: {}", stats.failed_jobs);
println!("Workers: {}", stats.workers_count);
```

### Stopping Workers

```rust
dispatcher.stop_worker("payments").await?;
```

## Error Handling

### Retry Logic

Jobs automatically retry based on configuration:

```rust
impl Job for MyJob {
    fn max_attempts(&self) -> u32 {
        5 // Retry up to 5 times
    }

    fn retry_delay(&self) -> u64 {
        300 // Wait 5 minutes between retries
    }
}
```

### Custom Failure Handling

```rust
async fn failed(&self, error: &anyhow::Error) {
    // Log to external monitoring
    tracing::error!(
        target: "job_failures",
        job_type = self.job_name(),
        error = %error,
        "Job failed permanently"
    );

    // Send admin notification
    self.notify_administrators(error).await;

    // Store failure audit
    self.store_failure_audit(error).await;
}
```

## Activity Logging

### Using ActivityLoggedJob Trait

```rust
use crate::app::jobs::ActivityLoggedJob;
use crate::app::traits::ServiceActivityLogger;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyJob {
    pub user_id: Option<String>,
    // ... other fields
}

impl ServiceActivityLogger for MyJob {
    // Implement activity logger methods
}

impl ActivityLoggedJob for MyJob {
    fn triggered_by(&self) -> Option<&str> {
        self.user_id.as_deref()
    }

    fn job_properties(&self) -> serde_json::Value {
        json!({
            "job_name": self.job_name(),
            "custom_property": "value"
        })
    }
}

// Execute with automatic logging
job.handle_with_logging().await?;
```

## Database Schema

The jobs table includes the following structure:

```sql
CREATE TABLE jobs (
    id CHAR(26) PRIMARY KEY,
    queue_name VARCHAR(255) NOT NULL DEFAULT 'default',
    job_name VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    priority INTEGER NOT NULL DEFAULT 0,
    available_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reserved_at TIMESTAMPTZ,
    processed_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    error_message TEXT,
    timeout_seconds INTEGER DEFAULT 300,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Optimized Indexes

- `idx_jobs_queue_processing` - For queue processing performance
- `idx_jobs_failed` - For failed job queries
- `idx_jobs_retry` - For retry logic
- Individual indexes on key columns

## Configuration

### Environment Variables

```env
# Database connection for job storage
DATABASE_URL=postgresql://user:password@localhost/database

# Redis connection for queue driver (if using Redis)
REDIS_URL=redis://localhost:6379

# Job worker settings
JOB_WORKER_CONCURRENCY=5
JOB_MAX_RETRY_ATTEMPTS=3
JOB_RETRY_DELAY=60
JOB_DEFAULT_TIMEOUT=300
```

### Application Configuration

```rust
// src/config/mod.rs
pub struct JobConfig {
    pub default_queue: String,
    pub max_attempts: u32,
    pub retry_delay: u64,
    pub timeout: u64,
    pub worker_concurrency: usize,
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            default_queue: "default".to_string(),
            max_attempts: 3,
            retry_delay: 60,
            timeout: 300,
            worker_concurrency: 5,
        }
    }
}
```

## Examples

### Email Job

```rust
use crate::app::jobs::SendEmailJob;

// Welcome email
let welcome_job = SendEmailJob::welcome(
    "user@example.com".to_string(),
    "John Doe".to_string(),
    Some("https://app.com/activate".to_string())
);

dispatch!(welcome_job)?;

// Password reset email
let reset_job = SendEmailJob::password_reset(
    "user@example.com".to_string(),
    "John Doe".to_string(),
    "reset_token_123".to_string()
);

dispatch!(reset_job)?;
```

### File Processing Job

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessFileJob {
    pub file_path: String,
    pub user_id: String,
    pub processing_type: FileProcessingType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileProcessingType {
    ImageResize { width: u32, height: u32 },
    DocumentConvert { target_format: String },
    DataImport { table_name: String },
}

#[async_trait]
impl Job for ProcessFileJob {
    fn job_name(&self) -> &'static str {
        "ProcessFileJob"
    }

    async fn handle(&self) -> Result<()> {
        match &self.processing_type {
            FileProcessingType::ImageResize { width, height } => {
                self.resize_image(*width, *height).await?;
            }
            FileProcessingType::DocumentConvert { target_format } => {
                self.convert_document(target_format).await?;
            }
            FileProcessingType::DataImport { table_name } => {
                self.import_data(table_name).await?;
            }
        }
        Ok(())
    }

    fn queue_name(&self) -> &str {
        "file-processing"
    }

    fn timeout(&self) -> Option<u64> {
        Some(1800) // 30 minutes for file processing
    }

    fn serialize(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}
```

### Batch Job Processing

```rust
pub struct BatchProcessJob {
    pub batch_id: String,
    pub item_ids: Vec<String>,
    pub batch_size: usize,
}

#[async_trait]
impl Job for BatchProcessJob {
    async fn handle(&self) -> Result<()> {
        // Process items in chunks
        for chunk in self.item_ids.chunks(self.batch_size) {
            self.process_chunk(chunk).await?;

            // Small delay between chunks to avoid overwhelming the system
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        Ok(())
    }

    fn queue_name(&self) -> &str {
        "batch-processing"
    }

    fn priority(&self) -> i32 {
        5 // Lower priority for batch jobs
    }
}
```

## Best Practices

### 1. Job Design

- Keep jobs small and focused on a single responsibility
- Make jobs idempotent (safe to run multiple times)
- Avoid long-running operations (use timeouts)
- Store minimal data in job payload

### 2. Error Handling

- Implement comprehensive error handling in `handle()` method
- Use appropriate retry strategies for different error types
- Log failures with sufficient context
- Implement fallback mechanisms for critical operations

### 3. Performance

- Use appropriate queue names to separate different types of work
- Set realistic timeouts based on job complexity
- Monitor queue sizes and processing times
- Scale workers based on load

### 4. Monitoring

- Log job start, completion, and failure events
- Track job processing metrics
- Set up alerts for failed jobs
- Monitor queue depths and processing delays

### 5. Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::jobs::MemoryQueueDriver;

    #[tokio::test]
    async fn test_payment_job_success() {
        let job = ProcessPaymentJob::new(
            "test_payment".to_string(),
            100.0,
            "USD".to_string(),
            "test_customer".to_string(),
        );

        // Test job execution
        assert!(job.handle().await.is_ok());
    }

    #[tokio::test]
    async fn test_job_serialization() {
        let job = ProcessPaymentJob::new(/* ... */);
        let serialized = job.serialize().unwrap();

        // Verify serialization works
        assert!(!serialized.is_empty());
    }
}
```

## Troubleshooting

### Common Issues

1. **Jobs not processing**

   - Check if workers are started
   - Verify queue names match
   - Check database connectivity
   - Review job status in database

2. **Jobs failing repeatedly**

   - Check error messages in job records
   - Verify job dependencies (database, external APIs)
   - Review timeout settings
   - Check retry configuration

3. **Performance issues**
   - Monitor queue sizes
   - Check worker concurrency settings
   - Review database indexes
   - Optimize job processing logic

### Debugging

Enable debug logging:

```rust
tracing::debug!("Job {} starting with data: {:?}", self.job_name(), self);
```

Query job status:

```sql
SELECT job_name, status, attempts, error_message, created_at
FROM jobs
WHERE status = 'failed'
ORDER BY created_at DESC
LIMIT 10;
```

### Monitoring Commands

```bash
# Check queue status
cargo run --bin artisan -- queue:status

# Retry failed jobs
cargo run --bin artisan -- queue:retry-failed

# Clear failed jobs
cargo run --bin artisan -- queue:clear-failed

# List active workers
cargo run --bin artisan -- queue:workers
```

## Advanced Features

### Delayed Jobs

```rust
impl Job for DelayedJob {
    fn available_at(&self) -> DateTime<Utc> {
        Utc::now() + chrono::Duration::minutes(30) // Run in 30 minutes
    }
}
```

### Job Chaining

```rust
impl Job for FirstJob {
    async fn handle(&self) -> Result<()> {
        // Do first job work

        // Chain next job
        let next_job = SecondJob::new(self.data.clone());
        dispatch!(next_job)?;

        Ok(())
    }
}
```

### Conditional Job Execution

```rust
impl Job for ConditionalJob {
    fn should_queue(&self) -> bool {
        // Only queue during business hours
        let now = chrono::Local::now();
        now.hour() >= 9 && now.hour() <= 17
    }
}
```

This comprehensive job system provides all the features needed for robust background processing in a production environment, following Laravel conventions while leveraging Rust's performance and safety features.
