-- Create ref_geo_cities table
CREATE TABLE ref_geo_cities (
    id CHAR(26) PRIMARY KEY,
    province_id CHAR(26) NOT NULL,
    name VARCHAR NOT NULL,
    code VARCHAR,
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by_id CHAR(26) NOT NULL REFERENCES sys_users(id),
    updated_by_id CHAR(26) NOT NULL REFERENCES sys_users(id),
    deleted_by_id CHAR(26) REFERENCES sys_users(id),
    FOREIGN KEY (province_id) REFERENCES ref_geo_provinces(id) ON DELETE CASCADE
);

-- Add indexes
CREATE INDEX idx_ref_geo_cities_name ON ref_geo_cities (name);
CREATE INDEX idx_ref_geo_cities_province_id ON ref_geo_cities (province_id);
CREATE INDEX idx_ref_geo_cities_code ON ref_geo_cities (code);
CREATE INDEX idx_ref_geo_cities_coordinates ON ref_geo_cities (latitude, longitude);
CREATE INDEX idx_ref_geo_cities_created_at ON ref_geo_cities (created_at);
CREATE INDEX idx_ref_geo_cities_created_by_id ON ref_geo_cities (created_by_id);
CREATE INDEX idx_ref_geo_cities_updated_by_id ON ref_geo_cities (updated_by_id);
CREATE INDEX idx_ref_geo_cities_deleted_by_id ON ref_geo_cities (deleted_by_id);
