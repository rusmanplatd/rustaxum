-- Create sys_permissions table
CREATE TABLE sys_permissions (
    id CHAR(26) PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    guard_name VARCHAR NOT NULL DEFAULT 'api',
    resource VARCHAR,
    action VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_permissions_name ON sys_permissions (name);
CREATE INDEX idx_permissions_guard_name ON sys_permissions (guard_name);
CREATE INDEX idx_permissions_resource ON sys_permissions (resource);
CREATE INDEX idx_permissions_action ON sys_permissions (action);
CREATE INDEX idx_permissions_created_at ON sys_permissions (created_at);
