-- Create oauth_scopes table
CREATE TABLE IF NOT EXISTS oauth_scopes (
    id TEXT PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT DEFAULT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create indexes for faster lookups
CREATE INDEX IF NOT EXISTS idx_oauth_scopes_name ON oauth_scopes(name);
CREATE INDEX IF NOT EXISTS idx_oauth_scopes_is_default ON oauth_scopes(is_default);

-- Insert default scopes
INSERT INTO oauth_scopes (id, name, description, is_default) VALUES
    ('01', '*', 'All permissions', false),
    ('02', 'read', 'Read access to resources', true),
    ('03', 'write', 'Write access to resources', false),
    ('04', 'admin', 'Administrative access', false)
ON CONFLICT (name) DO NOTHING;