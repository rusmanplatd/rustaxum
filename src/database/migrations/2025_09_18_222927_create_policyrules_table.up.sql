-- Create policyrules table
CREATE TABLE policyrules (
    id TEXT PRIMARY KEY,
    name VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_policyrules_name ON policyrules (name);
CREATE INDEX idx_policyrules_created_at ON policyrules (created_at);
