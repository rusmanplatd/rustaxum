-- Create job_levels table
CREATE TABLE job_levels (
    id TEXT PRIMARY KEY,
    name VARCHAR NOT NULL,
    code VARCHAR UNIQUE,
    level INTEGER NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_job_levels_name ON job_levels (name);
CREATE INDEX idx_job_levels_code ON job_levels (code);
CREATE INDEX idx_job_levels_level ON job_levels (level);
CREATE INDEX idx_job_levels_is_active ON job_levels (is_active);
CREATE INDEX idx_job_levels_created_at ON job_levels (created_at);
