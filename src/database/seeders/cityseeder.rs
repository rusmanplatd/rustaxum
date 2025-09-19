use sqlx::PgPool;
use anyhow::{Result, anyhow};
use crate::database::seeder::Seeder;
use crate::app::models::city::City;
use csv::Reader;
use std::fs::File;
use std::collections::HashMap;
use serde::Deserialize;
use ulid::Ulid;
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
struct CityRecord {
    country_iso: String,
    province_code: String,
    name: String,
    code: String,
    latitude: String,
    longitude: String,
}

pub struct Cityseeder;

impl Seeder for Cityseeder {
    fn name(&self) -> &'static str {
        "CitySeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seeds cities table from CSV data with coordinates, requires provinces")
    }

    async fn run(&self, pool: &PgPool) -> Result<()> {
        println!("Seeding cities from CSV...");

        // Check if cities already exist
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cities")
            .fetch_one(pool)
            .await?;

        if count > 0 {
            println!("Cities table already has {} records. Skipping seeding.", count);
            return Ok(());
        }

        // Get province mappings from database with country info
        let provinces: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT p.code, p.id, c.iso_code
             FROM provinces p
             JOIN countries c ON p.country_id = c.id"
        )
        .fetch_all(pool)
        .await?;

        let province_map: HashMap<String, Ulid> = provinces
            .into_iter()
            .map(|(province_code, id, country_iso)| {
                let key = format!("{}:{}", country_iso, province_code);
                (key, Ulid::from_string(&id).unwrap())
            })
            .collect();

        // Read CSV file
        let file = File::open("data/seeders/cities.csv")?;
        let mut rdr = Reader::from_reader(file);

        let mut inserted_count = 0;

        for result in rdr.deserialize() {
            let record: CityRecord = result?;

            let key = format!("{}:{}", record.country_iso, record.province_code);
            let province_id = province_map.get(&key)
                .ok_or_else(|| anyhow!("Province with key {} not found", key))?;

            let latitude = if record.latitude.is_empty() {
                None
            } else {
                Some(Decimal::from_str(&record.latitude)?)
            };

            let longitude = if record.longitude.is_empty() {
                None
            } else {
                Some(Decimal::from_str(&record.longitude)?)
            };

            let city = City::new(
                *province_id,
                record.name.clone(),
                Some(record.code.clone()),
                latitude,
                longitude,
            );

            sqlx::query(
                "INSERT INTO cities (id, province_id, name, code, latitude, longitude, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
            )
            .bind(city.id.to_string())
            .bind(city.province_id.to_string())
            .bind(city.name)
            .bind(city.code)
            .bind(city.latitude)
            .bind(city.longitude)
            .bind(city.created_at)
            .bind(city.updated_at)
            .execute(pool)
            .await?;

            inserted_count += 1;
        }

        println!("Successfully seeded {} cities", inserted_count);
        Ok(())
    }
}
