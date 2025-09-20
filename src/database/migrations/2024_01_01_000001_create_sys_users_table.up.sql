-- Create sys_users table
CREATE TABLE sys_users (
    id CHAR(26) PRIMARY KEY,
    name VARCHAR NOT NULL,
    email VARCHAR UNIQUE NOT NULL,
    email_verified_at TIMESTAMPTZ,
    username VARCHAR UNIQUE,
    password VARCHAR NOT NULL,
    remember_token VARCHAR,
    password_reset_token VARCHAR,
    password_reset_expires_at TIMESTAMPTZ,
    refresh_token VARCHAR,
    refresh_token_expires_at TIMESTAMPTZ,
    avatar VARCHAR,
    birthdate DATE,
    failed_login_attempts INTEGER NOT NULL DEFAULT 0,
    google_id VARCHAR,
    last_login_at TIMESTAMPTZ,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    locale VARCHAR,
    locked_until TIMESTAMPTZ,
    phone_number VARCHAR,
    phone_verified_at TIMESTAMPTZ,
    zoneinfo VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_users_name ON sys_users (name);
CREATE INDEX idx_users_created_at ON sys_users (created_at);