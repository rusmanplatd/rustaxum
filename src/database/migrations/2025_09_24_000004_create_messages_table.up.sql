-- Create messages table for E2EE chat
-- All message content is encrypted client-side before storage
-- Supports various message types including media, polls, and system messages
CREATE TABLE messages (
    id CHAR(26) PRIMARY KEY,
    conversation_id CHAR(26) NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    sender_user_id CHAR(26) NOT NULL REFERENCES sys_users(id),
    sender_device_id CHAR(26) NOT NULL REFERENCES devices(id),

    -- Message content and type
    message_type VARCHAR NOT NULL DEFAULT 'text' CHECK (message_type IN ('text', 'image', 'file', 'audio', 'video', 'location', 'contact', 'poll', 'system')),
    encrypted_content TEXT NOT NULL,
    content_algorithm VARCHAR NOT NULL,

    -- Message threading and relationships
    reply_to_message_id CHAR(26) REFERENCES messages(id),
    forward_from_message_id CHAR(26) REFERENCES messages(id),
    edit_of_message_id CHAR(26) REFERENCES messages(id),

    -- Message lifecycle and status
    is_edited BOOLEAN NOT NULL DEFAULT false,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    expires_at TIMESTAMPTZ,

    -- Message timing
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Add table and column comments
COMMENT ON TABLE messages IS 'Chat messages with end-to-end encryption support for various content types';
COMMENT ON COLUMN messages.id IS 'Unique message identifier (ULID format)';
COMMENT ON COLUMN messages.conversation_id IS 'Parent conversation - cascade deletes when conversation is removed';
COMMENT ON COLUMN messages.sender_user_id IS 'User who sent this message';
COMMENT ON COLUMN messages.sender_device_id IS 'Specific device used to send this message (for multi-device support)';
COMMENT ON COLUMN messages.message_type IS 'Content type: text, image, file, audio, video, location, contact, poll, system';
COMMENT ON COLUMN messages.encrypted_content IS 'Message content encrypted with conversation key (JSON blob containing all message data)';
COMMENT ON COLUMN messages.content_algorithm IS 'Encryption algorithm used for this specific message content';
COMMENT ON COLUMN messages.reply_to_message_id IS 'Reference to message being replied to (creates threading)';
COMMENT ON COLUMN messages.forward_from_message_id IS 'Reference to original message if this is a forward';
COMMENT ON COLUMN messages.edit_of_message_id IS 'Reference to original message if this is an edit';
COMMENT ON COLUMN messages.is_edited IS 'Whether this message has been edited after sending';
COMMENT ON COLUMN messages.is_deleted IS 'Soft delete flag - message content may be redacted but metadata preserved';
COMMENT ON COLUMN messages.expires_at IS 'When this message should auto-delete (disappearing messages feature)';
COMMENT ON COLUMN messages.sent_at IS 'When message was sent by sender (may differ from created_at due to offline sending)';
COMMENT ON COLUMN messages.created_at IS 'When message record was created in database';
COMMENT ON COLUMN messages.updated_at IS 'Last time message was modified (edits, status changes)';
COMMENT ON COLUMN messages.deleted_at IS 'Soft delete timestamp (null = not deleted)';

-- Indexes for message queries
CREATE INDEX idx_messages_conversation ON messages (conversation_id, sent_at DESC);
COMMENT ON INDEX idx_messages_conversation IS 'Primary message retrieval - get messages for conversation in chronological order';

CREATE INDEX idx_messages_sender ON messages (sender_user_id);
COMMENT ON INDEX idx_messages_sender IS 'Find all messages sent by a specific user across conversations';

CREATE INDEX idx_messages_device ON messages (sender_device_id);
COMMENT ON INDEX idx_messages_device IS 'Track messages by device for multi-device analytics and debugging';

CREATE INDEX idx_messages_reply_to ON messages (reply_to_message_id);
COMMENT ON INDEX idx_messages_reply_to IS 'Find all replies to a specific message for threading display';

CREATE INDEX idx_messages_forward_from ON messages (forward_from_message_id);
COMMENT ON INDEX idx_messages_forward_from IS 'Track forward chains and find all forwards of a message';

CREATE INDEX idx_messages_expires_at ON messages (expires_at) WHERE expires_at IS NOT NULL;
COMMENT ON INDEX idx_messages_expires_at IS 'Efficiently process disappearing messages for deletion (partial index)';

CREATE INDEX idx_messages_type ON messages (message_type);
COMMENT ON INDEX idx_messages_type IS 'Filter messages by content type for analytics and specialized processing';

CREATE INDEX idx_messages_created_at ON messages (created_at);
COMMENT ON INDEX idx_messages_created_at IS 'Global message timeline and database maintenance operations';