-- Add CHECK constraints for algorithm validation and compatibility
-- Ensures only supported algorithm combinations are stored in the database
-- Prevents invalid algorithm configurations that could break E2EE functionality

-- Add CHECK constraints to device_algorithm_preferences for valid algorithms
ALTER TABLE device_algorithm_preferences ADD CONSTRAINT check_valid_encryption_algorithms
CHECK (
    preferred_encryption_algorithms <@ ARRAY['aes-256-gcm', 'chacha20-poly1305', 'aes-128-gcm']::TEXT[]
);
COMMENT ON CONSTRAINT check_valid_encryption_algorithms ON device_algorithm_preferences IS 'Ensures only supported encryption algorithms are specified in preferences';

ALTER TABLE device_algorithm_preferences ADD CONSTRAINT check_valid_key_exchange_algorithms
CHECK (
    preferred_key_exchange_algorithms <@ ARRAY['curve25519', 'p-256', 'rsa-2048', 'rsa-4096']::TEXT[]
);
COMMENT ON CONSTRAINT check_valid_key_exchange_algorithms ON device_algorithm_preferences IS 'Ensures only supported key exchange algorithms are specified in preferences';

ALTER TABLE device_algorithm_preferences ADD CONSTRAINT check_valid_mac_algorithms
CHECK (
    preferred_mac_algorithms <@ ARRAY['hmac-sha256', 'hmac-sha384', 'hmac-sha512', 'blake3']::TEXT[]
);
COMMENT ON CONSTRAINT check_valid_mac_algorithms ON device_algorithm_preferences IS 'Ensures only supported MAC algorithms are specified in preferences';

ALTER TABLE device_algorithm_preferences ADD CONSTRAINT check_valid_pq_kem_algorithms
CHECK (
    preferred_pq_kem_algorithms <@ ARRAY['kyber-768', 'bike-r4']::TEXT[]
);
COMMENT ON CONSTRAINT check_valid_pq_kem_algorithms ON device_algorithm_preferences IS 'Ensures only supported post-quantum key encapsulation algorithms are specified';

ALTER TABLE device_algorithm_preferences ADD CONSTRAINT check_valid_pq_signature_algorithms
CHECK (
    preferred_pq_signature_algorithms <@ ARRAY['dilithium2', 'sphincs-plus']::TEXT[]
);
COMMENT ON CONSTRAINT check_valid_pq_signature_algorithms ON device_algorithm_preferences IS 'Ensures only supported post-quantum signature algorithms are specified';

-- Add CHECK constraints to conversation_algorithm_negotiations for negotiated algorithms
ALTER TABLE conversation_algorithm_negotiations ADD CONSTRAINT check_negotiated_encryption_algorithm
CHECK (
    negotiated_encryption_algorithm IN ('aes-256-gcm', 'chacha20-poly1305', 'aes-128-gcm')
);
COMMENT ON CONSTRAINT check_negotiated_encryption_algorithm ON conversation_algorithm_negotiations IS 'Ensures negotiated encryption algorithm is supported';

ALTER TABLE conversation_algorithm_negotiations ADD CONSTRAINT check_negotiated_key_exchange
CHECK (
    negotiated_key_exchange IN ('curve25519', 'p-256', 'rsa-2048', 'rsa-4096')
);
COMMENT ON CONSTRAINT check_negotiated_key_exchange ON conversation_algorithm_negotiations IS 'Ensures negotiated key exchange algorithm is supported';

ALTER TABLE conversation_algorithm_negotiations ADD CONSTRAINT check_negotiated_mac_algorithm
CHECK (
    negotiated_mac_algorithm IN ('hmac-sha256', 'hmac-sha384', 'hmac-sha512', 'blake3')
);
COMMENT ON CONSTRAINT check_negotiated_mac_algorithm ON conversation_algorithm_negotiations IS 'Ensures negotiated MAC algorithm is supported';

ALTER TABLE conversation_algorithm_negotiations ADD CONSTRAINT check_negotiated_signature_algorithm
CHECK (
    negotiated_signature_algorithm IS NULL OR
    negotiated_signature_algorithm IN ('ed25519', 'rsa-pss', 'ecdsa-p256')
);
COMMENT ON CONSTRAINT check_negotiated_signature_algorithm ON conversation_algorithm_negotiations IS 'Ensures negotiated signature algorithm is supported (nullable for backward compatibility)';

ALTER TABLE conversation_algorithm_negotiations ADD CONSTRAINT check_negotiated_kdf_algorithm
CHECK (
    negotiated_kdf_algorithm IS NULL OR
    negotiated_kdf_algorithm IN ('hkdf-sha256', 'hkdf-sha384', 'hkdf-sha512')
);
COMMENT ON CONSTRAINT check_negotiated_kdf_algorithm ON conversation_algorithm_negotiations IS 'Ensures negotiated KDF algorithm is supported (nullable for backward compatibility)';

ALTER TABLE conversation_algorithm_negotiations ADD CONSTRAINT check_negotiated_pq_kem_algorithm
CHECK (
    negotiated_pq_kem_algorithm IS NULL OR
    negotiated_pq_kem_algorithm IN ('kyber-768', 'bike-r4')
);
COMMENT ON CONSTRAINT check_negotiated_pq_kem_algorithm ON conversation_algorithm_negotiations IS 'Ensures negotiated post-quantum KEM algorithm is supported (nullable for non-PQ conversations)';

ALTER TABLE conversation_algorithm_negotiations ADD CONSTRAINT check_negotiated_pq_signature_algorithm
CHECK (
    negotiated_pq_signature_algorithm IS NULL OR
    negotiated_pq_signature_algorithm IN ('dilithium2', 'sphincs-plus')
);
COMMENT ON CONSTRAINT check_negotiated_pq_signature_algorithm ON conversation_algorithm_negotiations IS 'Ensures negotiated post-quantum signature algorithm is supported (nullable for non-PQ conversations)';

-- Add CHECK constraints for session and key algorithms
ALTER TABLE signal_sessions ADD CONSTRAINT check_session_algorithm
CHECK (
    session_algorithm IN ('aes-256-gcm', 'chacha20-poly1305', 'aes-128-gcm')
);
COMMENT ON CONSTRAINT check_session_algorithm ON signal_sessions IS 'Ensures session state encryption uses supported algorithms';

ALTER TABLE sender_key_sessions ADD CONSTRAINT check_key_algorithm
CHECK (
    key_algorithm IN ('aes-256-gcm', 'chacha20-poly1305', 'aes-128-gcm')
);
COMMENT ON CONSTRAINT check_key_algorithm ON sender_key_sessions IS 'Ensures sender key encryption uses supported algorithms';

ALTER TABLE skipped_message_keys ADD CONSTRAINT check_key_algorithm_skipped
CHECK (
    key_algorithm IN ('aes-256-gcm', 'chacha20-poly1305', 'aes-128-gcm')
);
COMMENT ON CONSTRAINT check_key_algorithm_skipped ON skipped_message_keys IS 'Ensures skipped message key encryption uses supported algorithms';

ALTER TABLE message_key_pools ADD CONSTRAINT check_pool_algorithm
CHECK (
    pool_algorithm IN ('aes-256-gcm', 'chacha20-poly1305', 'aes-128-gcm')
);
COMMENT ON CONSTRAINT check_pool_algorithm ON message_key_pools IS 'Ensures message key pool encryption uses supported algorithms';

-- Add CHECK constraints for backup and recovery algorithms
ALTER TABLE device_session_backups ADD CONSTRAINT check_backup_algorithm
CHECK (
    backup_algorithm IN ('aes-256-gcm', 'chacha20-poly1305', 'aes-128-gcm')
);
COMMENT ON CONSTRAINT check_backup_algorithm ON device_session_backups IS 'Ensures session backup encryption uses supported algorithms';

-- Add CHECK constraints for device trust levels and algorithm compatibility
ALTER TABLE devices ADD COLUMN trust_level VARCHAR NOT NULL DEFAULT 'unverified' CHECK (trust_level IN ('unverified', 'verified', 'trusted'));
COMMENT ON COLUMN devices.trust_level IS 'Device trust level: unverified (new device), verified (identity confirmed), trusted (long-term verified)';

-- Add CHECK constraint for supported algorithms array validity
ALTER TABLE devices ADD CONSTRAINT check_supported_algorithms_valid
CHECK (
    supported_algorithms <@ ARRAY['aes-256-gcm', 'chacha20-poly1305', 'aes-128-gcm', 'curve25519', 'p-256', 'rsa-2048', 'rsa-4096', 'hmac-sha256', 'hmac-sha384', 'hmac-sha512', 'blake3', 'ed25519', 'rsa-pss', 'ecdsa-p256', 'hkdf-sha256', 'hkdf-sha384', 'hkdf-sha512', 'kyber-768', 'bike-r4', 'dilithium2', 'sphincs-plus']::TEXT[]
);
COMMENT ON CONSTRAINT check_supported_algorithms_valid ON devices IS 'Ensures devices only claim support for recognized algorithms';

-- Add index for device trust levels
CREATE INDEX idx_devices_trust_level ON devices (trust_level, is_active);
COMMENT ON INDEX idx_devices_trust_level IS 'Optimize queries for devices by trust level and activity status';