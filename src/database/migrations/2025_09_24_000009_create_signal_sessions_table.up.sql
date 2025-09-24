-- Create Signal Protocol session state tables
-- Stores encrypted session state for Double Ratchet protocol (per device pair)
CREATE TABLE signal_sessions (
    id CHAR(26) PRIMARY KEY,
    local_device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    remote_device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    conversation_id CHAR(26) NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,

    -- Encrypted session state (Double Ratchet state encrypted with local device key)
    encrypted_session_state TEXT NOT NULL,
    session_algorithm VARCHAR NOT NULL,

    -- Session metadata
    session_version INTEGER NOT NULL DEFAULT 1,
    is_active BOOLEAN NOT NULL DEFAULT true,
    established_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Message counters for ordering (encrypted)
    encrypted_send_counter TEXT,
    encrypted_receive_counter TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table comment for signal_sessions
COMMENT ON TABLE signal_sessions IS 'Stores Signal Protocol Double Ratchet session state for secure device-to-device communication. Each device pair maintains an encrypted session state that provides forward secrecy and message ordering. Session state is encrypted locally and never exposed in plaintext server-side.';

-- Column comments for signal_sessions table
COMMENT ON COLUMN signal_sessions.id IS 'ULID primary key uniquely identifying this Signal Protocol session';
COMMENT ON COLUMN signal_sessions.local_device_id IS 'Foreign key to local device participating in this session - the device that owns this session state';
COMMENT ON COLUMN signal_sessions.remote_device_id IS 'Foreign key to remote device in this session - the other participant in the encrypted communication';
COMMENT ON COLUMN signal_sessions.conversation_id IS 'Foreign key to conversation context - sessions are scoped to specific conversations for key isolation';
COMMENT ON COLUMN signal_sessions.encrypted_session_state IS 'Double Ratchet session state encrypted with local device key - contains chain keys, root keys, and ratchet state for forward secrecy';
COMMENT ON COLUMN signal_sessions.session_algorithm IS 'Algorithm used to encrypt the session state (e.g., AES-256-GCM) - enables proper decryption by local device';
COMMENT ON COLUMN signal_sessions.session_version IS 'Signal Protocol version number - enables protocol version negotiation and backward compatibility';
COMMENT ON COLUMN signal_sessions.is_active IS 'Whether session is currently active - inactive sessions are preserved for message history but not used for new messages';
COMMENT ON COLUMN signal_sessions.established_at IS 'Session establishment timestamp - marks completion of X3DH key agreement and Double Ratchet initialization';
COMMENT ON COLUMN signal_sessions.last_used_at IS 'Last session usage timestamp - used for session cleanup and activity tracking';
COMMENT ON COLUMN signal_sessions.encrypted_send_counter IS 'Encrypted send message counter - prevents replay attacks and ensures message ordering (encrypted for privacy)';
COMMENT ON COLUMN signal_sessions.encrypted_receive_counter IS 'Encrypted receive message counter - tracks expected next message number for out-of-order detection';
COMMENT ON COLUMN signal_sessions.created_at IS 'Session creation timestamp for audit and lifecycle management';
COMMENT ON COLUMN signal_sessions.updated_at IS 'Last session update timestamp - updated on each message exchange for activity tracking';

-- Create sender key sessions for group messaging efficiency
CREATE TABLE sender_key_sessions (
    id CHAR(26) PRIMARY KEY,
    conversation_id CHAR(26) NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    sender_device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Encrypted sender key state for group messaging
    encrypted_sender_key_state TEXT NOT NULL,
    key_algorithm VARCHAR NOT NULL,

    -- Group key management
    key_generation INTEGER NOT NULL DEFAULT 1,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table comment for sender_key_sessions
COMMENT ON TABLE sender_key_sessions IS 'Stores sender key sessions for efficient group messaging in E2EE conversations. Enables one-to-many message encryption where sender encrypts once and all recipients can decrypt, reducing computational overhead in large groups while maintaining forward secrecy.';

-- Column comments for sender_key_sessions table
COMMENT ON COLUMN sender_key_sessions.id IS 'ULID primary key uniquely identifying this sender key session';
COMMENT ON COLUMN sender_key_sessions.conversation_id IS 'Foreign key to group conversation - sender keys are scoped to specific conversations for security isolation';
COMMENT ON COLUMN sender_key_sessions.sender_device_id IS 'Foreign key to device that created this sender key - each device maintains its own sender key for the group';
COMMENT ON COLUMN sender_key_sessions.encrypted_sender_key_state IS 'Encrypted sender key state including chain keys and ratchet state - encrypted with device key to protect group messaging keys';
COMMENT ON COLUMN sender_key_sessions.key_algorithm IS 'Algorithm used to encrypt the sender key state - enables proper decryption by the sender device';
COMMENT ON COLUMN sender_key_sessions.key_generation IS 'Key generation number for sender key rotation - higher generations indicate newer keys with improved forward secrecy';
COMMENT ON COLUMN sender_key_sessions.is_active IS 'Whether sender key is currently active - inactive keys are preserved for decrypting historical messages';
COMMENT ON COLUMN sender_key_sessions.created_at IS 'Sender key creation timestamp for lifecycle management';
COMMENT ON COLUMN sender_key_sessions.expires_at IS 'Optional expiration timestamp for automatic key rotation - null means no expiration';
COMMENT ON COLUMN sender_key_sessions.updated_at IS 'Last sender key update timestamp for activity and rotation tracking';

-- Unique constraints to prevent duplicate sessions
CREATE UNIQUE INDEX idx_signal_sessions_device_pair ON signal_sessions (local_device_id, remote_device_id, conversation_id);
COMMENT ON INDEX idx_signal_sessions_device_pair IS 'Enforces unique Signal sessions per device pair per conversation - prevents duplicate session establishment and ensures session state consistency';

CREATE UNIQUE INDEX idx_sender_key_sessions_device_conv ON sender_key_sessions (conversation_id, sender_device_id);
COMMENT ON INDEX idx_sender_key_sessions_device_conv IS 'Enforces unique sender key per device per conversation - prevents duplicate sender keys and ensures group messaging efficiency';

-- Indexes for session queries
CREATE INDEX idx_signal_sessions_local_device ON signal_sessions (local_device_id);
COMMENT ON INDEX idx_signal_sessions_local_device IS 'Optimizes local device session queries - enables efficient retrieval of all sessions for a specific device';

CREATE INDEX idx_signal_sessions_remote_device ON signal_sessions (remote_device_id);
COMMENT ON INDEX idx_signal_sessions_remote_device IS 'Optimizes remote device session queries - supports session lookup when receiving messages from specific devices';

CREATE INDEX idx_signal_sessions_conversation ON signal_sessions (conversation_id);
COMMENT ON INDEX idx_signal_sessions_conversation IS 'Optimizes conversation-scoped session queries - enables efficient session management per conversation';

CREATE INDEX idx_signal_sessions_active ON signal_sessions (is_active, last_used_at);
COMMENT ON INDEX idx_signal_sessions_active IS 'Optimizes active session queries with recency ordering - supports session cleanup and activity-based session management';

CREATE INDEX idx_sender_key_sessions_conversation ON sender_key_sessions (conversation_id);
COMMENT ON INDEX idx_sender_key_sessions_conversation IS 'Optimizes conversation-level sender key queries - enables group key distribution and management';

CREATE INDEX idx_sender_key_sessions_sender ON sender_key_sessions (sender_device_id);
COMMENT ON INDEX idx_sender_key_sessions_sender IS 'Optimizes device-specific sender key queries - supports sender key lifecycle management per device';

CREATE INDEX idx_sender_key_sessions_active ON sender_key_sessions (is_active, key_generation);
COMMENT ON INDEX idx_sender_key_sessions_active IS 'Optimizes active sender key queries with generation ordering - supports key rotation and latest key retrieval';