-- Create permissions table
CREATE TABLE permissions (
    id TEXT PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    guard_name VARCHAR NOT NULL DEFAULT 'web',
    resource VARCHAR,
    action VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_permissions_name ON permissions (name);
CREATE INDEX idx_permissions_guard_name ON permissions (guard_name);
CREATE INDEX idx_permissions_resource ON permissions (resource);
CREATE INDEX idx_permissions_action ON permissions (action);
CREATE INDEX idx_permissions_created_at ON permissions (created_at);
