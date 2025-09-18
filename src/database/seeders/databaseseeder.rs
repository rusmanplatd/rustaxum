use sqlx::PgPool;
use anyhow::Result;
use crate::database::seeder::Seeder;
use crate::database::seeders::{
    countryseeder::Countryseeder,
    provinceseeder::Provinceseeder,
    cityseeder::Cityseeder,
    userseeder::UserSeeder,
    rolepermissionseeder::RolePermissionSeeder,
    abacseeder::AbacSeeder,
};

pub struct Databaseseeder;

impl Seeder for Databaseseeder {
    fn name(&self) -> &'static str {
        "DatabaseSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Runs all seeders including users, RBAC, and ABAC data")
    }

    async fn run(&self, pool: &PgPool) -> Result<()> {
        println!("Running all database seeders...");

        // Run geographic data seeders first
        println!("\n🌱 Running CountrySeeder...");
        let country_seeder = Countryseeder;
        country_seeder.run(pool).await?;

        println!("\n🌱 Running ProvinceSeeder...");
        let province_seeder = Provinceseeder;
        province_seeder.run(pool).await?;

        println!("\n🌱 Running CitySeeder...");
        let city_seeder = Cityseeder;
        city_seeder.run(pool).await?;

        // Run user seeder
        println!("\n🌱 Running UserSeeder...");
        let user_seeder = UserSeeder;
        user_seeder.run(pool).await?;

        // Run RBAC seeder (roles and permissions)
        println!("\n🌱 Running RolePermissionSeeder...");
        let rbac_seeder = RolePermissionSeeder;
        rbac_seeder.run(pool).await?;

        // Run ABAC seeder (attributes and policies)
        println!("\n🌱 Running AbacSeeder...");
        let abac_seeder = AbacSeeder;
        abac_seeder.run(pool).await?;

        println!("\n✅ All database seeding completed successfully!");
        Ok(())
    }
}
