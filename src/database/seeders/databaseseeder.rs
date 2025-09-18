use sqlx::PgPool;
use anyhow::Result;
use crate::database::seeder::Seeder;
use crate::database::seeders::{
    countryseeder::Countryseeder,
    provinceseeder::Provinceseeder,
    cityseeder::Cityseeder,
};

pub struct Databaseseeder;

impl Seeder for Databaseseeder {
    fn name(&self) -> &'static str {
        "DatabaseSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Runs all geographic data seeders in the correct order")
    }

    async fn run(&self, pool: &PgPool) -> Result<()> {
        println!("Running all database seeders...");

        // Run seeders in order (countries -> provinces -> cities) - directly instantiate to avoid recursion
        println!("\n🌱 Running CountrySeeder...");
        let country_seeder = Countryseeder;
        country_seeder.run(pool).await?;

        println!("\n🌱 Running ProvinceSeeder...");
        let province_seeder = Provinceseeder;
        province_seeder.run(pool).await?;

        println!("\n🌱 Running CitySeeder...");
        let city_seeder = Cityseeder;
        city_seeder.run(pool).await?;

        println!("\n✅ All geographic data seeding completed successfully!");
        Ok(())
    }
}
