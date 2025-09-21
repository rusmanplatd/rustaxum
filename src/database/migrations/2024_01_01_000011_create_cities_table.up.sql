-- Create cities table
CREATE TABLE cities (
    id CHAR(26) PRIMARY KEY,
    province_id CHAR(26) NOT NULL,
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
    FOREIGN KEY (province_id) REFERENCES provinces(id) ON DELETE CASCADE
);

-- Add indexes
CREATE INDEX idx_cities_name ON cities (name);
CREATE INDEX idx_cities_province_id ON cities (province_id);
CREATE INDEX idx_cities_code ON cities (code);
CREATE INDEX idx_cities_coordinates ON cities (latitude, longitude);
CREATE INDEX idx_cities_created_at ON cities (created_at);
CREATE INDEX idx_cities_created_by ON cities (created_by);
CREATE INDEX idx_cities_updated_by ON cities (updated_by);
CREATE INDEX idx_cities_deleted_by ON cities (deleted_by);
