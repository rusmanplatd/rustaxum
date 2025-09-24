-- Add Signal Protocol key rotation support to existing tables
-- Enhances devices table with automated key rotation tracking and scheduling
-- Implements proper key lifecycle management for forward secrecy

-- Add key rotation fields to devices table
ALTER TABLE devices ADD COLUMN signed_prekey_rotation_needed BOOLEAN NOT NULL DEFAULT false;
COMMENT ON COLUMN devices.signed_prekey_rotation_needed IS 'Flag indicating signed prekey needs rotation - triggers automated rotation process';

ALTER TABLE devices ADD COLUMN last_key_rotation_at TIMESTAMPTZ;
COMMENT ON COLUMN devices.last_key_rotation_at IS 'Timestamp of last key rotation - used to schedule periodic rotations for security';

ALTER TABLE devices ADD COLUMN prekey_rotation_interval INTERVAL NOT NULL DEFAULT INTERVAL '30 days';
COMMENT ON COLUMN devices.prekey_rotation_interval IS 'How often to rotate signed prekeys - configurable per device for security policies';

-- Create device key rotation schedule table
-- Manages automated key rotation for all device cryptographic materials
CREATE TABLE device_key_rotations (
    id CHAR(26) PRIMARY KEY,
    device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Type of key rotation being performed
    rotation_type VARCHAR NOT NULL CHECK (rotation_type IN ('signed_prekey', 'prekey_bundle', 'identity', 'registration_id')),

    -- Rotation scheduling and completion
    scheduled_at TIMESTAMPTZ NOT NULL,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    failure_reason TEXT,

    -- Status tracking
    is_completed BOOLEAN NOT NULL DEFAULT false,
    is_failed BOOLEAN NOT NULL DEFAULT false,

    -- Retry mechanism for failed rotations
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    next_retry_at TIMESTAMPTZ,

    -- Old and new key identifiers for audit trail
    old_key_id TEXT,
    new_key_id TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table and column comments for device_key_rotations
COMMENT ON TABLE device_key_rotations IS 'Manages automated key rotation schedule for Signal Protocol devices - ensures forward secrecy through periodic key updates';
COMMENT ON COLUMN device_key_rotations.id IS 'Unique rotation job identifier (ULID format)';
COMMENT ON COLUMN device_key_rotations.device_id IS 'Device undergoing key rotation - cascade deletes if device is removed';
COMMENT ON COLUMN device_key_rotations.rotation_type IS 'Type of key material being rotated: signed_prekey (most common), prekey_bundle (refill), identity (rare), registration_id (device reset)';
COMMENT ON COLUMN device_key_rotations.scheduled_at IS 'When rotation should begin - based on key age and security policy';
COMMENT ON COLUMN device_key_rotations.started_at IS 'When rotation process actually began - may differ from scheduled due to system load';
COMMENT ON COLUMN device_key_rotations.completed_at IS 'When rotation successfully completed - new keys are active';
COMMENT ON COLUMN device_key_rotations.failed_at IS 'When rotation failed - triggers retry mechanism';
COMMENT ON COLUMN device_key_rotations.failure_reason IS 'Detailed failure reason for debugging and monitoring';
COMMENT ON COLUMN device_key_rotations.is_completed IS 'Whether rotation completed successfully';
COMMENT ON COLUMN device_key_rotations.is_failed IS 'Whether rotation failed permanently (exceeded max retries)';
COMMENT ON COLUMN device_key_rotations.retry_count IS 'Number of retry attempts made - prevents infinite retry loops';
COMMENT ON COLUMN device_key_rotations.max_retries IS 'Maximum retry attempts before marking as permanently failed';
COMMENT ON COLUMN device_key_rotations.next_retry_at IS 'When next retry attempt should be made - implements exponential backoff';
COMMENT ON COLUMN device_key_rotations.old_key_id IS 'Identifier of key being replaced - for audit and rollback purposes';
COMMENT ON COLUMN device_key_rotations.new_key_id IS 'Identifier of new key after rotation - for verification';

-- Indexes for key rotation queries
CREATE INDEX idx_device_key_rotations_device ON device_key_rotations (device_id);
COMMENT ON INDEX idx_device_key_rotations_device IS 'Find all rotation jobs for a specific device';

CREATE INDEX idx_device_key_rotations_scheduled ON device_key_rotations (scheduled_at, is_completed, is_failed) WHERE is_completed = false AND is_failed = false;
COMMENT ON INDEX idx_device_key_rotations_scheduled IS 'Efficiently find pending rotations ready for processing - partial index on pending only';

CREATE INDEX idx_device_key_rotations_retry ON device_key_rotations (next_retry_at, retry_count, max_retries) WHERE is_failed = false AND retry_count < max_retries;
COMMENT ON INDEX idx_device_key_rotations_retry IS 'Optimize retry processing for failed rotations within retry limits';

CREATE INDEX idx_device_key_rotations_type ON device_key_rotations (rotation_type, scheduled_at);
COMMENT ON INDEX idx_device_key_rotations_type IS 'Process rotations by type with scheduling order for batch operations';

-- Update devices table indexes for key rotation
CREATE INDEX idx_devices_rotation_needed ON devices (signed_prekey_rotation_needed, last_key_rotation_at) WHERE signed_prekey_rotation_needed = true;
COMMENT ON INDEX idx_devices_rotation_needed IS 'Efficiently find devices needing key rotation - partial index on flagged devices';

CREATE INDEX idx_devices_last_rotation ON devices (last_key_rotation_at, prekey_rotation_interval);
COMMENT ON INDEX idx_devices_last_rotation IS 'Optimize queries for devices due for scheduled key rotation based on interval';