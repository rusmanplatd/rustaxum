-- Create ref_geo_provinces table
CREATE TABLE ref_geo_provinces (
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
    FOREIGN KEY (country_id) REFERENCES ref_geo_countries(id) ON DELETE CASCADE
);

-- Add indexes
CREATE INDEX idx_ref_geo_provinces_name ON ref_geo_provinces (name);
CREATE INDEX idx_ref_geo_provinces_country_id ON ref_geo_provinces (country_id);
CREATE INDEX idx_ref_geo_provinces_code ON ref_geo_provinces (code);
CREATE INDEX idx_ref_geo_provinces_created_at ON ref_geo_provinces (created_at);
CREATE INDEX idx_ref_geo_provinces_created_by_id ON ref_geo_provinces (created_by_id);
CREATE INDEX idx_ref_geo_provinces_updated_by_id ON ref_geo_provinces (updated_by_id);
CREATE INDEX idx_ref_geo_provinces_deleted_by_id ON ref_geo_provinces (deleted_by_id);
