use sqlx::PgPool;
use anyhow::Result;
use crate::database::seeder::{Seeder, SeederContext};
use crate::database::seeders::{
    countryseeder::Countryseeder,
    provinceseeder::Provinceseeder,
    cityseeder::Cityseeder,
    userseeder::UserSeeder,
    rolepermissionseeder::RolePermissionSeeder,
    abacseeder::AbacSeeder,
    organizationseeder::OrganizationSeeder,
    joblevelpositionseeder::JobLevelPositionSeeder,
};

pub struct Databaseseeder;

impl Seeder for Databaseseeder {
    fn class_name(&self) -> &'static str {
        "DatabaseSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Run all seeders including geographic data, users, RBAC, and ABAC")
    }

    async fn run(&self, pool: &PgPool) -> Result<()> {
        let context = SeederContext::new(pool);

        println!("🌱 Database Seeding Started");
        println!("───────────────────────────");

        // Geographic data seeders (order matters due to foreign keys)
        context.call(Countryseeder).await?;
        context.call(Provinceseeder).await?;
        context.call(Cityseeder).await?;

        // Organization and job structure
        context.call(OrganizationSeeder).await?;
        context.call(JobLevelPositionSeeder).await?;

        // User management
        context.call(UserSeeder).await?;

        // Authorization systems
        context.call(RolePermissionSeeder).await?;
        context.call(AbacSeeder).await?;

        println!("───────────────────────────");
        println!("✅ Database seeding completed successfully!");
        Ok(())
    }
}
