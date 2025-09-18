-- Create attributes table
CREATE TABLE attributes (
    id TEXT PRIMARY KEY,
    name VARCHAR NOT NULL,
    attribute_type VARCHAR NOT NULL CHECK (attribute_type IN ('string', 'number', 'boolean', 'array', 'object')),
    value JSONB NOT NULL,
    subject_type VARCHAR NOT NULL,
    subject_id TEXT,
    resource_type VARCHAR,
    resource_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_attributes_name ON attributes (name);
CREATE INDEX idx_attributes_subject_type ON attributes (subject_type);
CREATE INDEX idx_attributes_subject_id ON attributes (subject_id);
CREATE INDEX idx_attributes_resource_type ON attributes (resource_type);
CREATE INDEX idx_attributes_resource_id ON attributes (resource_id);
CREATE INDEX idx_attributes_value ON attributes USING GIN (value);
CREATE INDEX idx_attributes_created_at ON attributes (created_at);
