-- Create device synchronization tables for multi-device E2EE
CREATE TABLE device_sync_sessions (
    id CHAR(26) PRIMARY KEY,
    primary_device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    secondary_device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Encrypted sync state between user's own devices
    encrypted_sync_key TEXT NOT NULL,
    sync_algorithm VARCHAR NOT NULL,

    -- Sync status and metadata
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_sync_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    established_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table comment for device_sync_sessions
COMMENT ON TABLE device_sync_sessions IS 'Manages synchronization sessions between a user''s own devices in multi-device E2EE setup. Enables secure sharing of conversation keys, message history, and settings across user-owned devices while maintaining E2EE integrity.';

-- Column comments for device_sync_sessions table
COMMENT ON COLUMN device_sync_sessions.id IS 'ULID primary key uniquely identifying this device sync relationship';
COMMENT ON COLUMN device_sync_sessions.primary_device_id IS 'Foreign key to primary device in sync relationship - typically the first registered device or designated master device';
COMMENT ON COLUMN device_sync_sessions.secondary_device_id IS 'Foreign key to secondary device being synced - receives data from primary device in sync operations';
COMMENT ON COLUMN device_sync_sessions.encrypted_sync_key IS 'Encrypted key for device-to-device synchronization - enables secure transfer of E2EE keys and data between user-owned devices';
COMMENT ON COLUMN device_sync_sessions.sync_algorithm IS 'Algorithm used to encrypt sync communications - typically AES-256-GCM for device sync operations';
COMMENT ON COLUMN device_sync_sessions.is_active IS 'Whether sync session is currently active - inactive sessions preserve sync history but don''t participate in new sync operations';
COMMENT ON COLUMN device_sync_sessions.last_sync_at IS 'Timestamp of last successful synchronization - used for sync scheduling and conflict resolution';
COMMENT ON COLUMN device_sync_sessions.established_at IS 'Sync relationship establishment timestamp - marks completion of device pairing and sync setup';
COMMENT ON COLUMN device_sync_sessions.created_at IS 'Sync session creation timestamp for audit and lifecycle management';
COMMENT ON COLUMN device_sync_sessions.updated_at IS 'Last sync session update timestamp for activity tracking';

-- NOTE: device_key_rotations table is now defined in migration 2025_09_24_000018_add_signal_protocol_key_rotation.up.sql
-- This duplicate definition has been removed to prevent migration conflicts

-- Create conversation encryption settings per device
CREATE TABLE conversation_device_settings (
    id CHAR(26) PRIMARY KEY,
    conversation_id CHAR(26) NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Device-specific encryption preferences
    preferred_algorithm VARCHAR DEFAULT 'aes-256-gcm',
    preferred_key_exchange VARCHAR DEFAULT 'curve25519',
    preferred_mac VARCHAR DEFAULT 'hmac-sha256',

    -- Device capabilities for this conversation
    supports_disappearing_messages BOOLEAN NOT NULL DEFAULT true,
    supports_file_encryption BOOLEAN NOT NULL DEFAULT true,
    supports_voice_encryption BOOLEAN NOT NULL DEFAULT true,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table comment for conversation_device_settings
COMMENT ON TABLE conversation_device_settings IS 'Stores device-specific encryption preferences and capabilities per conversation. Enables fine-grained control over E2EE behavior based on device capabilities, user preferences, and conversation requirements while maintaining compatibility across diverse device types.';

-- Column comments for conversation_device_settings table
COMMENT ON COLUMN conversation_device_settings.id IS 'ULID primary key uniquely identifying this device-conversation settings combination';
COMMENT ON COLUMN conversation_device_settings.conversation_id IS 'Foreign key to conversation these settings apply to - enables conversation-specific encryption preferences';
COMMENT ON COLUMN conversation_device_settings.device_id IS 'Foreign key to device with these settings - different devices can have different capabilities and preferences';
COMMENT ON COLUMN conversation_device_settings.preferred_algorithm IS 'Device preferred encryption algorithm for this conversation - influences algorithm negotiation process';
COMMENT ON COLUMN conversation_device_settings.preferred_key_exchange IS 'Device preferred key exchange method - used during session establishment and key negotiation';
COMMENT ON COLUMN conversation_device_settings.preferred_mac IS 'Device preferred MAC algorithm for message authentication - ensures message integrity verification';
COMMENT ON COLUMN conversation_device_settings.supports_disappearing_messages IS 'Whether device supports disappearing messages in this conversation - affects feature availability';
COMMENT ON COLUMN conversation_device_settings.supports_file_encryption IS 'Whether device supports encrypted file sharing in this conversation - influences attachment handling';
COMMENT ON COLUMN conversation_device_settings.supports_voice_encryption IS 'Whether device supports encrypted voice messages in this conversation - affects media message capabilities';
COMMENT ON COLUMN conversation_device_settings.created_at IS 'Settings creation timestamp for audit and lifecycle tracking';
COMMENT ON COLUMN conversation_device_settings.updated_at IS 'Last settings update timestamp for change tracking';

-- Unique constraints
CREATE UNIQUE INDEX idx_device_sync_sessions_pair ON device_sync_sessions (primary_device_id, secondary_device_id);
COMMENT ON INDEX idx_device_sync_sessions_pair IS 'Enforces unique sync relationships between device pairs - prevents duplicate sync sessions and ensures consistent sync state';

CREATE UNIQUE INDEX idx_conversation_device_settings_unique ON conversation_device_settings (conversation_id, device_id);
COMMENT ON INDEX idx_conversation_device_settings_unique IS 'Enforces unique settings per device per conversation - prevents conflicting encryption preferences and ensures settings consistency';

-- Indexes for device sync queries
CREATE INDEX idx_device_sync_sessions_primary ON device_sync_sessions (primary_device_id);
COMMENT ON INDEX idx_device_sync_sessions_primary IS 'Optimizes queries for sync sessions by primary device - enables efficient sync relationship management and status tracking';

CREATE INDEX idx_device_sync_sessions_secondary ON device_sync_sessions (secondary_device_id);
COMMENT ON INDEX idx_device_sync_sessions_secondary IS 'Optimizes queries for sync sessions by secondary device - supports bidirectional sync relationship lookups';

CREATE INDEX idx_device_sync_sessions_active ON device_sync_sessions (is_active, last_sync_at);
COMMENT ON INDEX idx_device_sync_sessions_active IS 'Optimizes active sync session queries with recency ordering - enables sync scheduling and inactive session cleanup';

-- NOTE: Indexes for device_key_rotations are now defined in migration 2025_09_24_000018_add_signal_protocol_key_rotation.up.sql

-- Indexes for conversation device settings
CREATE INDEX idx_conversation_device_settings_conversation ON conversation_device_settings (conversation_id);
COMMENT ON INDEX idx_conversation_device_settings_conversation IS 'Optimizes conversation-level settings queries - enables algorithm negotiation and feature compatibility checking';

CREATE INDEX idx_conversation_device_settings_device ON conversation_device_settings (device_id);
COMMENT ON INDEX idx_conversation_device_settings_device IS 'Optimizes device-centric settings queries - supports device capability management and preference application';