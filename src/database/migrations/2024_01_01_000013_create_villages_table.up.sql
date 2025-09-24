-- Create villages table
CREATE TABLE villages (
    id CHAR(26) PRIMARY KEY,
    district_id CHAR(26) NOT NULL,
    name VARCHAR NOT NULL,
    code VARCHAR,
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by CHAR(26) REFERENCES sys_users(id),
    updated_by CHAR(26) REFERENCES sys_users(id),
    deleted_by CHAR(26) REFERENCES sys_users(id),
    FOREIGN KEY (district_id) REFERENCES districts(id) ON DELETE CASCADE
);

-- Add indexes
CREATE INDEX idx_villages_name ON villages (name);
CREATE INDEX idx_villages_district_id ON villages (district_id);
CREATE INDEX idx_villages_code ON villages (code);
CREATE INDEX idx_villages_coordinates ON villages (latitude, longitude);
CREATE INDEX idx_villages_created_at ON villages (created_at);
CREATE INDEX idx_villages_created_by ON villages (created_by);
CREATE INDEX idx_villages_updated_by ON villages (updated_by);
CREATE INDEX idx_villages_deleted_by ON villages (deleted_by);