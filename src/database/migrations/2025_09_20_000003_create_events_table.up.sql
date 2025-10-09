-- Create events table for event sourcing and audit trail
CREATE TABLE events (
    id CHAR(26) PRIMARY KEY,
    event_name VARCHAR(255) NOT NULL,
    event_data JSONB NOT NULL,
    aggregate_id VARCHAR(255),
    aggregate_type VARCHAR(255),
    version INTEGER DEFAULT 1,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX idx_events_event_name ON events(event_name);
CREATE INDEX idx_events_aggregate_id ON events(aggregate_id);
CREATE INDEX idx_events_aggregate_type ON events(aggregate_type);
CREATE INDEX idx_events_occurred_at ON events(occurred_at);
CREATE INDEX idx_events_created_at ON events(created_at);

-- Create composite index for aggregate queries
CREATE INDEX idx_events_aggregate ON events(aggregate_type, aggregate_id, version);

CREATE TRIGGER update_events_updated_at
    BEFORE UPDATE ON events
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();