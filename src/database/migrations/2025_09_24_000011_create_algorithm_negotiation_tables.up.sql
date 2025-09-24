-- Create algorithm negotiation tables for E2EE compatibility
CREATE TABLE device_capabilities (
    id CHAR(26) PRIMARY KEY,
    device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Supported encryption algorithms
    supports_aes_256_gcm BOOLEAN NOT NULL DEFAULT true,
    supports_chacha20_poly1305 BOOLEAN NOT NULL DEFAULT false,
    supports_aes_128_gcm BOOLEAN NOT NULL DEFAULT false,

    -- Supported key exchange methods
    supports_curve25519 BOOLEAN NOT NULL DEFAULT true,
    supports_p256_ecdh BOOLEAN NOT NULL DEFAULT false,
    supports_rsa_2048 BOOLEAN NOT NULL DEFAULT false,
    supports_rsa_4096 BOOLEAN NOT NULL DEFAULT false,

    -- Supported MAC algorithms
    supports_hmac_sha256 BOOLEAN NOT NULL DEFAULT true,
    supports_hmac_sha384 BOOLEAN NOT NULL DEFAULT false,
    supports_hmac_sha512 BOOLEAN NOT NULL DEFAULT false,
    supports_blake3_mac BOOLEAN NOT NULL DEFAULT false,

    -- Protocol versions supported
    max_signal_protocol_version INTEGER NOT NULL DEFAULT 3,
    min_signal_protocol_version INTEGER NOT NULL DEFAULT 3,

    -- Feature capabilities
    supports_multi_device BOOLEAN NOT NULL DEFAULT true,
    supports_group_messaging BOOLEAN NOT NULL DEFAULT true,
    supports_disappearing_messages BOOLEAN NOT NULL DEFAULT true,
    supports_file_encryption BOOLEAN NOT NULL DEFAULT true,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create conversation algorithm negotiations
CREATE TABLE conversation_algorithm_negotiations (
    id CHAR(26) PRIMARY KEY,
    conversation_id CHAR(26) NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,

    -- Negotiated algorithms (best common algorithms)
    negotiated_encryption_algorithm VARCHAR NOT NULL,
    negotiated_key_exchange VARCHAR NOT NULL,
    negotiated_mac_algorithm VARCHAR NOT NULL,
    negotiated_protocol_version INTEGER NOT NULL,

    -- Negotiation metadata
    negotiation_started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    negotiation_completed_at TIMESTAMPTZ,
    is_negotiation_complete BOOLEAN NOT NULL DEFAULT false,

    -- Participants in negotiation
    negotiation_participants_count INTEGER NOT NULL,
    all_participants_responded BOOLEAN NOT NULL DEFAULT false,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create device algorithm preferences
CREATE TABLE device_algorithm_preferences (
    id CHAR(26) PRIMARY KEY,
    device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Preferred algorithms (ordered by preference)
    preferred_encryption_algorithms TEXT[] NOT NULL DEFAULT '{aes-256-gcm,chacha20-poly1305,aes-128-gcm}',
    preferred_key_exchange_algorithms TEXT[] NOT NULL DEFAULT '{curve25519,p-256,rsa-2048}',
    preferred_mac_algorithms TEXT[] NOT NULL DEFAULT '{hmac-sha256,hmac-sha384,blake3}',

    -- Fallback settings
    allow_algorithm_fallback BOOLEAN NOT NULL DEFAULT true,
    minimum_security_level VARCHAR NOT NULL DEFAULT 'high' CHECK (minimum_security_level IN ('low', 'medium', 'high', 'maximum')),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Unique constraints
CREATE UNIQUE INDEX idx_device_capabilities_device ON device_capabilities (device_id);
CREATE UNIQUE INDEX idx_conversation_algorithm_negotiations_conversation ON conversation_algorithm_negotiations (conversation_id);
CREATE UNIQUE INDEX idx_device_algorithm_preferences_device ON device_algorithm_preferences (device_id);

-- Indexes for capability queries
CREATE INDEX idx_device_capabilities_encryption ON device_capabilities (supports_aes_256_gcm, supports_chacha20_poly1305);
CREATE INDEX idx_device_capabilities_key_exchange ON device_capabilities (supports_curve25519, supports_p256_ecdh);
CREATE INDEX idx_device_capabilities_protocol_version ON device_capabilities (max_signal_protocol_version, min_signal_protocol_version);

-- Indexes for negotiation queries
CREATE INDEX idx_conversation_algorithm_negotiations_complete ON conversation_algorithm_negotiations (is_negotiation_complete);
CREATE INDEX idx_conversation_algorithm_negotiations_started ON conversation_algorithm_negotiations (negotiation_started_at);

-- Indexes for preferences
CREATE INDEX idx_device_algorithm_preferences_fallback ON device_algorithm_preferences (allow_algorithm_fallback);
CREATE INDEX idx_device_algorithm_preferences_security ON device_algorithm_preferences (minimum_security_level);