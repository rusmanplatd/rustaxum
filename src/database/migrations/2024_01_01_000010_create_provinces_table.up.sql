-- Create provinces table
CREATE TABLE provinces (
    id CHAR(26) PRIMARY KEY,
    country_id CHAR(26) NOT NULL,
    name VARCHAR NOT NULL,
    code VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by_id CHAR(26) REFERENCES sys_users(id),
    updated_by_id CHAR(26) REFERENCES sys_users(id),
    deleted_by_id CHAR(26) REFERENCES sys_users(id),
    FOREIGN KEY (country_id) REFERENCES countries(id) ON DELETE CASCADE
);

-- Add indexes
CREATE INDEX idx_provinces_name ON provinces (name);
CREATE INDEX idx_provinces_country_id ON provinces (country_id);
CREATE INDEX idx_provinces_code ON provinces (code);
CREATE INDEX idx_provinces_created_at ON provinces (created_at);
CREATE INDEX idx_provinces_created_by_id ON provinces (created_by_id);
CREATE INDEX idx_provinces_updated_by_id ON provinces (updated_by_id);
CREATE INDEX idx_provinces_deleted_by_id ON provinces (deleted_by_id);
