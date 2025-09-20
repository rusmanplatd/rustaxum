-- Create push subscriptions table for web push notifications
CREATE TABLE push_subscriptions (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL,
    endpoint TEXT NOT NULL,
    p256dh_key TEXT NOT NULL,
    auth_key TEXT NOT NULL,
    user_agent TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for efficient user queries
CREATE INDEX idx_push_subscriptions_user_id ON push_subscriptions(user_id);

-- Unique constraint on user_id and endpoint to prevent duplicates
CREATE UNIQUE INDEX idx_push_subscriptions_user_endpoint ON push_subscriptions(user_id, endpoint);

-- Index for endpoint queries (for cleanup)
CREATE INDEX idx_push_subscriptions_endpoint ON push_subscriptions(endpoint);