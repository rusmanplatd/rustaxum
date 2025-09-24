-- Add missing E2EE components identified in detailed review
-- Includes message key garbage collection, algorithm compatibility matrix, and enhanced security features

-- Create algorithm compatibility matrix for cross-algorithm compatibility tracking
CREATE TABLE algorithm_compatibility_matrix (
    id CHAR(26) PRIMARY KEY,

    -- Algorithm pair being tested for compatibility
    encryption_algorithm_a VARCHAR NOT NULL,
    encryption_algorithm_b VARCHAR NOT NULL,
    key_exchange_algorithm_a VARCHAR NOT NULL,
    key_exchange_algorithm_b VARCHAR NOT NULL,

    -- Compatibility status
    is_compatible BOOLEAN NOT NULL,
    compatibility_level VARCHAR NOT NULL CHECK (compatibility_level IN ('full', 'limited', 'none')) DEFAULT 'none',

    -- Performance metrics
    negotiation_overhead_ms INTEGER,
    interop_test_passed BOOLEAN DEFAULT false,

    -- Metadata
    tested_at TIMESTAMPTZ,
    test_version VARCHAR NOT NULL DEFAULT '1.0',
    notes TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE algorithm_compatibility_matrix IS 'Tracks compatibility between different algorithm combinations for multi-algorithm E2EE support';
COMMENT ON COLUMN algorithm_compatibility_matrix.encryption_algorithm_a IS 'First encryption algorithm in compatibility test pair';
COMMENT ON COLUMN algorithm_compatibility_matrix.encryption_algorithm_b IS 'Second encryption algorithm in compatibility test pair';
COMMENT ON COLUMN algorithm_compatibility_matrix.key_exchange_algorithm_a IS 'First key exchange algorithm in compatibility test pair';
COMMENT ON COLUMN algorithm_compatibility_matrix.key_exchange_algorithm_b IS 'Second key exchange algorithm in compatibility test pair';
COMMENT ON COLUMN algorithm_compatibility_matrix.is_compatible IS 'Whether the algorithm pair is compatible for interoperability';
COMMENT ON COLUMN algorithm_compatibility_matrix.compatibility_level IS 'Level of compatibility: full (seamless), limited (with constraints), none (incompatible)';
COMMENT ON COLUMN algorithm_compatibility_matrix.negotiation_overhead_ms IS 'Additional negotiation time required for this algorithm pair';
COMMENT ON COLUMN algorithm_compatibility_matrix.interop_test_passed IS 'Whether interoperability tests passed for this pair';
COMMENT ON COLUMN algorithm_compatibility_matrix.tested_at IS 'When compatibility was last tested';
COMMENT ON COLUMN algorithm_compatibility_matrix.test_version IS 'Version of compatibility test suite used';
COMMENT ON COLUMN algorithm_compatibility_matrix.notes IS 'Additional notes about compatibility constraints or requirements';

-- Create message key garbage collection policies
CREATE TABLE message_key_gc_policies (
    id CHAR(26) PRIMARY KEY,

    -- Policy scope
    policy_name VARCHAR NOT NULL UNIQUE,
    applies_to_table VARCHAR NOT NULL CHECK (applies_to_table IN ('skipped_message_keys', 'message_key_pools', 'signal_sessions', 'sender_key_sessions')),

    -- Cleanup rules
    max_age_days INTEGER NOT NULL DEFAULT 7,
    max_unused_keys INTEGER DEFAULT 1000,
    cleanup_frequency_hours INTEGER NOT NULL DEFAULT 24,

    -- Conditions for cleanup
    cleanup_condition TEXT NOT NULL,
    preserve_condition TEXT,

    -- Policy status
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE message_key_gc_policies IS 'Defines garbage collection policies for E2EE message keys and cryptographic materials';
COMMENT ON COLUMN message_key_gc_policies.policy_name IS 'Human-readable name for this garbage collection policy';
COMMENT ON COLUMN message_key_gc_policies.applies_to_table IS 'Database table this policy applies to for key cleanup';
COMMENT ON COLUMN message_key_gc_policies.max_age_days IS 'Maximum age in days before keys become eligible for cleanup';
COMMENT ON COLUMN message_key_gc_policies.max_unused_keys IS 'Maximum number of unused keys to retain per session/device';
COMMENT ON COLUMN message_key_gc_policies.cleanup_frequency_hours IS 'How often to run cleanup for this policy';
COMMENT ON COLUMN message_key_gc_policies.cleanup_condition IS 'SQL condition for identifying keys eligible for cleanup';
COMMENT ON COLUMN message_key_gc_policies.preserve_condition IS 'SQL condition for keys that should never be cleaned up';
COMMENT ON COLUMN message_key_gc_policies.is_active IS 'Whether this cleanup policy is currently active';
COMMENT ON COLUMN message_key_gc_policies.last_run_at IS 'When cleanup was last executed for this policy';
COMMENT ON COLUMN message_key_gc_policies.next_run_at IS 'When cleanup should next be executed';

-- Create device fingerprint verification for enhanced security
CREATE TABLE device_fingerprints (
    id CHAR(26) PRIMARY KEY,
    device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Fingerprint data
    identity_key_fingerprint TEXT NOT NULL,
    fingerprint_algorithm VARCHAR NOT NULL DEFAULT 'sha256' CHECK (fingerprint_algorithm IN ('sha256', 'sha384', 'sha512')),

    -- Verification status
    is_verified BOOLEAN NOT NULL DEFAULT false,
    verified_by_user_id CHAR(26) REFERENCES sys_users(id),
    verified_at TIMESTAMPTZ,
    verification_method VARCHAR CHECK (verification_method IN ('qr_code', 'safety_numbers', 'voice_call', 'in_person')),

    -- Trust level
    trust_score INTEGER NOT NULL DEFAULT 0 CHECK (trust_score >= 0 AND trust_score <= 100),
    trust_last_updated TIMESTAMPTZ DEFAULT NOW(),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE device_fingerprints IS 'Stores device identity fingerprints for manual verification and trust establishment';
COMMENT ON COLUMN device_fingerprints.identity_key_fingerprint IS 'Cryptographic fingerprint of device identity key for manual verification';
COMMENT ON COLUMN device_fingerprints.fingerprint_algorithm IS 'Hash algorithm used to generate fingerprint';
COMMENT ON COLUMN device_fingerprints.is_verified IS 'Whether fingerprint has been manually verified by user';
COMMENT ON COLUMN device_fingerprints.verified_by_user_id IS 'User who performed the verification';
COMMENT ON COLUMN device_fingerprints.verified_at IS 'When verification was completed';
COMMENT ON COLUMN device_fingerprints.verification_method IS 'Method used for verification (QR code, safety numbers, etc.)';
COMMENT ON COLUMN device_fingerprints.trust_score IS 'Computed trust score based on verification method and history';
COMMENT ON COLUMN device_fingerprints.trust_last_updated IS 'When trust score was last recalculated';

-- Add message key cleanup tracking
ALTER TABLE skipped_message_keys ADD COLUMN gc_eligible_at TIMESTAMPTZ DEFAULT NOW() + INTERVAL '7 days';
COMMENT ON COLUMN skipped_message_keys.gc_eligible_at IS 'When this key becomes eligible for garbage collection';

ALTER TABLE message_key_pools ADD COLUMN gc_eligible_at TIMESTAMPTZ DEFAULT NOW() + INTERVAL '7 days';
COMMENT ON COLUMN message_key_pools.gc_eligible_at IS 'When this key pool becomes eligible for garbage collection';

-- Create indexes for new tables
CREATE UNIQUE INDEX idx_algorithm_compatibility_matrix_pair ON algorithm_compatibility_matrix (encryption_algorithm_a, encryption_algorithm_b, key_exchange_algorithm_a, key_exchange_algorithm_b);
COMMENT ON INDEX idx_algorithm_compatibility_matrix_pair IS 'Ensures unique compatibility entries per algorithm pair';

CREATE INDEX idx_algorithm_compatibility_matrix_compatible ON algorithm_compatibility_matrix (is_compatible, compatibility_level);
COMMENT ON INDEX idx_algorithm_compatibility_matrix_compatible IS 'Optimize queries for compatible algorithm pairs';

CREATE INDEX idx_message_key_gc_policies_active ON message_key_gc_policies (is_active, next_run_at) WHERE is_active = true;
COMMENT ON INDEX idx_message_key_gc_policies_active IS 'Optimize queries for active policies ready to run';

CREATE UNIQUE INDEX idx_device_fingerprints_device ON device_fingerprints (device_id);
COMMENT ON INDEX idx_device_fingerprints_device IS 'Ensure one fingerprint per device';

CREATE INDEX idx_device_fingerprints_verified ON device_fingerprints (is_verified, trust_score DESC);
COMMENT ON INDEX idx_device_fingerprints_verified IS 'Optimize queries for verified devices by trust score';

-- Add indexes for garbage collection
CREATE INDEX idx_skipped_message_keys_gc ON skipped_message_keys (gc_eligible_at) WHERE is_used = false;
COMMENT ON INDEX idx_skipped_message_keys_gc IS 'Optimize garbage collection queries for unused skipped keys';

CREATE INDEX idx_message_key_pools_gc ON message_key_pools (gc_eligible_at) WHERE is_pool_exhausted = false;
COMMENT ON INDEX idx_message_key_pools_gc IS 'Optimize garbage collection queries for non-exhausted key pools';

-- Insert default garbage collection policies
INSERT INTO message_key_gc_policies (id, policy_name, applies_to_table, max_age_days, cleanup_frequency_hours, cleanup_condition, preserve_condition) VALUES
('01JCQM5X6Y7Z8A9B0C1D2E3F4G', 'skipped_keys_weekly', 'skipped_message_keys', 7, 168, 'is_used = false AND gc_eligible_at < NOW()', 'created_at > NOW() - INTERVAL ''24 hours'''),
('01JCQM5X6Y7Z8A9B0C1D2E3F4H', 'key_pools_weekly', 'message_key_pools', 7, 168, 'is_pool_exhausted = true AND gc_eligible_at < NOW()', 'created_at > NOW() - INTERVAL ''24 hours'''),
('01JCQM5X6Y7Z8A9B0C1D2E3F4J', 'sessions_monthly', 'signal_sessions', 30, 720, 'is_active = false AND last_used_at < NOW() - INTERVAL ''30 days''', 'last_used_at > NOW() - INTERVAL ''7 days''');

-- Insert common algorithm compatibility entries
INSERT INTO algorithm_compatibility_matrix (id, encryption_algorithm_a, encryption_algorithm_b, key_exchange_algorithm_a, key_exchange_algorithm_b, is_compatible, compatibility_level, test_version) VALUES
('01JCQM5X6Y7Z8A9B0C1D2E3F4K', 'aes-256-gcm', 'chacha20-poly1305', 'curve25519', 'curve25519', true, 'full', '1.0'),
('01JCQM5X6Y7Z8A9B0C1D2E3F4L', 'aes-256-gcm', 'aes-128-gcm', 'curve25519', 'p-256', true, 'limited', '1.0'),
('01JCQM5X6Y7Z8A9B0C1D2E3F4M', 'chacha20-poly1305', 'aes-128-gcm', 'curve25519', 'rsa-2048', false, 'none', '1.0');