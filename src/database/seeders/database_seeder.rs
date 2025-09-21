use crate::database::DbPool;
use anyhow::Result;
use crate::database::seeder::{Seeder, SeederContext};
use crate::database::seeders::{
    countryseeder::Countryseeder,
    provinceseeder::Provinceseeder,
    cityseeder::Cityseeder,
    userseeder::UserSeeder,
    rolepermissionseeder::RolePermissionSeeder,
    organizationseeder::OrganizationSeeder,
    joblevelpositionseeder::OrganizationPositionLevelSeeder,
};

pub struct Databaseseeder;

impl Seeder for Databaseseeder {
    fn class_name(&self) -> &'static str {
        "DatabaseSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Run all seeders including geographic data, users, RBAC, and ABAC")
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        let context = SeederContext::new(pool);

        println!("🌱 Database Seeding Started");
        println!("───────────────────────────");

        // User management
        context.call(UserSeeder)?;
        // Authorization systems
        context.call(RolePermissionSeeder)?;

        // Geographic data seeders (order matters due to foreign keys)
        context.call(Countryseeder)?;
        context.call(Provinceseeder)?;
        context.call(Cityseeder)?;

        // Organization and position structure
        context.call(OrganizationSeeder)?;
        context.call(OrganizationPositionLevelSeeder)?;


        println!("───────────────────────────");
        println!("✅ Database seeding completed successfully!");
        Ok(())
    }
}
