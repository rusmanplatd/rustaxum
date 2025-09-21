-- Create oauth_scopes table
CREATE TABLE oauth_scopes (
    id CHAR(26) PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    description TEXT DEFAULT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_oauth_scopes_name ON oauth_scopes (name);
CREATE INDEX idx_oauth_scopes_is_default ON oauth_scopes (is_default);
CREATE INDEX idx_oauth_scopes_created_at ON oauth_scopes (created_at);

-- Insert default scopes
INSERT INTO oauth_scopes (id, name, description, is_default) VALUES
    ('01', '*', 'All permissions', false),
    ('02', 'read', 'Read access to resources', true),
    ('03', 'write', 'Write access to resources', false),
    ('04', 'admin', 'Administrative access', false)
ON CONFLICT (name) DO NOTHING;