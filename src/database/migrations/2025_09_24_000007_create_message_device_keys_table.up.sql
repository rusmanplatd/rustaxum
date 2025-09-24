-- Create message device keys table for per-device encrypted message keys
-- Each message needs to be encrypted separately for each recipient device
CREATE TABLE message_device_keys (
    id CHAR(26) PRIMARY KEY,
    message_id CHAR(26) NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    recipient_device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Encrypted message key for this specific device
    -- The message content key is encrypted with the recipient device's session key
    encrypted_message_key TEXT NOT NULL,
    key_algorithm VARCHAR NOT NULL,

    -- Delivery status for this device
    delivered_at TIMESTAMPTZ,
    read_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table comment for message_device_keys
COMMENT ON TABLE message_device_keys IS 'Stores per-device encrypted message keys for E2EE multi-device support. Each message has one content encryption key that is individually encrypted for each recipient device using their session key. Enables efficient message distribution while maintaining forward secrecy across all devices.';

-- Column comments explaining per-device key distribution
COMMENT ON COLUMN message_device_keys.id IS 'ULID primary key uniquely identifying this device-specific message key';
COMMENT ON COLUMN message_device_keys.message_id IS 'Foreign key to the encrypted message - one message can have multiple device keys for multi-device delivery';
COMMENT ON COLUMN message_device_keys.recipient_device_id IS 'Foreign key to specific recipient device - each device gets its own encrypted copy of the message key';
COMMENT ON COLUMN message_device_keys.encrypted_message_key IS 'Message content key encrypted with recipient device session key - allows device to decrypt the actual message content. Uses Double Ratchet derived keys for forward secrecy.';
COMMENT ON COLUMN message_device_keys.key_algorithm IS 'Algorithm used to encrypt the message key for this device (e.g., AES-256-GCM) - may vary per device based on negotiated capabilities';
COMMENT ON COLUMN message_device_keys.delivered_at IS 'Timestamp when encrypted key was successfully delivered to recipient device - supports delivery confirmation tracking';
COMMENT ON COLUMN message_device_keys.read_at IS 'Timestamp when message was read/decrypted by recipient device - enables read receipt functionality without exposing reading patterns';
COMMENT ON COLUMN message_device_keys.created_at IS 'Key creation timestamp - used for message key lifecycle management and cleanup';

-- Unique constraint: one encrypted key per message per device
CREATE UNIQUE INDEX idx_message_device_keys_unique ON message_device_keys (message_id, recipient_device_id);
COMMENT ON INDEX idx_message_device_keys_unique IS 'Enforces one encrypted key per message per device - prevents duplicate key distribution and ensures E2EE integrity in multi-device scenarios';

-- Indexes for key queries
CREATE INDEX idx_message_device_keys_message ON message_device_keys (message_id);
COMMENT ON INDEX idx_message_device_keys_message IS 'Optimizes queries for all device keys of a message - used for multi-device message distribution and delivery status tracking';

CREATE INDEX idx_message_device_keys_device ON message_device_keys (recipient_device_id);
COMMENT ON INDEX idx_message_device_keys_device IS 'Optimizes device-centric queries for message keys - supports message synchronization and delivery confirmation for specific devices';

CREATE INDEX idx_message_device_keys_delivered ON message_device_keys (recipient_device_id, delivered_at);
COMMENT ON INDEX idx_message_device_keys_delivered IS 'Optimizes delivery status queries per device - enables efficient tracking of message delivery confirmation across multi-device E2EE';

CREATE INDEX idx_message_device_keys_read ON message_device_keys (recipient_device_id, read_at);
COMMENT ON INDEX idx_message_device_keys_read IS 'Optimizes read receipt queries per device - supports read status tracking without exposing reading patterns to unauthorized parties';