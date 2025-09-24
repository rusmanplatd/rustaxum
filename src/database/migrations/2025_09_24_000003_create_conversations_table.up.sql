-- Create conversations table for E2EE chat
-- Supports direct messages, group chats, and broadcast channels
-- Encryption settings are immutable after first message to ensure security
CREATE TABLE conversations (
    id CHAR(26) PRIMARY KEY,

    -- Conversation type and encryption configuration
    conversation_type VARCHAR NOT NULL CHECK (conversation_type IN ('direct', 'group', 'channel')),
    is_encrypted BOOLEAN NOT NULL DEFAULT false,
    encryption_immutable BOOLEAN NOT NULL DEFAULT false,

    -- Encrypted conversation metadata (encrypted with conversation key)
    encrypted_name TEXT,
    encrypted_description TEXT,
    encrypted_avatar_url TEXT,

    -- Negotiated encryption algorithms for this conversation
    preferred_algorithm VARCHAR DEFAULT 'aes-256-gcm',
    preferred_key_exchange VARCHAR DEFAULT 'curve25519',
    preferred_mac VARCHAR DEFAULT 'hmac-sha256',

    -- Conversation ownership and limits
    creator_id CHAR(26) REFERENCES sys_users(id),
    max_participants INTEGER,
    is_public BOOLEAN NOT NULL DEFAULT false,

    -- Message lifecycle settings
    disappearing_messages_timer INTEGER,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Add table and column comments
COMMENT ON TABLE conversations IS 'Chat conversations supporting direct messages, groups, and channels with optional E2EE';
COMMENT ON COLUMN conversations.id IS 'Unique conversation identifier (ULID format)';
COMMENT ON COLUMN conversations.conversation_type IS 'Type of conversation: direct (1:1), group (multi-user), channel (broadcast)';
COMMENT ON COLUMN conversations.is_encrypted IS 'Whether this conversation uses end-to-end encryption';
COMMENT ON COLUMN conversations.encryption_immutable IS 'Once true, encryption setting cannot be changed for security';
COMMENT ON COLUMN conversations.encrypted_name IS 'Group/channel display name encrypted with conversation key (null for direct chats)';
COMMENT ON COLUMN conversations.encrypted_description IS 'Group/channel description encrypted with conversation key';
COMMENT ON COLUMN conversations.encrypted_avatar_url IS 'Group/channel avatar image URL encrypted with conversation key';
COMMENT ON COLUMN conversations.preferred_algorithm IS 'Negotiated encryption algorithm for message content (AES-256-GCM, ChaCha20-Poly1305, etc.)';
COMMENT ON COLUMN conversations.preferred_key_exchange IS 'Negotiated key exchange method (Curve25519, P-256, RSA)';
COMMENT ON COLUMN conversations.preferred_mac IS 'Negotiated message authentication code algorithm (HMAC-SHA256, Blake3, etc.)';
COMMENT ON COLUMN conversations.creator_id IS 'User who created this conversation (owner for groups/channels)';
COMMENT ON COLUMN conversations.max_participants IS 'Maximum number of participants allowed (null = unlimited)';
COMMENT ON COLUMN conversations.is_public IS 'Whether this channel is publicly discoverable (channels only)';
COMMENT ON COLUMN conversations.disappearing_messages_timer IS 'Auto-delete timer for messages in seconds (null = messages persist)';
COMMENT ON COLUMN conversations.created_at IS 'When conversation was created';
COMMENT ON COLUMN conversations.updated_at IS 'Last time conversation metadata was modified';
COMMENT ON COLUMN conversations.deleted_at IS 'Soft delete timestamp (null = not deleted)';

-- Indexes for conversation queries
CREATE INDEX idx_conversations_type ON conversations (conversation_type);
COMMENT ON INDEX idx_conversations_type IS 'Filter conversations by type (direct, group, channel)';

CREATE INDEX idx_conversations_encrypted ON conversations (is_encrypted);
COMMENT ON INDEX idx_conversations_encrypted IS 'Separate encrypted and unencrypted conversations for different processing';

CREATE INDEX idx_conversations_creator ON conversations (creator_id);
COMMENT ON INDEX idx_conversations_creator IS 'Find all conversations created by a specific user';

CREATE INDEX idx_conversations_created_at ON conversations (created_at);
COMMENT ON INDEX idx_conversations_created_at IS 'Order conversations chronologically for user interface';

CREATE INDEX idx_conversations_public ON conversations (is_public) WHERE is_public = true;
COMMENT ON INDEX idx_conversations_public IS 'Efficiently find public channels for discovery (partial index)';