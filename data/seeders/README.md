# Database Seeders

This directory contains CSV files for seeding the database with country, province, and city data.

## CSV Files

- `countries.csv` - Contains country data with ISO codes and phone codes
- `provinces.csv` - Contains province/state data linked to countries
- `cities.csv` - Contains city data with coordinates, linked to provinces

## CSV Format

### countries.csv
```csv
name,iso_code,phone_code
United States,US,+1
Canada,CA,+1
```

### provinces.csv
```csv
country_iso,name,code
US,California,CA
US,Texas,TX
```

### cities.csv
```csv
country_iso,province_code,name,code,latitude,longitude
US,CA,Los Angeles,LA,34.0522,-118.2437
US,CA,San Francisco,SF,37.7749,-122.4194
```

## Usage

### Generate new seeders
```bash
# Generate a new seeder
cargo run --bin artisan -- make seeder CountrySeeder

# Generate seeder with custom logic
cargo run --bin artisan -- make seeder CustomSeeder
```

### Run seeders
```bash
# Run all seeders (DatabaseSeeder)
cargo run --bin artisan -- db:seed

# Run specific seeder
cargo run --bin artisan -- db:seed --class CountrySeeder

# Run the complete geography data seeding
cargo run --bin artisan -- db:seed --class DatabaseSeeder
```

## Available Seeders

- **CountrySeeder** - Seeds countries from `countries.csv`
- **ProvinceSeeder** - Seeds provinces from `provinces.csv` (requires countries)
- **CitySeeder** - Seeds cities from `cities.csv` (requires provinces)
- **DatabaseSeeder** - Runs all geography seeders in correct order

## Seeder Features

- **Duplicate protection** - Seeders check if data already exists before inserting
- **Foreign key relationships** - Provinces link to countries, cities link to provinces
- **CSV validation** - Proper error handling for malformed CSV data
- **Coordinate support** - Cities include latitude/longitude from CSV