-- Rollback additional MFA methods

-- Drop cleanup function
DROP FUNCTION IF EXISTS cleanup_expired_mfa_data_extended();

-- Drop indexes
DROP INDEX IF EXISTS idx_mfa_audit_log_method;
DROP INDEX IF EXISTS idx_mfa_audit_log_created_at;
DROP INDEX IF EXISTS idx_mfa_audit_log_user_id;
DROP INDEX IF EXISTS idx_mfa_trusted_devices_expires;
DROP INDEX IF EXISTS idx_mfa_trusted_devices_fingerprint;
DROP INDEX IF EXISTS idx_mfa_trusted_devices_token;
DROP INDEX IF EXISTS idx_mfa_trusted_devices_user_id;
DROP INDEX IF EXISTS idx_mfa_recovery_methods_type;
DROP INDEX IF EXISTS idx_mfa_recovery_methods_user_id;
DROP INDEX IF EXISTS idx_mfa_backup_email_codes_expires;
DROP INDEX IF EXISTS idx_mfa_backup_email_codes_user_id;
DROP INDEX IF EXISTS idx_mfa_backup_emails_verified;
DROP INDEX IF EXISTS idx_mfa_backup_emails_email;
DROP INDEX IF EXISTS idx_mfa_backup_emails_user_id;
DROP INDEX IF EXISTS idx_mfa_push_challenges_status;
DROP INDEX IF EXISTS idx_mfa_push_challenges_expires;
DROP INDEX IF EXISTS idx_mfa_push_challenges_device_id;
DROP INDEX IF EXISTS idx_mfa_push_challenges_user_id;
DROP INDEX IF EXISTS idx_mfa_push_devices_active;
DROP INDEX IF EXISTS idx_mfa_push_devices_token;
DROP INDEX IF EXISTS idx_mfa_push_devices_user_id;
DROP INDEX IF EXISTS idx_mfa_sms_codes_phone;
DROP INDEX IF EXISTS idx_mfa_sms_codes_expires;
DROP INDEX IF EXISTS idx_mfa_sms_codes_user_id;

-- Remove user preference columns
ALTER TABLE sys_users
DROP COLUMN IF EXISTS mfa_trust_device_duration_days,
DROP COLUMN IF EXISTS mfa_trust_device_enabled,
DROP COLUMN IF EXISTS mfa_backup_method,
DROP COLUMN IF EXISTS mfa_primary_method;

-- Restore original method type constraint
ALTER TABLE mfa_methods
DROP CONSTRAINT IF EXISTS mfa_methods_method_type_check;

ALTER TABLE mfa_methods
ADD CONSTRAINT mfa_methods_method_type_check
CHECK (method_type IN ('totp', 'backup_codes', 'email', 'webauthn', 'biometric'));

-- Drop new tables
DROP TABLE IF EXISTS mfa_audit_log;
DROP TABLE IF EXISTS mfa_trusted_devices;
DROP TABLE IF EXISTS mfa_recovery_methods;
DROP TABLE IF EXISTS mfa_backup_email_codes;
DROP TABLE IF EXISTS mfa_backup_emails;
DROP TABLE IF EXISTS mfa_push_challenges;
DROP TABLE IF EXISTS mfa_push_devices;
DROP TABLE IF EXISTS mfa_sms_codes;
