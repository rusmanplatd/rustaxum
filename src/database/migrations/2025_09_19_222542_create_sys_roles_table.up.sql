-- Create sys_roles table
CREATE TABLE sys_roles (
    id CHAR(26) PRIMARY KEY,
    organization_id CHAR(26) REFERENCES organizations(id) ON DELETE RESTRICT,
    name VARCHAR NOT NULL,
    description TEXT,
    guard_name VARCHAR NOT NULL DEFAULT 'api',
    scope_type VARCHAR(255), -- Type of resource this permission assignment is scoped to
    scope_id CHAR(26), -- ID of the resource this permission assignment is scoped to
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by_id CHAR(26) NOT NULL REFERENCES sys_users(id),
    updated_by_id CHAR(26) NOT NULL REFERENCES sys_users(id),
    deleted_by_id CHAR(26) REFERENCES sys_users(id),
    UNIQUE(organization_id, name, guard_name, scope_type, scope_id)
);

-- Add indexes
CREATE INDEX idx_sys_roles_organization_id ON sys_roles (organization_id);
CREATE INDEX idx_sys_roles_name ON sys_roles (name);
CREATE INDEX idx_sys_roles_guard_name ON sys_roles (guard_name);
CREATE INDEX idx_sys_roles_scope ON sys_roles (scope_type, scope_id);
CREATE INDEX idx_sys_roles_created_at ON sys_roles (created_at);
CREATE INDEX idx_sys_roles_created_by_id ON sys_roles (created_by_id);
CREATE INDEX idx_sys_roles_updated_by_id ON sys_roles (updated_by_id);
CREATE INDEX idx_sys_roles_deleted_by_id ON sys_roles (deleted_by_id);
