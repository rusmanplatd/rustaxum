-- Create scheduled messages table for E2EE chat
-- Enables messages to be composed and encrypted, then sent at a future time
CREATE TABLE scheduled_messages (
    id CHAR(26) PRIMARY KEY,
    message_id CHAR(26) NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    conversation_id CHAR(26) NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    sender_user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    sender_device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Scheduling details
    scheduled_for TIMESTAMPTZ NOT NULL,
    timezone VARCHAR(50) NOT NULL DEFAULT 'UTC',

    -- Processing status
    is_sent BOOLEAN NOT NULL DEFAULT false,
    sent_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    failure_reason TEXT,

    -- Retry mechanism
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    next_retry_at TIMESTAMPTZ,

    -- Cancellation support
    is_cancelled BOOLEAN NOT NULL DEFAULT false,
    cancelled_at TIMESTAMPTZ,
    cancelled_by_device_id CHAR(26) REFERENCES devices(id),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create message editing queue for scheduled messages
CREATE TABLE scheduled_message_edits (
    id CHAR(26) PRIMARY KEY,
    scheduled_message_id CHAR(26) NOT NULL REFERENCES scheduled_messages(id) ON DELETE CASCADE,

    -- New encrypted content (replaces original message content)
    new_encrypted_content TEXT NOT NULL,
    new_content_algorithm VARCHAR NOT NULL,

    -- Edit metadata
    edited_by_device_id CHAR(26) NOT NULL REFERENCES devices(id),
    is_applied BOOLEAN NOT NULL DEFAULT false,
    applied_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for scheduled messages
CREATE INDEX idx_scheduled_messages_message ON scheduled_messages (message_id);
CREATE INDEX idx_scheduled_messages_conversation ON scheduled_messages (conversation_id);
CREATE INDEX idx_scheduled_messages_sender ON scheduled_messages (sender_user_id, sender_device_id);
CREATE INDEX idx_scheduled_messages_pending ON scheduled_messages (scheduled_for, is_sent, is_cancelled) WHERE is_sent = false AND is_cancelled = false;
CREATE INDEX idx_scheduled_messages_retry ON scheduled_messages (next_retry_at, retry_count, max_retries) WHERE failed_at IS NOT NULL AND retry_count < max_retries;
CREATE INDEX idx_scheduled_messages_timezone ON scheduled_messages (timezone, scheduled_for);

-- Indexes for scheduled message edits
CREATE INDEX idx_scheduled_message_edits_scheduled ON scheduled_message_edits (scheduled_message_id);
CREATE INDEX idx_scheduled_message_edits_device ON scheduled_message_edits (edited_by_device_id);
CREATE INDEX idx_scheduled_message_edits_pending ON scheduled_message_edits (is_applied, created_at) WHERE is_applied = false;