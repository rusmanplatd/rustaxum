-- Create oauth_device_codes table for RFC 8628 Device Authorization Grant
CREATE TABLE oauth_device_codes (
    id CHAR(26) PRIMARY KEY,
    device_code VARCHAR(64) UNIQUE NOT NULL,
    user_code VARCHAR(9) UNIQUE NOT NULL, -- Format: ABCD-EFGH
    client_id CHAR(26) NOT NULL REFERENCES oauth_clients(id) ON DELETE CASCADE,
    user_id CHAR(26) REFERENCES sys_users(id) ON DELETE CASCADE,
    scopes TEXT DEFAULT NULL,
    verification_uri VARCHAR(255) NOT NULL,
    verification_uri_complete VARCHAR(512) DEFAULT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    interval INTEGER NOT NULL DEFAULT 5, -- Minimum polling interval in seconds
    user_authorized BOOLEAN NOT NULL DEFAULT FALSE,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes for performance
CREATE INDEX idx_oauth_device_codes_device_code ON oauth_device_codes (device_code);
CREATE INDEX idx_oauth_device_codes_user_code ON oauth_device_codes (user_code);
CREATE INDEX idx_oauth_device_codes_client_id ON oauth_device_codes (client_id);
CREATE INDEX idx_oauth_device_codes_user_id ON oauth_device_codes (user_id);
CREATE INDEX idx_oauth_device_codes_expires_at ON oauth_device_codes (expires_at);
CREATE INDEX idx_oauth_device_codes_user_authorized ON oauth_device_codes (user_authorized);
CREATE INDEX idx_oauth_device_codes_revoked ON oauth_device_codes (revoked);
CREATE INDEX idx_oauth_device_codes_created_at ON oauth_device_codes (created_at);

-- Add composite indexes for common queries
CREATE INDEX idx_oauth_device_codes_client_authorized ON oauth_device_codes (client_id, user_authorized, expires_at);
CREATE INDEX idx_oauth_device_codes_active ON oauth_device_codes (revoked, expires_at) WHERE revoked = FALSE;