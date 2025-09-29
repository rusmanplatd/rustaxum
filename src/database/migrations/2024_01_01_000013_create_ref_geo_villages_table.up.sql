-- Create ref_geo_villages table
CREATE TABLE ref_geo_villages (
    id CHAR(26) PRIMARY KEY,
    district_id CHAR(26) NOT NULL,
    name VARCHAR NOT NULL,
    code VARCHAR,
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by_id CHAR(26) REFERENCES sys_users(id),
    updated_by_id CHAR(26) REFERENCES sys_users(id),
    deleted_by_id CHAR(26) REFERENCES sys_users(id),
    FOREIGN KEY (district_id) REFERENCES ref_geo_districts(id) ON DELETE CASCADE
);

-- Add indexes
CREATE INDEX idx_ref_geo_villages_name ON ref_geo_villages (name);
CREATE INDEX idx_ref_geo_villages_district_id ON ref_geo_villages (district_id);
CREATE INDEX idx_ref_geo_villages_code ON ref_geo_villages (code);
CREATE INDEX idx_ref_geo_villages_coordinates ON ref_geo_villages (latitude, longitude);
CREATE INDEX idx_ref_geo_villages_created_at ON ref_geo_villages (created_at);
CREATE INDEX idx_ref_geo_villages_created_by_id ON ref_geo_villages (created_by_id);
CREATE INDEX idx_ref_geo_villages_updated_by_id ON ref_geo_villages (updated_by_id);
CREATE INDEX idx_ref_geo_villages_deleted_by_id ON ref_geo_villages (deleted_by_id);