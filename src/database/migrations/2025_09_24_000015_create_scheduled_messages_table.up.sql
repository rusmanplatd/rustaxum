-- Create scheduled messages table for E2EE chat
-- Enables messages to be composed and encrypted client-side, then sent at a future time
-- Messages are encrypted before scheduling to maintain E2EE security during storage
CREATE TABLE scheduled_messages (
    id CHAR(26) PRIMARY KEY,
    message_id CHAR(26) NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    conversation_id CHAR(26) NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    sender_user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    sender_device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Scheduling configuration
    scheduled_for TIMESTAMPTZ NOT NULL,
    timezone VARCHAR(50) NOT NULL DEFAULT 'UTC',

    -- Delivery processing status
    is_sent BOOLEAN NOT NULL DEFAULT false,
    sent_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    failure_reason TEXT,

    -- Retry mechanism for failed deliveries
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    next_retry_at TIMESTAMPTZ,

    -- User cancellation support
    is_cancelled BOOLEAN NOT NULL DEFAULT false,
    cancelled_at TIMESTAMPTZ,
    cancelled_by_device_id CHAR(26) REFERENCES devices(id),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add table and column comments
COMMENT ON TABLE scheduled_messages IS 'Manages message scheduling for E2EE chat - messages are encrypted client-side before scheduling to maintain security';
COMMENT ON COLUMN scheduled_messages.id IS 'Unique scheduled message identifier (ULID format)';
COMMENT ON COLUMN scheduled_messages.message_id IS 'Reference to the pre-encrypted message that will be delivered - cascade deletes if message is removed';
COMMENT ON COLUMN scheduled_messages.conversation_id IS 'Target conversation for delivery - ensures message reaches correct encrypted channel';
COMMENT ON COLUMN scheduled_messages.sender_user_id IS 'User who scheduled this message - used for permission verification and audit trails';
COMMENT ON COLUMN scheduled_messages.sender_device_id IS 'Device used to schedule message - important for multi-device E2EE key consistency';
COMMENT ON COLUMN scheduled_messages.scheduled_for IS 'Target delivery timestamp in UTC - when the encrypted message should be sent';
COMMENT ON COLUMN scheduled_messages.timezone IS 'Original timezone context for user experience (UI display and scheduling logic)';
COMMENT ON COLUMN scheduled_messages.is_sent IS 'Whether message has been successfully delivered to conversation participants';
COMMENT ON COLUMN scheduled_messages.sent_at IS 'Actual delivery timestamp - may differ from scheduled_for due to system load or failures';
COMMENT ON COLUMN scheduled_messages.failed_at IS 'When delivery attempt failed - triggers retry mechanism if within retry limits';
COMMENT ON COLUMN scheduled_messages.failure_reason IS 'Detailed failure reason for debugging and user notification (not encrypted, administrative info only)';
COMMENT ON COLUMN scheduled_messages.retry_count IS 'Number of delivery attempts made - prevents infinite retry loops';
COMMENT ON COLUMN scheduled_messages.max_retries IS 'Maximum retry attempts allowed before marking as permanently failed';
COMMENT ON COLUMN scheduled_messages.next_retry_at IS 'When next delivery attempt should be made - implements exponential backoff';
COMMENT ON COLUMN scheduled_messages.is_cancelled IS 'Whether user cancelled delivery before it was sent';
COMMENT ON COLUMN scheduled_messages.cancelled_at IS 'When cancellation occurred - important for audit trails';
COMMENT ON COLUMN scheduled_messages.cancelled_by_device_id IS 'Device that cancelled delivery - supports multi-device cancellation verification';
COMMENT ON COLUMN scheduled_messages.created_at IS 'When message was originally scheduled';
COMMENT ON COLUMN scheduled_messages.updated_at IS 'Last status update timestamp';

-- Create message editing queue for scheduled messages
-- Allows users to edit scheduled messages before they are sent while maintaining E2EE
CREATE TABLE scheduled_message_edits (
    id CHAR(26) PRIMARY KEY,
    scheduled_message_id CHAR(26) NOT NULL REFERENCES scheduled_messages(id) ON DELETE CASCADE,

    -- New encrypted content (replaces original message content)
    new_encrypted_content TEXT NOT NULL,
    new_content_algorithm VARCHAR NOT NULL,

    -- Edit tracking and authorization
    edited_by_device_id CHAR(26) NOT NULL REFERENCES devices(id),
    is_applied BOOLEAN NOT NULL DEFAULT false,
    applied_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add table and column comments for scheduled message edits
COMMENT ON TABLE scheduled_message_edits IS 'Manages edits to scheduled messages before delivery - preserves E2EE by storing new encrypted content';
COMMENT ON COLUMN scheduled_message_edits.id IS 'Unique edit operation identifier (ULID format)';
COMMENT ON COLUMN scheduled_message_edits.scheduled_message_id IS 'Reference to scheduled message being edited - cascade deletes with parent';
COMMENT ON COLUMN scheduled_message_edits.new_encrypted_content IS 'Updated message content encrypted with same algorithm as original - maintains E2EE security';
COMMENT ON COLUMN scheduled_message_edits.new_content_algorithm IS 'Encryption algorithm used for new content - should match conversation negotiated algorithms';
COMMENT ON COLUMN scheduled_message_edits.edited_by_device_id IS 'Device that performed the edit - used for multi-device authorization and audit trails';
COMMENT ON COLUMN scheduled_message_edits.is_applied IS 'Whether this edit has been applied to the scheduled message - prevents duplicate edits';
COMMENT ON COLUMN scheduled_message_edits.applied_at IS 'When edit was applied to scheduled message - audit trail for edit history';
COMMENT ON COLUMN scheduled_message_edits.created_at IS 'When edit was requested - helps order multiple edits';

-- Indexes for scheduled messages
CREATE INDEX idx_scheduled_messages_message ON scheduled_messages (message_id);
COMMENT ON INDEX idx_scheduled_messages_message IS 'Link scheduled messages to their encrypted message content for delivery processing';

CREATE INDEX idx_scheduled_messages_conversation ON scheduled_messages (conversation_id);
COMMENT ON INDEX idx_scheduled_messages_conversation IS 'Group scheduled messages by conversation for batch delivery optimization';

CREATE INDEX idx_scheduled_messages_sender ON scheduled_messages (sender_user_id, sender_device_id);
COMMENT ON INDEX idx_scheduled_messages_sender IS 'Find scheduled messages by sender for user management and device-specific operations';

CREATE INDEX idx_scheduled_messages_pending ON scheduled_messages (scheduled_for, is_sent, is_cancelled) WHERE is_sent = false AND is_cancelled = false;
COMMENT ON INDEX idx_scheduled_messages_pending IS 'Efficiently find pending messages ready for delivery (partial index for performance)';

CREATE INDEX idx_scheduled_messages_retry ON scheduled_messages (next_retry_at, retry_count, max_retries) WHERE failed_at IS NOT NULL AND retry_count < max_retries;
COMMENT ON INDEX idx_scheduled_messages_retry IS 'Optimize retry processing for failed deliveries within retry limits (partial index)';

CREATE INDEX idx_scheduled_messages_timezone ON scheduled_messages (timezone, scheduled_for);
COMMENT ON INDEX idx_scheduled_messages_timezone IS 'Support timezone-aware scheduling queries and user experience features';

-- Indexes for scheduled message edits
CREATE INDEX idx_scheduled_message_edits_scheduled ON scheduled_message_edits (scheduled_message_id);
COMMENT ON INDEX idx_scheduled_message_edits_scheduled IS 'Find all edit operations for a specific scheduled message';

CREATE INDEX idx_scheduled_message_edits_device ON scheduled_message_edits (edited_by_device_id);
COMMENT ON INDEX idx_scheduled_message_edits_device IS 'Track edit operations by device for multi-device consistency and audit trails';

CREATE INDEX idx_scheduled_message_edits_pending ON scheduled_message_edits (is_applied, created_at) WHERE is_applied = false;
COMMENT ON INDEX idx_scheduled_message_edits_pending IS 'Efficiently process pending edits before message delivery (partial index)';