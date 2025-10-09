-- Create ref_geo_districts table
CREATE TABLE ref_geo_districts (
    id CHAR(26) PRIMARY KEY,
    city_id CHAR(26) NOT NULL,
    name VARCHAR NOT NULL,
    code VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    created_by_id CHAR(26) NOT NULL REFERENCES sys_users(id),
    updated_by_id CHAR(26) NOT NULL REFERENCES sys_users(id),
    deleted_by_id CHAR(26) REFERENCES sys_users(id),
    FOREIGN KEY (city_id) REFERENCES ref_geo_cities(id) ON DELETE CASCADE
);

-- Add indexes
CREATE INDEX idx_ref_geo_districts_name ON ref_geo_districts (name);
CREATE INDEX idx_ref_geo_districts_city_id ON ref_geo_districts (city_id);
CREATE INDEX idx_ref_geo_districts_code ON ref_geo_districts (code);
CREATE INDEX idx_ref_geo_districts_created_at ON ref_geo_districts (created_at);
CREATE INDEX idx_ref_geo_districts_created_by_id ON ref_geo_districts (created_by_id);
CREATE INDEX idx_ref_geo_districts_updated_by_id ON ref_geo_districts (updated_by_id);
CREATE INDEX idx_ref_geo_districts_deleted_by_id ON ref_geo_districts (deleted_by_id);