use crate::database::DbPool;
use anyhow::Result;
use crate::database::seeder::{Seeder, SeederContext};
use crate::database::seeders::{
    country_seeder::Countryseeder,
    province_seeder::Provinceseeder,
    city_seeder::Cityseeder,
    user_seeder::UserSeeder,
    role_permission_seeder::RolePermissionSeeder,
    organization_seeder::OrganizationSeeder,
    organization_position_level_seeder::OrganizationPositionLevelSeeder,
    OAuthScopeSeeder,
};

pub struct DatabaseSeeder;

impl Seeder for DatabaseSeeder {
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
        // OAuth2 scopes
        context.call(OAuthScopeSeeder)?;

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
