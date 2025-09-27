-- Create countries table
CREATE TABLE countries (
    id CHAR(26) PRIMARY KEY,
    name VARCHAR NOT NULL,
    iso_code VARCHAR NOT NULL UNIQUE,
    phone_code VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by_id CHAR(26) REFERENCES sys_users(id),
    updated_by_id CHAR(26) REFERENCES sys_users(id),
    deleted_by_id CHAR(26) REFERENCES sys_users(id)
);

-- Add indexes
CREATE INDEX idx_countries_name ON countries (name);
CREATE INDEX idx_countries_iso_code ON countries (iso_code);
CREATE INDEX idx_countries_created_at ON countries (created_at);
CREATE INDEX idx_countries_created_by_id ON countries (created_by_id);
CREATE INDEX idx_countries_updated_by_id ON countries (updated_by_id);
CREATE INDEX idx_countries_deleted_by_id ON countries (deleted_by_id);
