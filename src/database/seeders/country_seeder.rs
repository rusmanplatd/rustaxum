use crate::database::DbPool;
use anyhow::Result;
use crate::database::seeder::Seeder;
use crate::app::models::country::{Country, NewCountry};
use csv::Reader;
use std::fs::File;
use std::collections::HashMap;
use serde::Deserialize;
use diesel::prelude::*;
use crate::schema::{ref_geo_countries, sys_users};

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

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("Seeding countries from CSV...");
        let mut conn = pool.get()?;

        // Get system user ID for audit tracking
        let system_user_id: String = sys_users::table
            .filter(sys_users::email.eq("system@seeder.internal"))
            .select(sys_users::id)
            .first(&mut conn)?;

        // Check if countries already exist
        let count: i64 = ref_geo_countries::table
            .count()
            .get_result(&mut conn)?;

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

            let new_country = NewCountry::new(
                record.name.clone(),
                record.iso_code.clone(),
                Some(record.phone_code),
                Some(&system_user_id),
            );

            let inserted_country: Country = diesel::insert_into(ref_geo_countries::table)
                .values(&new_country)
                .get_result(&mut conn)?;

            country_map.insert(record.iso_code, inserted_country.id.to_string());
            inserted_count += 1;
        }

        println!("Successfully seeded {} countries", inserted_count);
        Ok(())
    }
}
