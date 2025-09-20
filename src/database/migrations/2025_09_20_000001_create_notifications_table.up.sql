-- Create notifications table for storing database notifications
CREATE TABLE notifications (
    id TEXT PRIMARY KEY,
    notification_type VARCHAR(255) NOT NULL,
    notifiable_id TEXT NOT NULL,
    notifiable_type VARCHAR(255) NOT NULL,
    data JSONB NOT NULL,
    read_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for efficient queries by notifiable entity
CREATE INDEX idx_notifications_notifiable ON notifications(notifiable_type, notifiable_id);

-- Index for unread notifications
CREATE INDEX idx_notifications_unread ON notifications(notifiable_type, notifiable_id, read_at) WHERE read_at IS NULL;

-- Index for notification type queries
CREATE INDEX idx_notifications_type ON notifications(notification_type);

-- Index for created_at for chronological ordering
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);