-- Create oauth_auth_codes table
CREATE TABLE IF NOT EXISTS oauth_auth_codes (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_id TEXT NOT NULL REFERENCES oauth_clients(id) ON DELETE CASCADE,
    scopes TEXT DEFAULT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    expires_at TIMESTAMP DEFAULT NULL,
    challenge VARCHAR(255) DEFAULT NULL,
    challenge_method VARCHAR(255) DEFAULT NULL,
    redirect_uri TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create indexes for faster lookups
CREATE INDEX IF NOT EXISTS idx_oauth_auth_codes_user_id ON oauth_auth_codes(user_id);
CREATE INDEX IF NOT EXISTS idx_oauth_auth_codes_client_id ON oauth_auth_codes(client_id);
CREATE INDEX IF NOT EXISTS idx_oauth_auth_codes_revoked ON oauth_auth_codes(revoked);
CREATE INDEX IF NOT EXISTS idx_oauth_auth_codes_expires_at ON oauth_auth_codes(expires_at);