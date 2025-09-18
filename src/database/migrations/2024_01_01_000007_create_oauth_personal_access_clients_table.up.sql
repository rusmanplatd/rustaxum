-- Create oauth_personal_access_clients table
CREATE TABLE oauth_personal_access_clients (
    id TEXT PRIMARY KEY,
    client_id TEXT NOT NULL REFERENCES oauth_clients(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_oauth_personal_access_clients_client_id ON oauth_personal_access_clients (client_id);
CREATE INDEX idx_oauth_personal_access_clients_created_at ON oauth_personal_access_clients (created_at);