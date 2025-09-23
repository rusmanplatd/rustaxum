CREATE TABLE activity_log (
    id CHAR(26) PRIMARY KEY,
    correlation_id VARCHAR(255),
    log_name VARCHAR(255),
    description TEXT NOT NULL,
    subject_type VARCHAR(255),
    subject_id VARCHAR(255),
    causer_type VARCHAR(255),
    causer_id VARCHAR(255),
    properties JSONB,
    batch_uuid VARCHAR(255),
    event VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better query performance
CREATE INDEX idx_activity_log_log_name ON activity_log(log_name);
CREATE INDEX idx_activity_log_subject ON activity_log(subject_type, subject_id);
CREATE INDEX idx_activity_log_causer ON activity_log(causer_type, causer_id);
CREATE INDEX idx_activity_log_correlation_id ON activity_log(correlation_id);
CREATE INDEX idx_activity_log_batch_uuid ON activity_log(batch_uuid);
CREATE INDEX idx_activity_log_event ON activity_log(event);
CREATE INDEX idx_activity_log_created_at ON activity_log(created_at);

-- Create composite indexes for common query patterns
CREATE INDEX idx_activity_log_subject_created ON activity_log(subject_type, subject_id, created_at DESC);
CREATE INDEX idx_activity_log_causer_created ON activity_log(causer_type, causer_id, created_at DESC);
CREATE INDEX idx_activity_log_correlation_created ON activity_log(correlation_id, created_at DESC);

-- Create a function to automatically update the updated_at column
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically update updated_at
CREATE TRIGGER update_activity_log_updated_at
    BEFORE UPDATE ON activity_log
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();