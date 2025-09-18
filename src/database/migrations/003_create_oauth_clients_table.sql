-- Create oauth_clients table
CREATE TABLE IF NOT EXISTS oauth_clients (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    secret VARCHAR(255) DEFAULT NULL,
    provider VARCHAR(255) DEFAULT NULL,
    redirect_uris TEXT NOT NULL,
    personal_access_client BOOLEAN NOT NULL DEFAULT FALSE,
    password_client BOOLEAN NOT NULL DEFAULT FALSE,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create indexes for faster lookups
CREATE INDEX IF NOT EXISTS idx_oauth_clients_user_id ON oauth_clients(user_id);
CREATE INDEX IF NOT EXISTS idx_oauth_clients_revoked ON oauth_clients(revoked);
CREATE INDEX IF NOT EXISTS idx_oauth_clients_personal_access ON oauth_clients(personal_access_client);
CREATE INDEX IF NOT EXISTS idx_oauth_clients_password ON oauth_clients(password_client);