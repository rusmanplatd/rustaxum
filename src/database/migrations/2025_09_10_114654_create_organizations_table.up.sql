-- Create organizations table with hierarchical structure
CREATE TABLE organizations (
    id CHAR(26) PRIMARY KEY,
    domain_id CHAR(26) NOT NULL REFERENCES organization_domains(id) ON DELETE CASCADE,
    parent_id CHAR(26) REFERENCES organizations(id) ON DELETE CASCADE,
    type_id CHAR(26) NOT NULL REFERENCES organization_types(id) ON DELETE CASCADE,
    code VARCHAR,
    name VARCHAR NOT NULL,
    address TEXT,
    authorized_capital NUMERIC(15, 2),
    business_activities TEXT,
    contact_persons JSONB,
    description TEXT,
    email VARCHAR,
    establishment_date DATE,
    governance_structure JSONB,
    legal_status VARCHAR,
    paid_capital NUMERIC(15, 2),
    path VARCHAR,
    phone VARCHAR,
    registration_number VARCHAR,
    tax_number VARCHAR,
    website VARCHAR,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by_id CHAR(26) NOT NULL REFERENCES sys_users(id),
    updated_by_id CHAR(26) NOT NULL REFERENCES sys_users(id),
    deleted_by_id CHAR(26) REFERENCES sys_users(id),
    UNIQUE (domain_id, parent_id, code)
);

-- Add indexes
CREATE INDEX idx_organizations_name ON organizations (name);
CREATE INDEX idx_organizations_domain_id ON organizations (domain_id);
CREATE INDEX idx_organizations_parent_id ON organizations (parent_id);
CREATE INDEX idx_organizations_type_id ON organizations (type_id);
CREATE INDEX idx_organizations_code ON organizations (code);
CREATE INDEX idx_organizations_is_active ON organizations (is_active);
CREATE INDEX idx_organizations_created_at ON organizations (created_at);
CREATE INDEX idx_organizations_created_by_id ON organizations (created_by_id);
CREATE INDEX idx_organizations_updated_by_id ON organizations (updated_by_id);
CREATE INDEX idx_organizations_deleted_by_id ON organizations (deleted_by_id);
