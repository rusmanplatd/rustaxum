-- Create security and backup tables for E2EE
CREATE TABLE device_verification_codes (
    id CHAR(26) PRIMARY KEY,
    device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    verifying_device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Safety numbers for device verification
    safety_number VARCHAR(60) NOT NULL, -- 60-digit safety number
    verification_method VARCHAR NOT NULL CHECK (verification_method IN ('qr_code', 'manual_compare', 'voice_call')),

    -- Verification status
    is_verified BOOLEAN NOT NULL DEFAULT false,
    verified_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '24 hours'),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create backup keys table for encrypted backups
CREATE TABLE encrypted_backup_keys (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Encrypted backup data (session state, message keys, etc.)
    encrypted_backup_data TEXT NOT NULL,
    backup_algorithm VARCHAR NOT NULL,

    -- Backup metadata
    backup_type VARCHAR NOT NULL CHECK (backup_type IN ('full', 'incremental', 'keys_only')),
    backup_size_bytes BIGINT NOT NULL,

    -- Backup verification
    backup_hash VARCHAR(64) NOT NULL, -- SHA-256 hash for integrity
    is_verified BOOLEAN NOT NULL DEFAULT false,

    -- Expiration and cleanup
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days'),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create security incidents table for monitoring
CREATE TABLE security_incidents (
    id CHAR(26) PRIMARY KEY,
    device_id CHAR(26) REFERENCES devices(id) ON DELETE SET NULL,
    user_id CHAR(26) REFERENCES sys_users(id) ON DELETE SET NULL,
    conversation_id CHAR(26) REFERENCES conversations(id) ON DELETE SET NULL,

    -- Incident details
    incident_type VARCHAR NOT NULL CHECK (incident_type IN (
        'key_compromise', 'session_reset', 'device_unauthorized',
        'decryption_failure', 'verification_failure', 'replay_attack_detected',
        'algorithm_downgrade_attempt', 'multiple_failed_auth'
    )),
    severity VARCHAR NOT NULL DEFAULT 'medium' CHECK (severity IN ('low', 'medium', 'high', 'critical')),

    -- Incident data (encrypted)
    encrypted_incident_data TEXT,
    incident_algorithm VARCHAR,

    -- Resolution status
    is_resolved BOOLEAN NOT NULL DEFAULT false,
    resolved_at TIMESTAMPTZ,
    resolution_notes TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create message expiry cleanup table
CREATE TABLE message_expiry_queue (
    id CHAR(26) PRIMARY KEY,
    message_id CHAR(26) NOT NULL REFERENCES messages(id) ON DELETE CASCADE,

    -- Expiry details
    expires_at TIMESTAMPTZ NOT NULL,
    expiry_type VARCHAR NOT NULL CHECK (expiry_type IN ('disappearing', 'scheduled_delete', 'retention_policy')),

    -- Processing status
    is_processed BOOLEAN NOT NULL DEFAULT false,
    processed_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Unique constraints
CREATE UNIQUE INDEX idx_device_verification_codes_pair ON device_verification_codes (device_id, verifying_device_id);
CREATE UNIQUE INDEX idx_message_expiry_queue_message ON message_expiry_queue (message_id);

-- Indexes for verification codes
CREATE INDEX idx_device_verification_codes_device ON device_verification_codes (device_id);
CREATE INDEX idx_device_verification_codes_verifying ON device_verification_codes (verifying_device_id);
CREATE INDEX idx_device_verification_codes_verified ON device_verification_codes (is_verified, verified_at);
CREATE INDEX idx_device_verification_codes_expires ON device_verification_codes (expires_at);

-- Indexes for backup keys
CREATE INDEX idx_encrypted_backup_keys_user ON encrypted_backup_keys (user_id);
CREATE INDEX idx_encrypted_backup_keys_device ON encrypted_backup_keys (device_id);
CREATE INDEX idx_encrypted_backup_keys_type ON encrypted_backup_keys (backup_type);
CREATE INDEX idx_encrypted_backup_keys_expires ON encrypted_backup_keys (expires_at);

-- Indexes for security incidents
CREATE INDEX idx_security_incidents_device ON security_incidents (device_id);
CREATE INDEX idx_security_incidents_user ON security_incidents (user_id);
CREATE INDEX idx_security_incidents_conversation ON security_incidents (conversation_id);
CREATE INDEX idx_security_incidents_type ON security_incidents (incident_type, severity);
CREATE INDEX idx_security_incidents_unresolved ON security_incidents (is_resolved, created_at) WHERE is_resolved = false;

-- Indexes for message expiry
CREATE INDEX idx_message_expiry_queue_expires ON message_expiry_queue (expires_at, is_processed) WHERE is_processed = false;
CREATE INDEX idx_message_expiry_queue_type ON message_expiry_queue (expiry_type);