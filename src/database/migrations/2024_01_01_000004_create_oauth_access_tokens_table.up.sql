-- Create oauth_access_tokens table
CREATE TABLE oauth_access_tokens (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    client_id TEXT NOT NULL REFERENCES oauth_clients(id) ON DELETE CASCADE,
    name VARCHAR DEFAULT NULL,
    scopes TEXT DEFAULT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    expires_at TIMESTAMPTZ DEFAULT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_oauth_access_tokens_name ON oauth_access_tokens (name);
CREATE INDEX idx_oauth_access_tokens_user_id ON oauth_access_tokens (user_id);
CREATE INDEX idx_oauth_access_tokens_client_id ON oauth_access_tokens (client_id);
CREATE INDEX idx_oauth_access_tokens_revoked ON oauth_access_tokens (revoked);
CREATE INDEX idx_oauth_access_tokens_expires_at ON oauth_access_tokens (expires_at);
CREATE INDEX idx_oauth_access_tokens_created_at ON oauth_access_tokens (created_at);