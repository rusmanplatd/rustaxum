-- Create roles table
CREATE TABLE roles (
    id TEXT PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    description TEXT,
    guard_name VARCHAR NOT NULL DEFAULT 'web',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_roles_name ON roles (name);
CREATE INDEX idx_roles_guard_name ON roles (guard_name);
CREATE INDEX idx_roles_created_at ON roles (created_at);
