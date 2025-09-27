-- Create MFA methods table
CREATE TABLE mfa_methods (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    method_type VARCHAR(50) NOT NULL CHECK (method_type IN ('totp', 'backup_codes')),
    secret TEXT, -- Encrypted TOTP secret or backup codes
    is_enabled BOOLEAN NOT NULL DEFAULT false,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    backup_codes JSONB, -- Array of hashed backup codes
    recovery_codes_used_count INTEGER NOT NULL DEFAULT 0,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,

    UNIQUE(user_id, method_type)
);

-- Create MFA authentication attempts table (for rate limiting)
CREATE TABLE mfa_attempts (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    method_type VARCHAR(50) NOT NULL,
    ip_address TEXT,
    user_agent TEXT,
    success BOOLEAN NOT NULL,
    attempted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add MFA columns to sys_users table
ALTER TABLE sys_users
ADD COLUMN mfa_enabled BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN mfa_secret TEXT, -- Legacy field for backward compatibility
ADD COLUMN mfa_backup_codes JSONB, -- Legacy field for backward compatibility
ADD COLUMN mfa_required BOOLEAN NOT NULL DEFAULT false; -- Admin can enforce MFA

-- Create indexes for performance
CREATE INDEX idx_mfa_methods_user_id ON mfa_methods(user_id);
CREATE INDEX idx_mfa_methods_user_type ON mfa_methods(user_id, method_type);
CREATE INDEX idx_mfa_attempts_user_id ON mfa_attempts(user_id);
CREATE INDEX idx_mfa_attempts_user_time ON mfa_attempts(user_id, attempted_at);
CREATE INDEX idx_sys_users_mfa_enabled ON sys_users(mfa_enabled);