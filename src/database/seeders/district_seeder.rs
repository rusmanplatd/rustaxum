use crate::database::DbPool;
use anyhow::Result;
use crate::database::seeder::Seeder;
use crate::app::models::district::{District, NewDistrict};
use csv::Reader;
use std::fs::File;
use std::collections::HashMap;
use serde::Deserialize;
use ulid::Ulid;
use diesel::prelude::*;
use crate::schema::{ref_geo_districts, sys_users};

#[derive(Debug, Deserialize)]
struct DistrictRecord {
    country_iso: String,
    province_code: String,
    city_code: String,
    name: String,
    code: String,
}

pub struct Districtseeder;

impl Seeder for Districtseeder {
    fn class_name(&self) -> &'static str {
        "DistrictSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seeds districts table from CSV data, requires cities")
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("Seeding districts from CSV...");
        let mut conn = pool.get()?;

        // Get system user ID for audit tracking
        let system_user_id: String = sys_users::table
            .filter(sys_users::email.eq("system@seeder.internal"))
            .select(sys_users::id)
            .first(&mut conn)?;

        // Check if districts already exist
        let count: i64 = ref_geo_districts::table
            .count()
            .get_result(&mut conn)?;

        if count > 0 {
            println!("Districts table already has {} records. Skipping seeding.", count);
            return Ok(());
        }

        // Get city mappings from database with province and country info
        use diesel::sql_query;

        #[derive(QueryableByName)]
        struct CityMapping {
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub code: String,
            #[diesel(sql_type = diesel::sql_types::Bpchar)]
            pub id: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub province_code: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub iso_code: String,
        }

        let cities: Vec<CityMapping> = sql_query(
            "SELECT ci.code, ci.id, p.code as province_code, co.iso_code
             FROM ref_geo_cities ci
             JOIN ref_geo_provinces p ON ci.province_id = p.id
             JOIN ref_geo_countries co ON p.country_id = co.id"
        )
        .load::<CityMapping>(&mut conn)?;

        let city_map: HashMap<String, Ulid> = cities
            .into_iter()
            .map(|city| {
                let key = format!("{}:{}:{}", city.iso_code, city.province_code, city.code);
                (key, Ulid::from_string(&city.id).unwrap())
            })
            .collect();

        // Read CSV file
        let file = File::open("data/seeders/districts.csv")?;
        let mut rdr = Reader::from_reader(file);

        let mut inserted_count = 0;

        for result in rdr.deserialize() {
            let record: DistrictRecord = result?;

            let key = format!("{}:{}:{}", record.country_iso, record.province_code, record.city_code);
            let city_id = match city_map.get(&key) {
                Some(id) => id,
                None => {
                    println!("Skipping district '{}' - City with key {} not found", record.name, key);
                    continue;
                }
            };

            let new_district = NewDistrict::new(
                city_id.to_string(),
                record.name.clone(),
                Some(record.code.clone()),
                Some(&system_user_id),
            );

            let _inserted_district: District = diesel::insert_into(ref_geo_districts::table)
                .values(&new_district)
                .get_result(&mut conn)?;

            inserted_count += 1;
        }

        println!("Successfully seeded {} districts", inserted_count);
        Ok(())
    }
}