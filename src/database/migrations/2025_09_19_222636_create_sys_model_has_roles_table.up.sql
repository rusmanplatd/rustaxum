-- Create sys_model_has_roles table
CREATE TABLE sys_model_has_roles (
    id CHAR(26) PRIMARY KEY,
    model_type VARCHAR(255) NOT NULL,
    model_id CHAR(26) NOT NULL,
    role_id CHAR(26) NOT NULL,
    scope_type VARCHAR(255), -- Type of resource this permission assignment is scoped to
    scope_id CHAR(26), -- ID of the resource this permission assignment is scoped to
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by CHAR(26) NOT NULL REFERENCES sys_users(id),
    updated_by CHAR(26) NOT NULL REFERENCES sys_users(id),
    deleted_by CHAR(26) REFERENCES sys_users(id),
    FOREIGN KEY (role_id) REFERENCES sys_roles(id) ON DELETE CASCADE
);

-- Add indexes
CREATE INDEX idx_sys_model_has_roles_model ON sys_model_has_roles (model_type, model_id);
CREATE INDEX idx_sys_model_has_roles_scope ON sys_model_has_roles (scope_type, scope_id);
CREATE INDEX idx_sys_model_has_roles_role_id ON sys_model_has_roles (role_id);

-- Add unique constraint for conflict handling
CREATE UNIQUE INDEX idx_sys_model_has_roles_unique ON sys_model_has_roles (model_type, model_id, role_id);
