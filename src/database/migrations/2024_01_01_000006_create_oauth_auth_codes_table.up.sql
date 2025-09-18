-- Create oauth_auth_codes table
CREATE TABLE oauth_auth_codes (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_id TEXT NOT NULL REFERENCES oauth_clients(id) ON DELETE CASCADE,
    scopes TEXT DEFAULT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    expires_at TIMESTAMPTZ DEFAULT NULL,
    challenge VARCHAR DEFAULT NULL,
    challenge_method VARCHAR DEFAULT NULL,
    redirect_uri TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_oauth_auth_codes_user_id ON oauth_auth_codes (user_id);
CREATE INDEX idx_oauth_auth_codes_client_id ON oauth_auth_codes (client_id);
CREATE INDEX idx_oauth_auth_codes_revoked ON oauth_auth_codes (revoked);
CREATE INDEX idx_oauth_auth_codes_expires_at ON oauth_auth_codes (expires_at);
CREATE INDEX idx_oauth_auth_codes_created_at ON oauth_auth_codes (created_at);