-- Create oauth_refresh_tokens table
CREATE TABLE oauth_refresh_tokens (
    id CHAR(26) PRIMARY KEY,
    access_token_id CHAR(26) NOT NULL REFERENCES oauth_access_tokens(id) ON DELETE CASCADE,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    expires_at TIMESTAMPTZ DEFAULT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_oauth_refresh_tokens_access_token_id ON oauth_refresh_tokens (access_token_id);
CREATE INDEX idx_oauth_refresh_tokens_revoked ON oauth_refresh_tokens (revoked);
CREATE INDEX idx_oauth_refresh_tokens_expires_at ON oauth_refresh_tokens (expires_at);
CREATE INDEX idx_oauth_refresh_tokens_created_at ON oauth_refresh_tokens (created_at);