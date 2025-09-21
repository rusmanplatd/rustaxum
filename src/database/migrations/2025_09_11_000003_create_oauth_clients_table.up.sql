-- Create oauth_clients table
CREATE TABLE oauth_clients (
    id CHAR(26) PRIMARY KEY,
    organization_id CHAR(26) REFERENCES organizations(id) ON DELETE RESTRICT,
    user_id CHAR(26) REFERENCES sys_users(id) ON DELETE RESTRICT,
    name VARCHAR NOT NULL,
    secret VARCHAR DEFAULT NULL,
    provider VARCHAR DEFAULT NULL,
    redirect_uris TEXT NOT NULL,
    personal_access_client BOOLEAN NOT NULL DEFAULT FALSE,
    password_client BOOLEAN NOT NULL DEFAULT FALSE,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by CHAR(26) REFERENCES sys_users(id),
    updated_by CHAR(26) REFERENCES sys_users(id),
    deleted_by CHAR(26) REFERENCES sys_users(id)
);

-- Add indexes
CREATE INDEX idx_oauth_clients_organization_id ON oauth_clients (organization_id);
CREATE INDEX idx_oauth_clients_user_id ON oauth_clients (user_id);
CREATE INDEX idx_oauth_clients_name ON oauth_clients (name);
CREATE INDEX idx_oauth_clients_revoked ON oauth_clients (revoked);
CREATE INDEX idx_oauth_clients_personal_access_client ON oauth_clients (personal_access_client);
CREATE INDEX idx_oauth_clients_password_client ON oauth_clients (password_client);
CREATE INDEX idx_oauth_clients_created_at ON oauth_clients (created_at);
CREATE INDEX idx_oauth_clients_created_by ON oauth_clients (created_by);
CREATE INDEX idx_oauth_clients_updated_by ON oauth_clients (updated_by);
CREATE INDEX idx_oauth_clients_deleted_by ON oauth_clients (deleted_by);
