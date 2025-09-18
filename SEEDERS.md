# Laravel-style Database Seeders for Rust Axum

This implementation provides a complete Laravel-like seeding system for the Rust Axum framework, allowing you to populate your database with test or production data in a consistent, organized manner.

## 🌟 Features

### ✅ **Laravel-Compatible Commands**
```bash
# Generate new seeders
cargo run --bin artisan -- make seeder UserSeeder

# Run all seeders
cargo run --bin artisan -- db:seed

# Run specific seeder
cargo run --bin artisan -- db:seed --class DatabaseSeeder

# Fresh seeding (reset DB + run seeders)
cargo run --bin artisan -- db:seed --fresh

# List available seeders
cargo run --bin artisan -- db:seed:list

# Migration with automatic seeding
cargo run --bin artisan -- migrate --seed

# Fresh migrations with seeding
cargo run --bin artisan -- migrate --fresh --seed

# Refresh migrations with seeding
cargo run --bin artisan -- migrate:refresh --seed
```

### ✅ **CSV Data Import**
- **Country/Province/City data** from CSV files
- **Relationship handling** with foreign key lookups
- **Geographic coordinates** support
- **Duplicate protection** with existence checks

### ✅ **Progress Tracking**
- **Execution timing** for each seeder
- **Progress indicators** with emojis
- **Descriptive output** with seeder descriptions
- **Error handling** with detailed messages

### ✅ **Registration System**
- **Automatic seeder discovery**
- **Type-safe seeder registry**
- **Description metadata** for each seeder

## 📁 Project Structure

```
src/database/
├── seeder.rs              # Core seeder trait and registry
├── seeders/
│   ├── mod.rs            # Module declarations
│   ├── databaseseeder.rs # Master seeder (runs all)
│   ├── countryseeder.rs  # Countries from CSV
│   ├── provinceseeder.rs # Provinces from CSV
│   ├── cityseeder.rs     # Cities from CSV
│   └── userseeder.rs     # Example user seeder
└── mod.rs                # Database module

data/seeders/
├── countries.csv         # 20 countries with ISO codes
├── provinces.csv         # 32 provinces/states
├── cities.csv           # 47 cities with coordinates
└── README.md            # CSV format documentation
```

## 🎯 Available Seeders

| Seeder | Description | Dependencies |
|--------|-------------|--------------|
| **DatabaseSeeder** | Runs all geographic data seeders in order | None |
| **CountrySeeder** | Seeds countries from CSV with ISO codes | None |
| **ProvinceSeeder** | Seeds provinces/states linked to countries | Countries |
| **CitySeeder** | Seeds cities with coordinates linked to provinces | Provinces |
| **UserSeeder** | Example template seeder | None |

## 🛠️ Creating Custom Seeders

### 1. Generate a New Seeder
```bash
cargo run --bin artisan -- make seeder PostSeeder
```

### 2. Implement the Seeder
```rust
use sqlx::PgPool;
use anyhow::Result;
use crate::database::seeder::Seeder;

pub struct Postseeder;

impl Seeder for Postseeder {
    fn name(&self) -> &'static str {
        "PostSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seeds blog posts with sample content")
    }

    async fn run(&self, pool: &PgPool) -> Result<()> {
        // Check if data already exists
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts")
            .fetch_one(pool)
            .await?;

        if count > 0 {
            println!("Posts already exist, skipping...");
            return Ok(());
        }

        // Insert sample data
        sqlx::query!(
            "INSERT INTO posts (title, content) VALUES ($1, $2)",
            "Sample Post",
            "This is sample content"
        )
        .execute(pool)
        .await?;

        println!("Successfully seeded posts");
        Ok(())
    }
}
```

### 3. Register the Seeder
Add your seeder to the registry in `src/database/seeder.rs`:

```rust
// In SeederType enum
pub enum SeederType {
    // ... existing variants
    Post,
}

// In the match statements
match self {
    // ... existing matches
    SeederType::Post => {
        let seeder = Postseeder;
        seeder.run(pool).await
    }
}

// In SeederRegistry::new()
registry.seeders.insert("PostSeeder".to_string(), SeederType::Post);
```

## 📊 CSV Data Format

### Countries (`countries.csv`)
```csv
name,iso_code,phone_code
United States,US,+1
Canada,CA,+1
```

### Provinces (`provinces.csv`)
```csv
country_iso,name,code
US,California,CA
CA,Ontario,ON
```

### Cities (`cities.csv`)
```csv
country_iso,province_code,name,code,latitude,longitude
US,CA,Los Angeles,LA,34.0522,-118.2437
CA,ON,Toronto,TOR,43.6532,-79.3832
```

## 🔄 Usage Examples

### Run All Geographic Data
```bash
cargo run --bin artisan -- db:seed --class DatabaseSeeder
```

### Fresh Database Seeding
```bash
# This will:
# 1. Drop all tables
# 2. Re-run migrations
# 3. Run all seeders
cargo run --bin artisan -- db:seed --fresh
```

### Migration with Automatic Seeding
```bash
# Run migrations and then all seeders
cargo run --bin artisan -- migrate --seed

# Fresh migrations with seeding (reset + migrate + seed)
cargo run --bin artisan -- migrate --fresh --seed

# Refresh migrations with seeding (reset + migrate + seed)
cargo run --bin artisan -- migrate:refresh --seed
```

### Run Individual Seeders
```bash
cargo run --bin artisan -- db:seed --class CountrySeeder
cargo run --bin artisan -- db:seed --class ProvinceSeeder
cargo run --bin artisan -- db:seed --class CitySeeder
```

## ⚡ Performance Features

- **Duplicate checking** to avoid re-seeding existing data
- **Bulk insert** support for large datasets
- **Execution timing** for performance monitoring
- **Transaction support** for data integrity

## 🔒 Security Features

- **SQL injection protection** with parameterized queries
- **Data validation** before insertion
- **Error handling** with rollback support
- **Type safety** with Rust's type system

## 🎨 Output Examples

```
🌱 Running specific seeder: DatabaseSeeder
Running seeder: DatabaseSeeder
Description: Runs all geographic data seeders in the correct order

🌱 Running CountrySeeder...
Running seeder: CountrySeeder
Description: Seeds countries table from CSV data
Seeding countries from CSV...
Successfully seeded 20 countries
Seeder CountrySeeder completed in 45ms

🌱 Running ProvinceSeeder...
Running seeder: ProvinceSeeder
Description: Seeds provinces table from CSV data, requires countries
Seeding provinces from CSV...
Successfully seeded 32 provinces
Seeder ProvinceSeeder completed in 38ms

🌱 Running CitySeeder...
Running seeder: CitySeeder
Description: Seeds cities table from CSV data with coordinates, requires provinces
Seeding cities from CSV...
Successfully seeded 47 cities
Seeder CitySeeder completed in 52ms

✅ All geographic data seeding completed successfully!
Seeder DatabaseSeeder completed in 137ms
✅ Seeding completed successfully!
```

This implementation provides a complete, production-ready seeding system that follows Laravel conventions while leveraging Rust's safety and performance benefits.