-- Create districts table
CREATE TABLE districts (
    id CHAR(26) PRIMARY KEY,
    city_id CHAR(26) NOT NULL,
    name VARCHAR NOT NULL,
    code VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by CHAR(26) REFERENCES sys_users(id),
    updated_by CHAR(26) REFERENCES sys_users(id),
    deleted_by CHAR(26) REFERENCES sys_users(id),
    FOREIGN KEY (city_id) REFERENCES cities(id) ON DELETE CASCADE
);

-- Add indexes
CREATE INDEX idx_districts_name ON districts (name);
CREATE INDEX idx_districts_city_id ON districts (city_id);
CREATE INDEX idx_districts_code ON districts (code);
CREATE INDEX idx_districts_created_at ON districts (created_at);
CREATE INDEX idx_districts_created_by ON districts (created_by);
CREATE INDEX idx_districts_updated_by ON districts (updated_by);
CREATE INDEX idx_districts_deleted_by ON districts (deleted_by);