-- Create sys_permissions table
CREATE TABLE sys_permissions (
    id CHAR(26) PRIMARY KEY,
    organization_id CHAR(26) REFERENCES organizations(id) ON DELETE RESTRICT,
    guard_name VARCHAR NOT NULL DEFAULT 'api',
    resource VARCHAR,
    action VARCHAR NOT NULL,
    scope_type VARCHAR(255), -- Type of resource this permission assignment is scoped to
    scope_id CHAR(26), -- ID of the resource this permission assignment is scoped to
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by CHAR(26) NOT NULL REFERENCES sys_users(id),
    updated_by CHAR(26) NOT NULL REFERENCES sys_users(id),
    deleted_by CHAR(26) REFERENCES sys_users(id),
    UNIQUE(resource, action, guard_name, scope_type, scope_id)
);

-- Add indexes
CREATE INDEX idx_permissions_organization_id ON sys_permissions (organization_id);
CREATE INDEX idx_permissions_guard_name ON sys_permissions (guard_name);
CREATE INDEX idx_permissions_scope ON sys_permissions (scope_type, scope_id);
CREATE INDEX idx_permissions_created_at ON sys_permissions (created_at);
