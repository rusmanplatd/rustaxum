use crate::database::DbPool;
use anyhow::{Result, anyhow};
use crate::database::seeder::Seeder;
use crate::app::models::province::Province;
use csv::Reader;
use std::fs::File;
use std::collections::HashMap;
use serde::Deserialize;
use ulid::Ulid;
use diesel::prelude::*;
use crate::schema::{provinces, countries};

#[derive(Debug, Deserialize)]
struct ProvinceRecord {
    country_iso: String,
    name: String,
    code: String,
}

pub struct Provinceseeder;

impl Seeder for Provinceseeder {
    fn class_name(&self) -> &'static str {
        "ProvinceSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seeds provinces table from CSV data, requires countries")
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("Seeding provinces from CSV...");
        let mut conn = pool.get()?;

        // Check if provinces already exist
        let count: i64 = provinces::table
            .count()
            .get_result(&mut conn)?;

        if count > 0 {
            println!("Provinces table already has {} records. Skipping seeding.", count);
            return Ok(());
        }

        // Get country mappings from database
        let countries: Vec<(String, String)> = countries::table
            .select((countries::iso_code, countries::id))
            .load::<(String, String)>(&mut conn)?;

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

            diesel::insert_into(provinces::table)
                .values((
                    provinces::id.eq(province.id.to_string()),
                    provinces::country_id.eq(province.country_id.to_string()),
                    provinces::name.eq(&province.name),
                    provinces::code.eq(&province.code),
                    provinces::created_at.eq(province.created_at),
                    provinces::updated_at.eq(province.updated_at),
                ))
                .execute(&mut conn)?;

            let key = format!("{}:{}", record.country_iso, record.code);
            province_map.insert(key, province.id.to_string());
            inserted_count += 1;
        }

        println!("Successfully seeded {} provinces", inserted_count);
        Ok(())
    }
}
