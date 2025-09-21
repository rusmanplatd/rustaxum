-- Create sys_roles table
CREATE TABLE sys_roles (
    id CHAR(26) PRIMARY KEY,
    organization_id CHAR(26) REFERENCES organizations(id) ON DELETE RESTRICT,
    name VARCHAR NOT NULL,
    description TEXT,
    guard_name VARCHAR NOT NULL DEFAULT 'api',
    scope_type VARCHAR(255),
    scope_id CHAR(26),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by CHAR(26) REFERENCES sys_users(id),
    updated_by CHAR(26) REFERENCES sys_users(id),
    deleted_by CHAR(26) REFERENCES sys_users(id),
    UNIQUE(organization_id, name, guard_name)
);

-- Add indexes
CREATE INDEX idx_sys_roles_organization_id ON sys_roles (organization_id);
CREATE INDEX idx_sys_roles_name ON sys_roles (name);
CREATE INDEX idx_sys_roles_guard_name ON sys_roles (guard_name);
CREATE INDEX idx_sys_roles_created_at ON sys_roles (created_at);
CREATE INDEX idx_sys_roles_scope ON sys_roles (scope_type, scope_id);
CREATE INDEX idx_sys_roles_created_by ON sys_roles (created_by);
CREATE INDEX idx_sys_roles_updated_by ON sys_roles (updated_by);
CREATE INDEX idx_sys_roles_deleted_by ON sys_roles (deleted_by);
