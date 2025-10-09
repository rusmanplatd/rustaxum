-- Rollback advanced MFA methods

-- Drop cleanup function
DROP FUNCTION IF EXISTS cleanup_expired_mfa_data();

-- Drop indexes
DROP INDEX IF EXISTS idx_mfa_webauthn_challenges_expires;
DROP INDEX IF EXISTS idx_mfa_webauthn_challenges_user_id;
DROP INDEX IF EXISTS idx_mfa_biometric_credentials_device;
DROP INDEX IF EXISTS idx_mfa_biometric_credentials_user_id;
DROP INDEX IF EXISTS idx_mfa_webauthn_credentials_credential_id;
DROP INDEX IF EXISTS idx_mfa_webauthn_credentials_user_id;
DROP INDEX IF EXISTS idx_mfa_email_codes_expires;
DROP INDEX IF EXISTS idx_mfa_email_codes_user_id;

-- Remove metadata column from mfa_methods
ALTER TABLE mfa_methods
DROP COLUMN IF EXISTS metadata;

-- Restore original method type constraint
ALTER TABLE mfa_methods
DROP CONSTRAINT IF EXISTS mfa_methods_method_type_check;

ALTER TABLE mfa_methods
ADD CONSTRAINT mfa_methods_method_type_check
CHECK (method_type IN ('totp', 'backup_codes'));

-- Drop new tables
DROP TABLE IF EXISTS mfa_webauthn_challenges;
DROP TABLE IF EXISTS mfa_biometric_credentials;
DROP TABLE IF EXISTS mfa_webauthn_credentials;
DROP TABLE IF EXISTS mfa_email_codes;
