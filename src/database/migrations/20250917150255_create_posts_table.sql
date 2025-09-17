-- Create posts table
CREATE TABLE posts (
    id TEXT PRIMARY KEY,
    name VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_posts_name ON posts (name);
CREATE INDEX idx_posts_created_at ON posts (created_at);
