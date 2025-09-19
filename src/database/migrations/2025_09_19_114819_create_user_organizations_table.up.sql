-- Create user_organizations pivot table
CREATE TABLE user_organizations (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    job_position_id TEXT NOT NULL REFERENCES job_positions(id) ON DELETE RESTRICT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, organization_id, job_position_id, started_at)
);

-- Add indexes
CREATE INDEX idx_user_organizations_user_id ON user_organizations (user_id);
CREATE INDEX idx_user_organizations_organization_id ON user_organizations (organization_id);
CREATE INDEX idx_user_organizations_job_position_id ON user_organizations (job_position_id);
CREATE INDEX idx_user_organizations_is_active ON user_organizations (is_active);
CREATE INDEX idx_user_organizations_started_at ON user_organizations (started_at);
CREATE INDEX idx_user_organizations_ended_at ON user_organizations (ended_at);
CREATE INDEX idx_user_organizations_created_at ON user_organizations (created_at);
