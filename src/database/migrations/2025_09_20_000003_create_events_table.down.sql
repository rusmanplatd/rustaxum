-- Drop the events table and related objects
DROP TRIGGER IF EXISTS update_events_updated_at ON events;
DROP FUNCTION IF EXISTS update_updated_at_column();
DROP INDEX IF EXISTS idx_events_aggregate;
DROP INDEX IF EXISTS idx_events_created_at;
DROP INDEX IF EXISTS idx_events_occurred_at;
DROP INDEX IF EXISTS idx_events_aggregate_type;
DROP INDEX IF EXISTS idx_events_aggregate_id;
DROP INDEX IF EXISTS idx_events_event_name;
DROP TABLE IF EXISTS events;