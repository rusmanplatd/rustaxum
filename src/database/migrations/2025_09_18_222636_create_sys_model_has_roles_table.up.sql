-- Create role_permissions table
CREATE TABLE sys_model_has_roles (
    id CHAR(26) PRIMARY KEY,
    model_type VARCHAR(255) NOT NULL,
    model_id CHAR(26) NOT NULL,
    role_id CHAR(26) NOT NULL,
    scope_type VARCHAR(255),
    scope_id CHAR(26),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (role_id) REFERENCES sys_roles(id) ON DELETE CASCADE
);

-- Add indexes
CREATE INDEX idx_sys_model_has_roles_model ON sys_model_has_roles (model_type, model_id);
CREATE INDEX idx_sys_model_has_roles_scope ON sys_model_has_roles (scope_type, scope_id);
CREATE INDEX idx_sys_model_has_roles_role_id ON sys_model_has_roles (role_id);

