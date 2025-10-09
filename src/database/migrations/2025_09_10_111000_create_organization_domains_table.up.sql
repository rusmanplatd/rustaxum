-- Create organization_domains table with hierarchical structure
CREATE TABLE organization_domains (
    id CHAR(26) PRIMARY KEY,
    code VARCHAR,
    name VARCHAR NOT NULL,
    description TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by_id CHAR(26) NOT NULL REFERENCES sys_users(id),
    updated_by_id CHAR(26) NOT NULL REFERENCES sys_users(id),
    deleted_by_id CHAR(26) REFERENCES sys_users(id),
    UNIQUE (code)
);

-- Add indexes
CREATE INDEX idx_organization_domains_name ON organization_domains (name);
CREATE INDEX idx_organization_domains_code ON organization_domains (code);
CREATE INDEX idx_organization_domains_created_at ON organization_domains (created_at);
CREATE INDEX idx_organization_domains_created_by_id ON organization_domains (created_by_id);
CREATE INDEX idx_organization_domains_updated_by_id ON organization_domains (updated_by_id);
CREATE INDEX idx_organization_domains_deleted_by_id ON organization_domains (deleted_by_id);
