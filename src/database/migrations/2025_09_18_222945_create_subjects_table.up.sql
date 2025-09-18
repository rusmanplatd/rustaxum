-- Create subjects table
CREATE TABLE subjects (
    id TEXT PRIMARY KEY,
    name VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_subjects_name ON subjects (name);
CREATE INDEX idx_subjects_created_at ON subjects (created_at);
