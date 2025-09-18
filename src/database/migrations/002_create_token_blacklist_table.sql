-- Create token_blacklist table
CREATE TABLE token_blacklist (
    id TEXT PRIMARY KEY,
    token_hash VARCHAR NOT NULL UNIQUE,
    user_id TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reason VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Add indexes
CREATE INDEX idx_token_blacklist_token_hash ON token_blacklist (token_hash);
CREATE INDEX idx_token_blacklist_user_id ON token_blacklist (user_id);
CREATE INDEX idx_token_blacklist_expires_at ON token_blacklist (expires_at);
CREATE INDEX idx_token_blacklist_created_at ON token_blacklist (created_at);