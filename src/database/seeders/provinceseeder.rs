use sqlx::PgPool;
use anyhow::{Result, anyhow};
use crate::database::seeder::Seeder;
use crate::app::models::province::Province;
use csv::Reader;
use std::fs::File;
use std::collections::HashMap;
use serde::Deserialize;
use ulid::Ulid;

#[derive(Debug, Deserialize)]
struct ProvinceRecord {
    country_iso: String,
    name: String,
    code: String,
}

pub struct Provinceseeder;

impl Seeder for Provinceseeder {
    fn name(&self) -> &'static str {
        "ProvinceSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seeds provinces table from CSV data, requires countries")
    }

    async fn run(&self, pool: &PgPool) -> Result<()> {
        println!("Seeding provinces from CSV...");

        // Check if provinces already exist
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM provinces")
            .fetch_one(pool)
            .await?;

        if count > 0 {
            println!("Provinces table already has {} records. Skipping seeding.", count);
            return Ok(());
        }

        // Get country mappings from database
        let countries: Vec<(String, String)> = sqlx::query_as("SELECT iso_code, id FROM countries")
            .fetch_all(pool)
            .await?;

        let country_map: HashMap<String, Ulid> = countries
            .into_iter()
            .map(|(iso, id)| (iso, Ulid::from_string(&id).unwrap()))
            .collect();

        // Read CSV file
        let file = File::open("data/seeders/provinces.csv")?;
        let mut rdr = Reader::from_reader(file);

        let mut inserted_count = 0;
        let mut province_map: HashMap<String, String> = HashMap::new();

        for result in rdr.deserialize() {
            let record: ProvinceRecord = result?;

            let country_id = country_map.get(&record.country_iso)
                .ok_or_else(|| anyhow!("Country with ISO code {} not found", record.country_iso))?;

            let province = Province::new(
                *country_id,
                record.name.clone(),
                Some(record.code.clone()),
            );

            sqlx::query!(
                "INSERT INTO provinces (id, country_id, name, code, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6)",
                province.id.to_string(),
                province.country_id.to_string(),
                province.name,
                province.code,
                province.created_at,
                province.updated_at
            )
            .execute(pool)
            .await?;

            let key = format!("{}:{}", record.country_iso, record.code);
            province_map.insert(key, province.id.to_string());
            inserted_count += 1;
        }

        println!("Successfully seeded {} provinces", inserted_count);
        Ok(())
    }
}
