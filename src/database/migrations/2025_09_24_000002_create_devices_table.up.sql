-- Create devices table for multi-device E2EE support
-- Each user can have multiple devices, each with their own cryptographic keys
-- Supports Signal Protocol's multi-device architecture
CREATE TABLE devices (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    device_name VARCHAR NOT NULL,
    device_type VARCHAR NOT NULL,

    -- Signal Protocol public keys (private keys never stored on server)
    identity_public_key TEXT NOT NULL,
    signed_prekey_public TEXT NOT NULL,
    signed_prekey_signature TEXT NOT NULL,
    signed_prekey_id INTEGER NOT NULL,

    -- Device encryption capabilities
    supported_algorithms TEXT[] NOT NULL DEFAULT '{}',

    -- Device lifecycle and status
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    registration_id INTEGER NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add table and column comments
COMMENT ON TABLE devices IS 'Multi-device E2EE support - each user device has unique cryptographic keys for Signal Protocol';
COMMENT ON COLUMN devices.id IS 'Unique device identifier (ULID format)';
COMMENT ON COLUMN devices.user_id IS 'Owner of this device - cascades delete when user is removed';
COMMENT ON COLUMN devices.device_name IS 'User-friendly device name (e.g. "John''s iPhone", "Work Laptop")';
COMMENT ON COLUMN devices.device_type IS 'Device platform type: mobile, desktop, web, tablet, etc.';
COMMENT ON COLUMN devices.identity_public_key IS 'Device''s Ed25519 identity public key for Signal Protocol (base64 encoded)';
COMMENT ON COLUMN devices.signed_prekey_public IS 'Current signed prekey for X3DH key agreement (base64 encoded)';
COMMENT ON COLUMN devices.signed_prekey_signature IS 'Signature of signed prekey using identity key (base64 encoded)';
COMMENT ON COLUMN devices.signed_prekey_id IS 'Incrementing ID for signed prekey rotation tracking';
COMMENT ON COLUMN devices.supported_algorithms IS 'Array of encryption algorithms this device supports for capability negotiation';
COMMENT ON COLUMN devices.is_active IS 'Whether device is currently registered and can receive messages';
COMMENT ON COLUMN devices.last_seen_at IS 'Last time device was online or sent a message';
COMMENT ON COLUMN devices.registration_id IS 'Signal Protocol registration ID for device uniqueness verification';
COMMENT ON COLUMN devices.created_at IS 'When device was first registered';
COMMENT ON COLUMN devices.updated_at IS 'Last time device information was modified';

-- Indexes for device lookups and queries
CREATE INDEX idx_devices_user_id ON devices (user_id);
COMMENT ON INDEX idx_devices_user_id IS 'Find all devices belonging to a specific user';

CREATE INDEX idx_devices_identity_public_key ON devices (identity_public_key);
COMMENT ON INDEX idx_devices_identity_public_key IS 'Lookup device by identity public key for verification';

CREATE INDEX idx_devices_user_active ON devices (user_id, is_active);
COMMENT ON INDEX idx_devices_user_active IS 'Find active devices for a user for message delivery';

CREATE INDEX idx_devices_last_seen ON devices (last_seen_at);
COMMENT ON INDEX idx_devices_last_seen IS 'Query devices by last activity for presence and cleanup';

CREATE UNIQUE INDEX idx_devices_user_registration ON devices (user_id, registration_id);
COMMENT ON INDEX idx_devices_user_registration IS 'Ensure unique registration IDs per user for Signal Protocol compliance';