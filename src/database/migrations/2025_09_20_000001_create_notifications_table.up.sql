-- Create notifications table for storing notification history
CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY,
    type VARCHAR(255) NOT NULL,
    notifiable_type VARCHAR(255) NOT NULL,
    notifiable_id VARCHAR(255) NOT NULL,
    data JSONB NOT NULL,
    channels TEXT[] NOT NULL DEFAULT '{}',
    read_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    error_message TEXT,
    priority INTEGER DEFAULT 0,
    scheduled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_notifications_type ON notifications(type);
CREATE INDEX IF NOT EXISTS idx_notifications_notifiable ON notifications(notifiable_type, notifiable_id);
CREATE INDEX IF NOT EXISTS idx_notifications_read_at ON notifications(read_at);
CREATE INDEX IF NOT EXISTS idx_notifications_sent_at ON notifications(sent_at);
CREATE INDEX IF NOT EXISTS idx_notifications_failed_at ON notifications(failed_at);
CREATE INDEX IF NOT EXISTS idx_notifications_scheduled_at ON notifications(scheduled_at);
CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications(created_at);
CREATE INDEX IF NOT EXISTS idx_notifications_priority ON notifications(priority);

-- Create composite index for unread notifications
CREATE INDEX IF NOT EXISTS idx_notifications_unread ON notifications(notifiable_type, notifiable_id, read_at) WHERE read_at IS NULL;

-- Create composite index for failed notifications
CREATE INDEX IF NOT EXISTS idx_notifications_failed ON notifications(failed_at, retry_count, max_retries) WHERE failed_at IS NOT NULL;

-- Add trigger to update updated_at timestamp
CREATE TRIGGER update_notifications_updated_at
    BEFORE UPDATE ON notifications
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();