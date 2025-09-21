use crate::database::DbPool;
use anyhow::Result;
use crate::database::seeder::Seeder;
use crate::app::models::country::{Country, NewCountry};
use csv::Reader;
use std::fs::File;
use std::collections::HashMap;
use serde::Deserialize;
use diesel::prelude::*;
use crate::schema::countries;

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

        // Check if countries already exist
        let count: i64 = countries::table
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
            );

            let inserted_country: Country = diesel::insert_into(countries::table)
                .values(&new_country)
                .get_result(&mut conn)?;

            country_map.insert(record.iso_code, inserted_country.id.clone());
            inserted_count += 1;
        }

        println!("Successfully seeded {} countries", inserted_count);
        Ok(())
    }
}
