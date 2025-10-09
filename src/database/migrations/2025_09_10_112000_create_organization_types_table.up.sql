-- Create organization_types table with hierarchical structure
CREATE TABLE organization_types (
    id CHAR(26) PRIMARY KEY,
    domain_id CHAR(26) REFERENCES organization_domains(id) ON DELETE CASCADE,
    code VARCHAR,
    name VARCHAR NOT NULL,
    description TEXT,
    level INTEGER NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by_id CHAR(26) NOT NULL REFERENCES sys_users(id),
    updated_by_id CHAR(26) NOT NULL REFERENCES sys_users(id),
    deleted_by_id CHAR(26) REFERENCES sys_users(id),
    UNIQUE (domain_id, code)
);

-- Add indexes
CREATE INDEX idx_organization_levels_name ON organization_types (name);
CREATE INDEX idx_organization_levels_domain_id ON organization_types (domain_id);
CREATE INDEX idx_organization_levels_code ON organization_types (code);
CREATE INDEX idx_organization_levels_created_at ON organization_types (created_at);
CREATE INDEX idx_organization_levels_created_by_id ON organization_types (created_by_id);
CREATE INDEX idx_organization_levels_updated_by_id ON organization_types (updated_by_id);
CREATE INDEX idx_organization_levels_deleted_by_id ON organization_types (deleted_by_id);
