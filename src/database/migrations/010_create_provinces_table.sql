CREATE TABLE provinces (
    id TEXT PRIMARY KEY,
    country_id TEXT NOT NULL,
    name VARCHAR(255) NOT NULL,
    code VARCHAR(10),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    FOREIGN KEY (country_id) REFERENCES countries(id) ON DELETE CASCADE
);

CREATE INDEX idx_provinces_country_id ON provinces(country_id);
CREATE INDEX idx_provinces_name ON provinces(name);
CREATE INDEX idx_provinces_code ON provinces(code);