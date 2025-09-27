-- Drop indexes
DROP INDEX IF EXISTS idx_sys_users_mfa_enabled;
DROP INDEX IF EXISTS idx_mfa_attempts_user_time;
DROP INDEX IF EXISTS idx_mfa_attempts_user_id;
DROP INDEX IF EXISTS idx_mfa_methods_user_type;
DROP INDEX IF EXISTS idx_mfa_methods_user_id;

-- Remove MFA columns from sys_users table
ALTER TABLE sys_users
DROP COLUMN IF EXISTS mfa_required,
DROP COLUMN IF EXISTS mfa_backup_codes,
DROP COLUMN IF EXISTS mfa_secret,
DROP COLUMN IF EXISTS mfa_enabled;

-- Drop tables
DROP TABLE IF EXISTS mfa_attempts;
DROP TABLE IF EXISTS mfa_methods;