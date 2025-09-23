-- Drop the trigger first
DROP TRIGGER IF EXISTS update_activity_log_updated_at ON activity_log;

-- Drop the function
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop all indexes
DROP INDEX IF EXISTS idx_activity_log_correlation_created;
DROP INDEX IF EXISTS idx_activity_log_causer_created;
DROP INDEX IF EXISTS idx_activity_log_subject_created;
DROP INDEX IF EXISTS idx_activity_log_created_at;
DROP INDEX IF EXISTS idx_activity_log_event;
DROP INDEX IF EXISTS idx_activity_log_batch_uuid;
DROP INDEX IF EXISTS idx_activity_log_correlation_id;
DROP INDEX IF EXISTS idx_activity_log_causer;
DROP INDEX IF EXISTS idx_activity_log_subject;
DROP INDEX IF EXISTS idx_activity_log_log_name;

-- Drop the table
DROP TABLE IF EXISTS activity_log;