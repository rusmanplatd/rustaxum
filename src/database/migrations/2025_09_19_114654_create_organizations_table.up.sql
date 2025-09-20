-- Create organizations table with hierarchical structure
CREATE TABLE organizations (
    id TEXT PRIMARY KEY,
    name VARCHAR NOT NULL,
    type VARCHAR NOT NULL CHECK (type IN ('company', 'boc', 'bod', 'division', 'department', 'branch', 'subbranch', 'section')),
    parent_id TEXT REFERENCES organizations(id) ON DELETE CASCADE,
    code VARCHAR UNIQUE,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_organizations_name ON organizations (name);
CREATE INDEX idx_organizations_type ON organizations (type);
CREATE INDEX idx_organizations_parent_id ON organizations (parent_id);
CREATE INDEX idx_organizations_code ON organizations (code);
CREATE INDEX idx_organizations_is_active ON organizations (is_active);
CREATE INDEX idx_organizations_created_at ON organizations (created_at);
