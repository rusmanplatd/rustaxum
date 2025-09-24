-- Remove E2EE public identity keys from sys_users table
DROP INDEX IF EXISTS idx_sys_users_identity_public_key;
DROP INDEX IF EXISTS idx_sys_users_identity_key_created_at;

ALTER TABLE sys_users DROP COLUMN IF EXISTS identity_public_key;
ALTER TABLE sys_users DROP COLUMN IF EXISTS identity_key_created_at;
ALTER TABLE sys_users DROP COLUMN IF EXISTS supported_algorithms;