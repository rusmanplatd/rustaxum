-- Create user_organizations pivot table
CREATE TABLE user_organizations (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE RESTRICT,
    organization_id CHAR(26) NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    organization_position_id CHAR(26) NOT NULL REFERENCES organization_positions(id) ON DELETE RESTRICT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by_id CHAR(26) REFERENCES sys_users(id),
    updated_by_id CHAR(26) REFERENCES sys_users(id),
    deleted_by_id CHAR(26) REFERENCES sys_users(id),
    UNIQUE(user_id, organization_id, organization_position_id, started_at)
);

-- Add indexes
CREATE INDEX idx_user_organizations_user_id ON user_organizations (user_id);
CREATE INDEX idx_user_organizations_organization_id ON user_organizations (organization_id);
CREATE INDEX idx_user_organizations_organization_position_id ON user_organizations (organization_position_id);
CREATE INDEX idx_user_organizations_is_active ON user_organizations (is_active);
CREATE INDEX idx_user_organizations_started_at ON user_organizations (started_at);
CREATE INDEX idx_user_organizations_ended_at ON user_organizations (ended_at);
CREATE INDEX idx_user_organizations_created_at ON user_organizations (created_at);
CREATE INDEX idx_user_organizations_created_by_id ON user_organizations (created_by_id);
CREATE INDEX idx_user_organizations_updated_by_id ON user_organizations (updated_by_id);
CREATE INDEX idx_user_organizations_deleted_by_id ON user_organizations (deleted_by_id);
