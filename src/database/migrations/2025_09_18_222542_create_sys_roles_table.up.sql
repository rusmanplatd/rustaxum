-- Create sys_roles table
CREATE TABLE sys_roles (
    id CHAR(26) PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    description TEXT,
    guard_name VARCHAR NOT NULL DEFAULT 'api',
    scope_type VARCHAR(255),
    scope_id CHAR(26),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_roles_name ON sys_roles (name);
CREATE INDEX idx_roles_guard_name ON sys_roles (guard_name);
CREATE INDEX idx_roles_created_at ON sys_roles (created_at);
CREATE INDEX idx_roles_scope ON sys_roles (scope_type, scope_id);