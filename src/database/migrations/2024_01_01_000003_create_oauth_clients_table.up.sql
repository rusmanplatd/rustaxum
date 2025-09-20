-- Create oauth_clients table
CREATE TABLE oauth_clients (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) REFERENCES sys_users(id) ON DELETE CASCADE,
    name VARCHAR NOT NULL,
    secret VARCHAR DEFAULT NULL,
    provider VARCHAR DEFAULT NULL,
    redirect_uris TEXT NOT NULL,
    personal_access_client BOOLEAN NOT NULL DEFAULT FALSE,
    password_client BOOLEAN NOT NULL DEFAULT FALSE,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_oauth_clients_name ON oauth_clients (name);
CREATE INDEX idx_oauth_clients_user_id ON oauth_clients (user_id);
CREATE INDEX idx_oauth_clients_revoked ON oauth_clients (revoked);
CREATE INDEX idx_oauth_clients_personal_access_client ON oauth_clients (personal_access_client);
CREATE INDEX idx_oauth_clients_password_client ON oauth_clients (password_client);
CREATE INDEX idx_oauth_clients_created_at ON oauth_clients (created_at);