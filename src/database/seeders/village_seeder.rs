use crate::database::DbPool;
use anyhow::Result;
use crate::database::seeder::Seeder;
use crate::app::models::village::{Village, NewVillage};
use csv::Reader;
use std::fs::File;
use std::collections::HashMap;
use serde::Deserialize;
use ulid::Ulid;
use rust_decimal::Decimal;
use std::str::FromStr;
use diesel::prelude::*;
use crate::schema::ref_geo_villages;

#[derive(Debug, Deserialize)]
struct VillageRecord {
    country_iso: String,
    province_code: String,
    city_code: String,
    district_code: String,
    name: String,
    code: String,
    latitude: String,
    longitude: String,
}

pub struct Villageseeder;

impl Seeder for Villageseeder {
    fn class_name(&self) -> &'static str {
        "VillageSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seeds villages table from CSV data with coordinates, requires districts")
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("Seeding villages from CSV...");
        let mut conn = pool.get()?;

        // Check if villages already exist
        use diesel::sql_query;

        #[derive(QueryableByName)]
        struct CountResult {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            pub count: i64,
        }

        let count: i64 = sql_query("SELECT COUNT(*) as count FROM villages")
            .load::<CountResult>(&mut conn)?
            .first()
            .map(|r| r.count)
            .unwrap_or(0);

        if count > 0 {
            println!("Villages table already has {} records. Skipping seeding.", count);
            return Ok(());
        }

        // Get district mappings from database with city, province and country info
        #[derive(QueryableByName)]
        struct DistrictMapping {
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub code: String,
            #[diesel(sql_type = diesel::sql_types::Bpchar)]
            pub id: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub city_code: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub province_code: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub iso_code: String,
        }

        let districts: Vec<DistrictMapping> = sql_query(
            "SELECT d.code, d.id, ci.code as city_code, p.code as province_code, co.iso_code
             FROM districts d
             JOIN cities ci ON d.city_id = ci.id
             JOIN provinces p ON ci.province_id = p.id
             JOIN countries co ON p.country_id = co.id"
        )
        .load::<DistrictMapping>(&mut conn)?;

        let district_map: HashMap<String, Ulid> = districts
            .into_iter()
            .map(|dist| {
                let key = format!("{}:{}:{}:{}", dist.iso_code, dist.province_code, dist.city_code, dist.code);
                (key, Ulid::from_string(&dist.id).unwrap())
            })
            .collect();

        // Read CSV file
        let file = File::open("data/seeders/villages.csv")?;
        let mut rdr = Reader::from_reader(file);

        let mut inserted_count = 0;

        for result in rdr.deserialize() {
            let record: VillageRecord = result?;

            let key = format!("{}:{}:{}:{}", record.country_iso, record.province_code, record.city_code, record.district_code);
            let district_id = match district_map.get(&key) {
                Some(id) => id,
                None => {
                    println!("Skipping village '{}' - District with key {} not found", record.name, key);
                    continue;
                }
            };

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

            let new_village = NewVillage::new(
                district_id.to_string(),
                record.name.clone(),
                Some(record.code.clone()),
                latitude,
                longitude,
            );

            let _inserted_village: Village = diesel::insert_into(ref_geo_villages::table)
                .values(&new_village)
                .get_result(&mut conn)?;

            inserted_count += 1;
        }

        println!("Successfully seeded {} villages", inserted_count);
        Ok(())
    }
}