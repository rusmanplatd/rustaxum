-- Create policies table
CREATE TABLE policies (
    id TEXT PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    description TEXT,
    effect VARCHAR NOT NULL CHECK (effect IN ('permit', 'deny')),
    target VARCHAR NOT NULL,
    condition TEXT,
    priority INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_policies_name ON policies (name);
CREATE INDEX idx_policies_effect ON policies (effect);
CREATE INDEX idx_policies_target ON policies (target);
CREATE INDEX idx_policies_priority ON policies (priority);
CREATE INDEX idx_policies_is_active ON policies (is_active);
CREATE INDEX idx_policies_created_at ON policies (created_at);
