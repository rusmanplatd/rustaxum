-- Create users table
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    name VARCHAR NOT NULL,
    email VARCHAR UNIQUE NOT NULL,
    email_verified_at TIMESTAMPTZ,
    password VARCHAR NOT NULL,
    remember_token VARCHAR,
    refresh_token VARCHAR,
    refresh_token_expires_at TIMESTAMPTZ,
    password_reset_token VARCHAR,
    password_reset_expires_at TIMESTAMPTZ,
    last_login_at TIMESTAMPTZ,
    failed_login_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_name ON users (name);
CREATE INDEX idx_users_created_at ON users (created_at);