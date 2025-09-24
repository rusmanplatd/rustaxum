-- Enhance algorithm preferences with signature and KDF algorithms
-- Add missing algorithm types for comprehensive E2EE support
-- Extends device capabilities and negotiations to include digital signatures and key derivation

-- Add signature algorithm preferences to device_algorithm_preferences
-- Digital signatures are crucial for message authentication and key verification in E2EE
ALTER TABLE device_algorithm_preferences
ADD COLUMN preferred_signature_algorithms TEXT[] NOT NULL DEFAULT '{ed25519,rsa-pss,ecdsa-p256}';
COMMENT ON COLUMN device_algorithm_preferences.preferred_signature_algorithms IS 'Ordered list of preferred digital signature algorithms for message authentication and identity verification (Ed25519, RSA-PSS, ECDSA-P256)';

-- Add key derivation function preferences
-- KDFs are essential for deriving encryption keys from shared secrets in E2EE protocols
ALTER TABLE device_algorithm_preferences
ADD COLUMN preferred_kdf_algorithms TEXT[] NOT NULL DEFAULT '{hkdf-sha256,hkdf-sha384,hkdf-sha512}';
COMMENT ON COLUMN device_algorithm_preferences.preferred_kdf_algorithms IS 'Ordered list of preferred key derivation functions for generating encryption keys from shared secrets (HKDF-SHA256/384/512)';

-- Add signature algorithm support to device_capabilities
-- Tracks which digital signature algorithms each device can verify and generate
ALTER TABLE device_capabilities
ADD COLUMN supports_ed25519_signature BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN supports_rsa_pss_signature BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN supports_ecdsa_p256_signature BOOLEAN NOT NULL DEFAULT false;
COMMENT ON COLUMN device_capabilities.supports_ed25519_signature IS 'Whether device supports Ed25519 digital signatures (modern, fast, secure - recommended default)';
COMMENT ON COLUMN device_capabilities.supports_rsa_pss_signature IS 'Whether device supports RSA-PSS digital signatures (legacy compatibility, larger signatures)';
COMMENT ON COLUMN device_capabilities.supports_ecdsa_p256_signature IS 'Whether device supports ECDSA-P256 digital signatures (NIST standard, moderate performance)';

-- Add KDF algorithm support to device_capabilities
-- Tracks which key derivation functions each device supports for shared secret processing
ALTER TABLE device_capabilities
ADD COLUMN supports_hkdf_sha256 BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN supports_hkdf_sha384 BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN supports_hkdf_sha512 BOOLEAN NOT NULL DEFAULT false;
COMMENT ON COLUMN device_capabilities.supports_hkdf_sha256 IS 'Whether device supports HKDF-SHA256 key derivation (recommended default for most E2EE scenarios)';
COMMENT ON COLUMN device_capabilities.supports_hkdf_sha384 IS 'Whether device supports HKDF-SHA384 key derivation (higher security margin, slower performance)';
COMMENT ON COLUMN device_capabilities.supports_hkdf_sha512 IS 'Whether device supports HKDF-SHA512 key derivation (maximum security, slowest performance)';

-- Add post-quantum cryptography support to device_capabilities
-- Prepare for quantum-resistant algorithms as they become standardized
ALTER TABLE device_capabilities
ADD COLUMN supports_kyber_768 BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN supports_dilithium2 BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN supports_sphincs_plus BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN supports_bike_r4 BOOLEAN NOT NULL DEFAULT false;
COMMENT ON COLUMN device_capabilities.supports_kyber_768 IS 'Whether device supports Kyber-768 post-quantum key encapsulation mechanism (NIST standard)';
COMMENT ON COLUMN device_capabilities.supports_dilithium2 IS 'Whether device supports Dilithium2 post-quantum digital signatures (NIST standard)';
COMMENT ON COLUMN device_capabilities.supports_sphincs_plus IS 'Whether device supports SPHINCS+ post-quantum signatures (alternative to Dilithium)';
COMMENT ON COLUMN device_capabilities.supports_bike_r4 IS 'Whether device supports BIKE Round 4 post-quantum key encapsulation (alternative to Kyber)';

-- Add negotiated signature and KDF algorithms to conversation negotiations
-- Stores the agreed-upon algorithms for digital signatures and key derivation in each conversation
ALTER TABLE conversation_algorithm_negotiations
ADD COLUMN negotiated_signature_algorithm VARCHAR,
ADD COLUMN negotiated_kdf_algorithm VARCHAR,
ADD COLUMN negotiated_pq_kem_algorithm VARCHAR,
ADD COLUMN negotiated_pq_signature_algorithm VARCHAR;
COMMENT ON COLUMN conversation_algorithm_negotiations.negotiated_signature_algorithm IS 'Agreed-upon digital signature algorithm for message authentication in this conversation';
COMMENT ON COLUMN conversation_algorithm_negotiations.negotiated_kdf_algorithm IS 'Agreed-upon key derivation function for generating conversation keys from shared secrets';
COMMENT ON COLUMN conversation_algorithm_negotiations.negotiated_pq_kem_algorithm IS 'Agreed-upon post-quantum key encapsulation mechanism for hybrid classical/quantum-resistant security';
COMMENT ON COLUMN conversation_algorithm_negotiations.negotiated_pq_signature_algorithm IS 'Agreed-upon post-quantum signature algorithm for quantum-resistant message authentication';

-- Add indexes for new algorithm capabilities
-- Optimize queries for signature algorithm capability matching during conversation setup
CREATE INDEX idx_device_capabilities_signature ON device_capabilities (supports_ed25519_signature, supports_rsa_pss_signature);
COMMENT ON INDEX idx_device_capabilities_signature IS 'Optimizes signature algorithm capability queries for E2EE negotiation and compatibility checking';

-- Optimize queries for KDF algorithm capability matching during key derivation
CREATE INDEX idx_device_capabilities_kdf ON device_capabilities (supports_hkdf_sha256, supports_hkdf_sha384);
COMMENT ON INDEX idx_device_capabilities_kdf IS 'Optimizes KDF algorithm capability queries for secure key derivation negotiation';

-- Add indexes for post-quantum algorithm capabilities
CREATE INDEX idx_device_capabilities_pq_kem ON device_capabilities (supports_kyber_768, supports_bike_r4);
COMMENT ON INDEX idx_device_capabilities_pq_kem IS 'Optimizes post-quantum key encapsulation mechanism queries for quantum-resistant key exchange';

CREATE INDEX idx_device_capabilities_pq_signature ON device_capabilities (supports_dilithium2, supports_sphincs_plus);
COMMENT ON INDEX idx_device_capabilities_pq_signature IS 'Optimizes post-quantum signature algorithm queries for quantum-resistant authentication';

-- Add post-quantum algorithm preferences to device preferences
ALTER TABLE device_algorithm_preferences
ADD COLUMN preferred_pq_kem_algorithms TEXT[] NOT NULL DEFAULT '{}',
ADD COLUMN preferred_pq_signature_algorithms TEXT[] NOT NULL DEFAULT '{}';
COMMENT ON COLUMN device_algorithm_preferences.preferred_pq_kem_algorithms IS 'Ordered list of preferred post-quantum key encapsulation mechanisms (Kyber-768, BIKE-R4, etc.)';
COMMENT ON COLUMN device_algorithm_preferences.preferred_pq_signature_algorithms IS 'Ordered list of preferred post-quantum signature algorithms (Dilithium2, SPHINCS+, etc.)';