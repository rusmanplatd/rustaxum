-- Enhance algorithm preferences with signature and KDF algorithms
-- Add missing algorithm types for comprehensive E2EE support

-- Add signature algorithm preferences to device_algorithm_preferences
ALTER TABLE device_algorithm_preferences
ADD COLUMN preferred_signature_algorithms TEXT[] NOT NULL DEFAULT '{ed25519,rsa-pss,ecdsa-p256}';

-- Add key derivation function preferences
ALTER TABLE device_algorithm_preferences
ADD COLUMN preferred_kdf_algorithms TEXT[] NOT NULL DEFAULT '{hkdf-sha256,hkdf-sha384,hkdf-sha512}';

-- Add signature algorithm support to device_capabilities
ALTER TABLE device_capabilities
ADD COLUMN supports_ed25519_signature BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN supports_rsa_pss_signature BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN supports_ecdsa_p256_signature BOOLEAN NOT NULL DEFAULT false;

-- Add KDF algorithm support to device_capabilities
ALTER TABLE device_capabilities
ADD COLUMN supports_hkdf_sha256 BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN supports_hkdf_sha384 BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN supports_hkdf_sha512 BOOLEAN NOT NULL DEFAULT false;

-- Add negotiated signature and KDF algorithms to conversation negotiations
ALTER TABLE conversation_algorithm_negotiations
ADD COLUMN negotiated_signature_algorithm VARCHAR,
ADD COLUMN negotiated_kdf_algorithm VARCHAR;

-- Add indexes for new algorithm capabilities
CREATE INDEX idx_device_capabilities_signature ON device_capabilities (supports_ed25519_signature, supports_rsa_pss_signature);
CREATE INDEX idx_device_capabilities_kdf ON device_capabilities (supports_hkdf_sha256, supports_hkdf_sha384);