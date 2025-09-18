CREATE TABLE countries (
    id TEXT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    iso_code VARCHAR(3) NOT NULL UNIQUE,
    phone_code VARCHAR(10),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_countries_iso_code ON countries(iso_code);
CREATE INDEX idx_countries_name ON countries(name);