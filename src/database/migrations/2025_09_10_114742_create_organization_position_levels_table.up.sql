-- Create organization_position_levels table
CREATE TABLE organization_position_levels (
    id CHAR(26) PRIMARY KEY,
    organization_id CHAR(26) NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    code VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    description TEXT,
    level INTEGER NOT NULL, -- Lower numbers = higher hierarchy
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by_id CHAR(26) REFERENCES sys_users(id),
    updated_by_id CHAR(26) REFERENCES sys_users(id),
    deleted_by_id CHAR(26) REFERENCES sys_users(id),
    UNIQUE (organization_id, code)
);

-- Add indexes
CREATE INDEX idx_organization_position_level_organization ON organization_position_levels (organization_id);
CREATE INDEX idx_organization_position_level_name ON organization_position_levels (name);
CREATE INDEX idx_organization_position_level_code ON organization_position_levels (code);
CREATE INDEX idx_organization_position_level_level ON organization_position_levels (level);
CREATE INDEX idx_organization_position_level_is_active ON organization_position_levels (is_active);
CREATE INDEX idx_organization_position_level_created_at ON organization_position_levels (created_at);
CREATE INDEX idx_organization_position_level_created_by_id ON organization_position_levels (created_by_id);
CREATE INDEX idx_organization_position_level_updated_by_id ON organization_position_levels (updated_by_id);
CREATE INDEX idx_organization_position_level_deleted_by_id ON organization_position_levels (deleted_by_id);
