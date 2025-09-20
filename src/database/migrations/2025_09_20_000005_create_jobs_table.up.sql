-- Create jobs table for queue management
CREATE TABLE IF NOT EXISTS jobs (
    id CHAR(26) PRIMARY KEY DEFAULT gen_random_uuid()::text,
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

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_jobs_queue_name ON jobs(queue_name);
CREATE INDEX IF NOT EXISTS idx_jobs_job_name ON jobs(job_name);
CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_priority ON jobs(priority);
CREATE INDEX IF NOT EXISTS idx_jobs_available_at ON jobs(available_at);
CREATE INDEX IF NOT EXISTS idx_jobs_reserved_at ON jobs(reserved_at);
CREATE INDEX IF NOT EXISTS idx_jobs_processed_at ON jobs(processed_at);
CREATE INDEX IF NOT EXISTS idx_jobs_failed_at ON jobs(failed_at);
CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs(created_at);

-- Create composite index for queue processing (most important)
CREATE INDEX IF NOT EXISTS idx_jobs_queue_processing ON jobs(queue_name, status, priority, available_at)
WHERE status = 'pending';

-- Create composite index for failed jobs
CREATE INDEX IF NOT EXISTS idx_jobs_failed ON jobs(failed_at, attempts, max_attempts)
WHERE status = 'failed';

-- Create composite index for retry logic
CREATE INDEX IF NOT EXISTS idx_jobs_retry ON jobs(queue_name, status, available_at, attempts, max_attempts)
WHERE status = 'retrying';

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add trigger to update updated_at timestamp
CREATE TRIGGER update_jobs_updated_at
    BEFORE UPDATE ON jobs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Add check constraints
ALTER TABLE jobs ADD CONSTRAINT chk_jobs_status
CHECK (status IN ('pending', 'processing', 'completed', 'failed', 'retrying'));

ALTER TABLE jobs ADD CONSTRAINT chk_jobs_attempts
CHECK (attempts >= 0 AND attempts <= max_attempts);

ALTER TABLE jobs ADD CONSTRAINT chk_jobs_priority
CHECK (priority >= -100 AND priority <= 100);