CREATE TABLE cities (
    id TEXT PRIMARY KEY,
    province_id TEXT NOT NULL,
    name VARCHAR(255) NOT NULL,
    code VARCHAR(10),
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    FOREIGN KEY (province_id) REFERENCES provinces(id) ON DELETE CASCADE
);

CREATE INDEX idx_cities_province_id ON cities(province_id);
CREATE INDEX idx_cities_name ON cities(name);
CREATE INDEX idx_cities_code ON cities(code);
CREATE INDEX idx_cities_coordinates ON cities(latitude, longitude);