-- Create oauth_personal_access_clients table
CREATE TABLE IF NOT EXISTS oauth_personal_access_clients (
    id TEXT PRIMARY KEY,
    client_id TEXT NOT NULL REFERENCES oauth_clients(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create indexes for faster lookups
CREATE INDEX IF NOT EXISTS idx_oauth_personal_access_clients_client_id ON oauth_personal_access_clients(client_id);