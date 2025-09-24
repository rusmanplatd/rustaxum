-- Create message delivery status tables for multi-device tracking
CREATE TABLE message_delivery_status (
    id CHAR(26) PRIMARY KEY,
    message_id CHAR(26) NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    recipient_device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Delivery tracking
    status VARCHAR NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'sent', 'delivered', 'read', 'failed')),
    delivered_at TIMESTAMPTZ,
    read_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    failure_reason VARCHAR,

    -- Retry tracking for failed deliveries
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    next_retry_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create typing indicators table for real-time features
CREATE TABLE typing_indicators (
    id CHAR(26) PRIMARY KEY,
    conversation_id CHAR(26) NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Typing status
    is_typing BOOLEAN NOT NULL DEFAULT true,
    started_typing_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 seconds'),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create presence status table for online/offline tracking
CREATE TABLE device_presence (
    id CHAR(26) PRIMARY KEY,
    device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Presence status
    status VARCHAR NOT NULL DEFAULT 'offline' CHECK (status IN ('online', 'away', 'busy', 'invisible', 'offline')),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Encrypted status message (optional custom status)
    encrypted_status_message TEXT,
    status_message_algorithm VARCHAR,

    -- Auto-status settings
    auto_away_after_minutes INTEGER DEFAULT 15,
    auto_offline_after_minutes INTEGER DEFAULT 60,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create push notification tokens table
CREATE TABLE device_push_tokens (
    id CHAR(26) PRIMARY KEY,
    device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Push notification settings
    platform VARCHAR NOT NULL CHECK (platform IN ('fcm', 'apns', 'web')), -- Firebase, Apple, Web Push
    token TEXT NOT NULL,
    endpoint TEXT, -- For web push

    -- Notification preferences (encrypted)
    encrypted_notification_settings TEXT,
    settings_algorithm VARCHAR,

    -- Token status
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Unique constraints
CREATE UNIQUE INDEX idx_message_delivery_status_unique ON message_delivery_status (message_id, recipient_device_id);
CREATE UNIQUE INDEX idx_typing_indicators_unique ON typing_indicators (conversation_id, user_id, device_id);
CREATE UNIQUE INDEX idx_device_presence_device ON device_presence (device_id);

-- Indexes for delivery status
CREATE INDEX idx_message_delivery_status_message ON message_delivery_status (message_id);
CREATE INDEX idx_message_delivery_status_device ON message_delivery_status (recipient_device_id);
CREATE INDEX idx_message_delivery_status_status ON message_delivery_status (status);
CREATE INDEX idx_message_delivery_status_retry ON message_delivery_status (next_retry_at) WHERE status = 'failed' AND retry_count < max_retries;

-- Indexes for typing indicators
CREATE INDEX idx_typing_indicators_conversation ON typing_indicators (conversation_id, is_typing);
CREATE INDEX idx_typing_indicators_user ON typing_indicators (user_id, device_id);
CREATE INDEX idx_typing_indicators_expires ON typing_indicators (expires_at);

-- Indexes for presence
CREATE INDEX idx_device_presence_status ON device_presence (status, last_seen_at);
CREATE INDEX idx_device_presence_last_seen ON device_presence (last_seen_at);

-- Indexes for push tokens
CREATE INDEX idx_device_push_tokens_device ON device_push_tokens (device_id);
CREATE INDEX idx_device_push_tokens_platform ON device_push_tokens (platform, is_active);
CREATE INDEX idx_device_push_tokens_token ON device_push_tokens (token) WHERE is_active = true;