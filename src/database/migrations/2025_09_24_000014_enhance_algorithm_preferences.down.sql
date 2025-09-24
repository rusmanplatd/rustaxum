-- Rollback algorithm preferences enhancements

-- Remove indexes
DROP INDEX IF EXISTS idx_device_capabilities_kdf;
DROP INDEX IF EXISTS idx_device_capabilities_signature;

-- Remove columns from conversation_algorithm_negotiations
ALTER TABLE conversation_algorithm_negotiations
DROP COLUMN IF EXISTS negotiated_kdf_algorithm,
DROP COLUMN IF EXISTS negotiated_signature_algorithm;

-- Remove KDF algorithm support from device_capabilities
ALTER TABLE device_capabilities
DROP COLUMN IF EXISTS supports_hkdf_sha512,
DROP COLUMN IF EXISTS supports_hkdf_sha384,
DROP COLUMN IF EXISTS supports_hkdf_sha256;

-- Remove signature algorithm support from device_capabilities
ALTER TABLE device_capabilities
DROP COLUMN IF EXISTS supports_ecdsa_p256_signature,
DROP COLUMN IF EXISTS supports_rsa_pss_signature,
DROP COLUMN IF EXISTS supports_ed25519_signature;

-- Remove algorithm preferences from device_algorithm_preferences
ALTER TABLE device_algorithm_preferences
DROP COLUMN IF EXISTS preferred_kdf_algorithms,
DROP COLUMN IF EXISTS preferred_signature_algorithms;