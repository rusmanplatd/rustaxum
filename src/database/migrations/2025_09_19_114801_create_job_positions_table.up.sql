-- Create job_positions table
CREATE TABLE job_positions (
    id CHAR(26) PRIMARY KEY,
    name VARCHAR NOT NULL,
    code VARCHAR UNIQUE,
    job_level_id CHAR(26) NOT NULL REFERENCES job_levels(id) ON DELETE RESTRICT,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_job_positions_name ON job_positions (name);
CREATE INDEX idx_job_positions_code ON job_positions (code);
CREATE INDEX idx_job_positions_job_level_id ON job_positions (job_level_id);
CREATE INDEX idx_job_positions_is_active ON job_positions (is_active);
CREATE INDEX idx_job_positions_created_at ON job_positions (created_at);
