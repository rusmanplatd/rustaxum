-- Create districts table
CREATE TABLE districts (
    id CHAR(26) PRIMARY KEY,
    city_id CHAR(26) NOT NULL,
    name VARCHAR NOT NULL,
    code VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by_id CHAR(26) REFERENCES sys_users(id),
    updated_by_id CHAR(26) REFERENCES sys_users(id),
    deleted_by_id CHAR(26) REFERENCES sys_users(id),
    FOREIGN KEY (city_id) REFERENCES cities(id) ON DELETE CASCADE
);

-- Add indexes
CREATE INDEX idx_districts_name ON districts (name);
CREATE INDEX idx_districts_city_id ON districts (city_id);
CREATE INDEX idx_districts_code ON districts (code);
CREATE INDEX idx_districts_created_at ON districts (created_at);
CREATE INDEX idx_districts_created_by_id ON districts (created_by_id);
CREATE INDEX idx_districts_updated_by_id ON districts (updated_by_id);
CREATE INDEX idx_districts_deleted_by_id ON districts (deleted_by_id);