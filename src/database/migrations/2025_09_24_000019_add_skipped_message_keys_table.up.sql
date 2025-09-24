-- Create skipped message keys table for handling out-of-order messages
-- Stores encrypted message keys for messages that arrive out of sequence
-- Essential for Signal Protocol Double Ratchet implementation with proper message ordering

CREATE TABLE skipped_message_keys (
    id CHAR(26) PRIMARY KEY,
    session_id CHAR(26) NOT NULL REFERENCES signal_sessions(id) ON DELETE CASCADE,

    -- Encrypted message key for out-of-order decryption
    encrypted_message_key TEXT NOT NULL,
    key_algorithm VARCHAR NOT NULL,

    -- Message sequence information
    message_number INTEGER NOT NULL,
    chain_key_index INTEGER NOT NULL,
    header_key TEXT NOT NULL,

    -- Message identification
    sender_device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    expected_message_id CHAR(26), -- May be null if message hasn't arrived yet

    -- Lifecycle management
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '7 days',
    used_at TIMESTAMPTZ,
    is_used BOOLEAN NOT NULL DEFAULT false

);

-- Table and column comments
COMMENT ON TABLE skipped_message_keys IS 'Stores encrypted message keys for out-of-order Signal Protocol messages - enables proper decryption when messages arrive in wrong sequence while maintaining Double Ratchet forward secrecy';
COMMENT ON COLUMN skipped_message_keys.id IS 'Unique skipped key identifier (ULID format)';
COMMENT ON COLUMN skipped_message_keys.session_id IS 'Signal session this skipped key belongs to - cascade deletes with session';
COMMENT ON COLUMN skipped_message_keys.encrypted_message_key IS 'Message key encrypted with chain key - used to decrypt specific out-of-order message';
COMMENT ON COLUMN skipped_message_keys.key_algorithm IS 'Algorithm used to encrypt the message key (matches session algorithm)';
COMMENT ON COLUMN skipped_message_keys.message_number IS 'Sequence number of the message this key can decrypt - used for message ordering';
COMMENT ON COLUMN skipped_message_keys.chain_key_index IS 'Index in the chain key sequence - determines key derivation path';
COMMENT ON COLUMN skipped_message_keys.header_key IS 'Key used to decrypt message header - needed for proper message processing';
COMMENT ON COLUMN skipped_message_keys.sender_device_id IS 'Device that sent the message this key decrypts - for multi-device session management';
COMMENT ON COLUMN skipped_message_keys.expected_message_id IS 'Message ID when it arrives - links skipped key to actual message';
COMMENT ON COLUMN skipped_message_keys.created_at IS 'When skipped key was created - used for cleanup and expiration';
COMMENT ON COLUMN skipped_message_keys.expires_at IS 'When skipped key expires and should be deleted - prevents indefinite key storage';
COMMENT ON COLUMN skipped_message_keys.used_at IS 'When skipped key was used to decrypt message - marks key as consumed';
COMMENT ON COLUMN skipped_message_keys.is_used IS 'Whether key has been used to decrypt a message - used keys can be cleaned up';

-- Create message key pools for efficient batch key storage
-- Optimizes storage when many messages are skipped in sequence
CREATE TABLE message_key_pools (
    id CHAR(26) PRIMARY KEY,
    session_id CHAR(26) NOT NULL REFERENCES signal_sessions(id) ON DELETE CASCADE,

    -- Key pool information
    pool_start_index INTEGER NOT NULL,
    pool_end_index INTEGER NOT NULL,
    pool_size INTEGER NOT NULL,

    -- Encrypted pool of message keys
    encrypted_key_pool TEXT NOT NULL,
    pool_algorithm VARCHAR NOT NULL,

    -- Pool metadata
    sender_device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '7 days',

    -- Usage tracking
    keys_used_count INTEGER NOT NULL DEFAULT 0,
    is_pool_exhausted BOOLEAN NOT NULL DEFAULT false

);

-- Table and column comments for message key pools
COMMENT ON TABLE message_key_pools IS 'Stores pools of message keys for efficient handling of large gaps in message sequence - optimizes storage when many consecutive messages are skipped';
COMMENT ON COLUMN message_key_pools.id IS 'Unique key pool identifier (ULID format)';
COMMENT ON COLUMN message_key_pools.session_id IS 'Signal session this key pool belongs to';
COMMENT ON COLUMN message_key_pools.pool_start_index IS 'Starting message sequence number for this pool';
COMMENT ON COLUMN message_key_pools.pool_end_index IS 'Ending message sequence number for this pool';
COMMENT ON COLUMN message_key_pools.pool_size IS 'Number of keys stored in this pool';
COMMENT ON COLUMN message_key_pools.encrypted_key_pool IS 'Encrypted array of message keys for sequence range';
COMMENT ON COLUMN message_key_pools.pool_algorithm IS 'Algorithm used to encrypt the key pool';
COMMENT ON COLUMN message_key_pools.sender_device_id IS 'Device that sent messages this pool can decrypt';
COMMENT ON COLUMN message_key_pools.keys_used_count IS 'Number of keys from pool that have been used';
COMMENT ON COLUMN message_key_pools.is_pool_exhausted IS 'Whether all keys in pool have been consumed';

-- Unique constraints and indexes for skipped message keys
CREATE UNIQUE INDEX idx_skipped_message_keys_session_number ON skipped_message_keys (session_id, message_number);
COMMENT ON INDEX idx_skipped_message_keys_session_number IS 'Prevents duplicate skipped keys for same message number in session';

CREATE INDEX idx_skipped_message_keys_session ON skipped_message_keys (session_id);
COMMENT ON INDEX idx_skipped_message_keys_session IS 'Find all skipped keys for a specific session';

CREATE INDEX idx_skipped_message_keys_sender ON skipped_message_keys (sender_device_id, message_number);
COMMENT ON INDEX idx_skipped_message_keys_sender IS 'Find skipped keys by sender device with message ordering';

CREATE INDEX idx_skipped_message_keys_unused ON skipped_message_keys (session_id, is_used, message_number) WHERE is_used = false;
COMMENT ON INDEX idx_skipped_message_keys_unused IS 'Efficiently find unused skipped keys for decryption attempts - partial index';

CREATE INDEX idx_skipped_message_keys_expired ON skipped_message_keys (expires_at) WHERE is_used = false;
COMMENT ON INDEX idx_skipped_message_keys_expired IS 'Optimize cleanup of expired unused skipped keys';

CREATE INDEX idx_skipped_message_keys_chain_index ON skipped_message_keys (session_id, chain_key_index);
COMMENT ON INDEX idx_skipped_message_keys_chain_index IS 'Find skipped keys by chain position for Double Ratchet operations';

-- Indexes for message key pools
CREATE INDEX idx_message_key_pools_session ON message_key_pools (session_id);
COMMENT ON INDEX idx_message_key_pools_session IS 'Find key pools for a specific session';

CREATE INDEX idx_message_key_pools_range ON message_key_pools (session_id, pool_start_index, pool_end_index);
COMMENT ON INDEX idx_message_key_pools_range IS 'Find key pools covering specific message number ranges';

CREATE INDEX idx_message_key_pools_sender ON message_key_pools (sender_device_id, created_at);
COMMENT ON INDEX idx_message_key_pools_sender IS 'Find key pools by sender with creation ordering';

CREATE INDEX idx_message_key_pools_expired ON message_key_pools (expires_at) WHERE is_pool_exhausted = false;
COMMENT ON INDEX idx_message_key_pools_expired IS 'Optimize cleanup of expired non-exhausted key pools';

-- Add partial index for active pools with remaining keys
CREATE INDEX idx_message_key_pools_active ON message_key_pools (session_id, keys_used_count, pool_size) WHERE is_pool_exhausted = false;
COMMENT ON INDEX idx_message_key_pools_active IS 'Efficiently find active key pools with remaining unused keys';