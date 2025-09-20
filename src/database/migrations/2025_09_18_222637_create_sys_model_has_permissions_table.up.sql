-- Create role_permissions table
CREATE TABLE sys_model_has_permissions (
    id CHAR(26) PRIMARY KEY,
    model_type VARCHAR(255) NOT NULL,
    model_id CHAR(26) NOT NULL,
    permission_id CHAR(26) NOT NULL,
    scope_type VARCHAR(255),
    scope_id CHAR(26),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (permission_id) REFERENCES sys_permissions(id) ON DELETE CASCADE
);

-- Add indexes
CREATE INDEX idx_sys_model_has_permissions_model ON sys_model_has_permissions (model_type, model_id);
CREATE INDEX idx_sys_model_has_permissions_scope ON sys_model_has_permissions (scope_type, scope_id);
CREATE INDEX idx_sys_model_has_permissions_permission_id ON sys_model_has_permissions (permission_id);

