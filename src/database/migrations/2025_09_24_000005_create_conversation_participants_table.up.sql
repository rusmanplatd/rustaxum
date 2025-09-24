-- Create conversation participants table
CREATE TABLE conversation_participants (
    id CHAR(26) PRIMARY KEY,
    conversation_id CHAR(26) NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,

    -- Participant role and permissions
    role VARCHAR NOT NULL DEFAULT 'member' CHECK (role IN ('owner', 'admin', 'member')),

    -- Participant status
    is_active BOOLEAN NOT NULL DEFAULT true,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    left_at TIMESTAMPTZ,

    -- E2EE specific fields for each participant
    -- Each participant may have different encryption settings
    last_read_message_id CHAR(26) REFERENCES messages(id),
    last_read_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table comments for conversation_participants
COMMENT ON TABLE conversation_participants IS 'Manages membership and participation in E2EE conversations. Each participant has individual encryption context, read receipts, and role-based permissions that affect their access to conversation keys and historical messages.';

-- Column comments explaining E2EE context and usage
COMMENT ON COLUMN conversation_participants.id IS 'ULID primary key uniquely identifying this participation relationship';
COMMENT ON COLUMN conversation_participants.conversation_id IS 'Foreign key to conversations table - defines which encrypted conversation this participation belongs to';
COMMENT ON COLUMN conversation_participants.user_id IS 'Foreign key to sys_users table - identifies the user participating in the encrypted conversation';
COMMENT ON COLUMN conversation_participants.role IS 'Participant role determining E2EE permissions: owner (full key management), admin (user management), member (basic participation). Affects access to conversation keys and ability to add/remove participants.';
COMMENT ON COLUMN conversation_participants.is_active IS 'Active participation status - false when user leaves conversation but retains access to encrypted message history from their participation period';
COMMENT ON COLUMN conversation_participants.joined_at IS 'Timestamp when user joined conversation - critical for E2EE forward secrecy as determines earliest encrypted messages accessible to this participant';
COMMENT ON COLUMN conversation_participants.left_at IS 'Timestamp when user left conversation - defines end of access period for new encrypted messages while preserving historical message access';
COMMENT ON COLUMN conversation_participants.last_read_message_id IS 'Reference to last message read by this participant - used for read receipts and determining unread encrypted message count';
COMMENT ON COLUMN conversation_participants.last_read_at IS 'Timestamp of last message read - supports read receipt functionality in E2EE context without exposing reading patterns to server';
COMMENT ON COLUMN conversation_participants.created_at IS 'Record creation timestamp for audit trail';
COMMENT ON COLUMN conversation_participants.updated_at IS 'Record modification timestamp for change tracking';

-- Unique constraint to prevent duplicate memberships
CREATE UNIQUE INDEX idx_conversation_participants_unique ON conversation_participants (conversation_id, user_id);
COMMENT ON INDEX idx_conversation_participants_unique IS 'Enforces one participation record per user per conversation - critical for E2EE integrity to prevent duplicate key distribution and role conflicts';

-- Indexes for participant queries
CREATE INDEX idx_conversation_participants_conversation ON conversation_participants (conversation_id);
COMMENT ON INDEX idx_conversation_participants_conversation IS 'Optimizes queries for all participants in a conversation - used for key distribution, permission checks, and message delivery in E2EE context';

CREATE INDEX idx_conversation_participants_user ON conversation_participants (user_id);
COMMENT ON INDEX idx_conversation_participants_user IS 'Optimizes user-centric queries for all conversations a user participates in - supports dashboard and notification delivery';

CREATE INDEX idx_conversation_participants_active ON conversation_participants (conversation_id, is_active);
COMMENT ON INDEX idx_conversation_participants_active IS 'Optimizes queries for active participants only - essential for E2EE key distribution and message routing to current participants';

CREATE INDEX idx_conversation_participants_role ON conversation_participants (conversation_id, role);
COMMENT ON INDEX idx_conversation_participants_role IS 'Optimizes role-based permission queries - critical for E2EE admin operations like key rotation, participant management, and conversation settings';

CREATE INDEX idx_conversation_participants_joined ON conversation_participants (joined_at);
COMMENT ON INDEX idx_conversation_participants_joined IS 'Optimizes temporal queries for participation history - supports audit trails and forward secrecy boundary enforcement';