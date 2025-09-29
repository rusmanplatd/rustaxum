use crate::database::DbPool;
use anyhow::{Result, anyhow};
use crate::database::seeder::Seeder;
use crate::app::models::city::NewCity;
use csv::Reader;
use std::fs::File;
use std::collections::HashMap;
use serde::Deserialize;
use ulid::Ulid;
use rust_decimal::Decimal;
use std::str::FromStr;
use diesel::prelude::*;
use crate::schema::ref_geo_cities;

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
    fn class_name(&self) -> &'static str {
        "CitySeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seeds cities table from CSV data with coordinates, requires provinces")
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("Seeding cities from CSV...");
        let mut conn = pool.get()?;

        // Check if cities already exist
        use diesel::sql_query;

        #[derive(QueryableByName)]
        struct CountResult {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            pub count: i64,
        }

        let count: i64 = sql_query("SELECT COUNT(*) as count FROM cities")
            .load::<CountResult>(&mut conn)?
            .first()
            .map(|r| r.count)
            .unwrap_or(0);

        if count > 0 {
            println!("Cities table already has {} records. Skipping seeding.", count);
            return Ok(());
        }

        // Get province mappings from database with country info
        #[derive(QueryableByName)]
        struct ProvinceMapping {
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub code: String,
            #[diesel(sql_type = diesel::sql_types::Bpchar)]
            pub id: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub iso_code: String,
        }

        let provinces: Vec<ProvinceMapping> = sql_query(
            "SELECT p.code, p.id, c.iso_code
             FROM provinces p
             JOIN countries c ON p.country_id = c.id"
        )
        .load::<ProvinceMapping>(&mut conn)?;

        let province_map: HashMap<String, Ulid> = provinces
            .into_iter()
            .map(|prov| {
                let key = format!("{}:{}", prov.iso_code, prov.code);
                (key, Ulid::from_string(&prov.id).unwrap())
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

            let new_city = NewCity::new(
                province_id.to_string(),
                record.name.clone(),
                Some(record.code.clone()),
                latitude,
                longitude,
            );

            diesel::insert_into(ref_geo_cities::table)
                .values(&new_city)
                .execute(&mut conn)?;

            inserted_count += 1;
        }

        println!("Successfully seeded {} cities", inserted_count);
        Ok(())
    }
}
