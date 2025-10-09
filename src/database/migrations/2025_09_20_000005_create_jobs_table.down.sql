-- Drop the jobs table and related objects
DROP TRIGGER IF EXISTS update_jobs_updated_at ON jobs;
DROP INDEX IF EXISTS idx_jobs_retry;
DROP INDEX IF EXISTS idx_jobs_failed;
DROP INDEX IF EXISTS idx_jobs_queue_processing;
DROP INDEX IF EXISTS idx_jobs_created_at;
DROP INDEX IF EXISTS idx_jobs_failed_at;
DROP INDEX IF EXISTS idx_jobs_processed_at;
DROP INDEX IF EXISTS idx_jobs_reserved_at;
DROP INDEX IF EXISTS idx_jobs_available_at;
DROP INDEX IF EXISTS idx_jobs_priority;
DROP INDEX IF EXISTS idx_jobs_status;
DROP INDEX IF EXISTS idx_jobs_job_name;
DROP INDEX IF EXISTS idx_jobs_queue_name;
DROP TABLE jobs;