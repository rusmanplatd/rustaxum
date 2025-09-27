-- Create organization_positions table
CREATE TABLE organization_positions (
    id CHAR(26) PRIMARY KEY,
    organization_id CHAR(26) NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    organization_position_level_id CHAR(26) NOT NULL REFERENCES organization_position_levels(id) ON DELETE RESTRICT,
    code VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    min_salary DECIMAL(10, 2) NOT NULL DEFAULT 0,
    max_salary DECIMAL(10, 2) NOT NULL DEFAULT 0,
    max_incumbents INTEGER NOT NULL DEFAULT 1,
    qualifications JSONB NOT NULL DEFAULT '[]',
    responsibilities JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by_id CHAR(26) REFERENCES sys_users(id),
    updated_by_id CHAR(26) REFERENCES sys_users(id),
    deleted_by_id CHAR(26) REFERENCES sys_users(id),
    UNIQUE (organization_id, organization_position_level_id, code)
);

-- Add indexes
CREATE INDEX idx_organization_positions_level_organization ON organization_position_levels (organization_id);
CREATE INDEX idx_organization_positions_organization_position_level_id ON organization_positions (organization_position_level_id);
CREATE INDEX idx_organization_positions_name ON organization_positions (name);
CREATE INDEX idx_organization_positions_code ON organization_positions (code);
CREATE INDEX idx_organization_positions_is_active ON organization_positions (is_active);
CREATE INDEX idx_organization_positions_created_at ON organization_positions (created_at);
CREATE INDEX idx_organization_positions_created_by_id ON organization_positions (created_by_id);
CREATE INDEX idx_organization_positions_updated_by_id ON organization_positions (updated_by_id);
CREATE INDEX idx_organization_positions_deleted_by_id ON organization_positions (deleted_by_id);
