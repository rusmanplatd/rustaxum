-- Create countries table
CREATE TABLE countries (
    id CHAR(26) PRIMARY KEY,
    name VARCHAR NOT NULL,
    iso_code VARCHAR NOT NULL UNIQUE,
    phone_code VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_countries_name ON countries (name);
CREATE INDEX idx_countries_iso_code ON countries (iso_code);
CREATE INDEX idx_countries_created_at ON countries (created_at);