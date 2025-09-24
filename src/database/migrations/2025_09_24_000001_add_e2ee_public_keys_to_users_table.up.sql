-- Add E2EE identity keys to sys_users table
-- Extends the existing users table with E2EE cryptographic key support

-- Add E2EE identity key (public key only, private key never stored on server)
ALTER TABLE sys_users ADD COLUMN identity_public_key TEXT;
COMMENT ON COLUMN sys_users.identity_public_key IS 'User''s Ed25519 identity public key for E2EE verification (base64 encoded)';

-- Add timestamp for key rotation tracking
ALTER TABLE sys_users ADD COLUMN identity_key_created_at TIMESTAMPTZ;
COMMENT ON COLUMN sys_users.identity_key_created_at IS 'When the current identity key was generated, used for key rotation scheduling';

-- Add indexes for E2EE key lookups
CREATE INDEX idx_sys_users_identity_public_key ON sys_users (identity_public_key);
COMMENT ON INDEX idx_sys_users_identity_public_key IS 'Fast lookup of users by their identity public key for verification';

CREATE INDEX idx_sys_users_identity_key_created_at ON sys_users (identity_key_created_at);
COMMENT ON INDEX idx_sys_users_identity_key_created_at IS 'Optimize queries for key rotation scheduling and age-based filtering';