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
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by_id CHAR(26) REFERENCES sys_users(id),
    updated_by_id CHAR(26) REFERENCES sys_users(id),
    deleted_by_id CHAR(26) REFERENCES sys_users(id)
);

-- Add indexes
CREATE INDEX idx_sys_users_name ON sys_users (name);
CREATE INDEX idx_sys_users_created_at ON sys_users (created_at);
CREATE INDEX idx_sys_users_created_by_id ON sys_users (created_by_id);
CREATE INDEX idx_sys_users_updated_by_id ON sys_users (updated_by_id);
CREATE INDEX idx_sys_users_deleted_by_id ON sys_users (deleted_by_id);


-- Add trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';