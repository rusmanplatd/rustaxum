-- Create token blacklist table for JWT revocation
CREATE TABLE IF NOT EXISTS token_blacklist (
    id TEXT PRIMARY KEY,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    user_id TEXT NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    revoked_at TIMESTAMP NOT NULL DEFAULT NOW(),
    reason VARCHAR(100),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create index on token_hash for faster lookups
CREATE INDEX IF NOT EXISTS idx_token_blacklist_hash ON token_blacklist(token_hash);

-- Create index on user_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_token_blacklist_user_id ON token_blacklist(user_id);

-- Create index on expires_at for cleanup
CREATE INDEX IF NOT EXISTS idx_token_blacklist_expires_at ON token_blacklist(expires_at);