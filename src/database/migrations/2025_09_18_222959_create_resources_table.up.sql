-- Create resources table
CREATE TABLE resources (
    id TEXT PRIMARY KEY,
    name VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_resources_name ON resources (name);
CREATE INDEX idx_resources_created_at ON resources (created_at);
