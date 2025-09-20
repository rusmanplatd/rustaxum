use sqlx::PgPool;
use anyhow::Result;
use crate::database::seeder::Seeder;
use crate::app::models::country::Country;
use csv::Reader;
use std::fs::File;
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CountryRecord {
    name: String,
    iso_code: String,
    phone_code: String,
}

pub struct Countryseeder;

impl Seeder for Countryseeder {
    fn class_name(&self) -> &'static str {
        "CountrySeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seeds countries table from CSV data")
    }

    async fn run(&self, pool: &PgPool) -> Result<()> {
        println!("Seeding countries from CSV...");

        // Check if countries already exist
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM countries")
            .fetch_one(pool)
            .await?;

        if count > 0 {
            println!("Countries table already has {} records. Skipping seeding.", count);
            return Ok(());
        }

        // Read CSV file
        let file = File::open("data/seeders/countries.csv")?;
        let mut rdr = Reader::from_reader(file);

        let mut inserted_count = 0;
        let mut country_map: HashMap<String, String> = HashMap::new();

        for result in rdr.deserialize() {
            let record: CountryRecord = result?;

            let country = Country::new(
                record.name.clone(),
                record.iso_code.clone(),
                Some(record.phone_code),
            );

            sqlx::query(
                "INSERT INTO countries (id, name, iso_code, phone_code, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6)"
            )
            .bind(country.id.to_string())
            .bind(country.name)
            .bind(country.iso_code)
            .bind(country.phone_code)
            .bind(country.created_at)
            .bind(country.updated_at)
            .execute(pool)
            .await?;

            country_map.insert(record.iso_code, country.id.to_string());
            inserted_count += 1;
        }

        println!("Successfully seeded {} countries", inserted_count);
        Ok(())
    }
}
