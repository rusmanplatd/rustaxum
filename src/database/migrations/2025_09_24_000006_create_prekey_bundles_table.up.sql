-- Create prekey bundles table for Signal Protocol key exchange
-- Stores public prekeys for each device to enable X3DH key agreement
CREATE TABLE prekey_bundles (
    id CHAR(26) PRIMARY KEY,
    device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,

    -- One-time prekeys (public keys only)
    prekey_id INTEGER NOT NULL,
    prekey_public TEXT NOT NULL,

    -- Key creation and usage tracking
    is_used BOOLEAN NOT NULL DEFAULT false,
    used_at TIMESTAMPTZ,
    used_by_user_id CHAR(26) REFERENCES sys_users(id),
    used_by_device_id CHAR(26) REFERENCES devices(id),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table comment for prekey_bundles
COMMENT ON TABLE prekey_bundles IS 'Stores one-time prekey bundles for Signal Protocol X3DH key agreement. Each device generates multiple prekeys that are consumed once for establishing initial secure sessions. Critical for forward secrecy and preventing replay attacks in E2EE initialization.';

-- Column comments explaining Signal Protocol prekey bundle usage
COMMENT ON COLUMN prekey_bundles.id IS 'ULID primary key uniquely identifying this prekey bundle';
COMMENT ON COLUMN prekey_bundles.device_id IS 'Foreign key to device that generated this prekey - each device maintains its own prekey pool for multi-device E2EE support';
COMMENT ON COLUMN prekey_bundles.user_id IS 'Foreign key to user owning the device - enables efficient prekey lookup when initiating conversations';
COMMENT ON COLUMN prekey_bundles.prekey_id IS 'Sequential identifier for this prekey within the device scope - used in X3DH protocol for key identification';
COMMENT ON COLUMN prekey_bundles.prekey_public IS 'Base64-encoded public key (Curve25519) for this one-time prekey - shared with other devices for X3DH key agreement. No private key stored server-side for security.';
COMMENT ON COLUMN prekey_bundles.is_used IS 'Whether this prekey has been consumed in a key exchange - once used, prekey is marked as consumed to prevent replay attacks and maintain forward secrecy';
COMMENT ON COLUMN prekey_bundles.used_at IS 'Timestamp when prekey was consumed in key exchange - helps track prekey lifecycle and rotation needs';
COMMENT ON COLUMN prekey_bundles.used_by_user_id IS 'User who consumed this prekey for key exchange - provides audit trail for security monitoring';
COMMENT ON COLUMN prekey_bundles.used_by_device_id IS 'Device that consumed this prekey for X3DH - enables tracking of which devices initiated sessions with each other';
COMMENT ON COLUMN prekey_bundles.created_at IS 'Prekey generation timestamp - used for prekey rotation and cleanup of old unused keys';
COMMENT ON COLUMN prekey_bundles.updated_at IS 'Last modification timestamp - updated when prekey is consumed';

-- Unique constraint for prekey ID per device
CREATE UNIQUE INDEX idx_prekey_bundles_device_key ON prekey_bundles (device_id, prekey_id);
COMMENT ON INDEX idx_prekey_bundles_device_key IS 'Enforces unique prekey IDs per device - prevents duplicate prekey generation and ensures proper X3DH protocol implementation';

-- Indexes for prekey queries
CREATE INDEX idx_prekey_bundles_device ON prekey_bundles (device_id);
COMMENT ON INDEX idx_prekey_bundles_device IS 'Optimizes device-specific prekey queries - used for prekey bundle distribution and device prekey pool management';

CREATE INDEX idx_prekey_bundles_user ON prekey_bundles (user_id);
COMMENT ON INDEX idx_prekey_bundles_user IS 'Optimizes user-level prekey queries - supports multi-device prekey lookup when initiating encrypted conversations';

CREATE INDEX idx_prekey_bundles_unused ON prekey_bundles (device_id, is_used) WHERE is_used = false;
COMMENT ON INDEX idx_prekey_bundles_unused IS 'Partial index for unused prekeys only - critical performance optimization for X3DH key exchange as only unused prekeys are eligible for consumption';

CREATE INDEX idx_prekey_bundles_used_by ON prekey_bundles (used_by_user_id, used_by_device_id);
COMMENT ON INDEX idx_prekey_bundles_used_by IS 'Optimizes queries for prekey usage audit trails - supports security monitoring and session establishment tracking';

CREATE INDEX idx_prekey_bundles_created ON prekey_bundles (created_at);
COMMENT ON INDEX idx_prekey_bundles_created IS 'Optimizes temporal queries for prekey lifecycle management - supports prekey rotation and cleanup of old unused keys';