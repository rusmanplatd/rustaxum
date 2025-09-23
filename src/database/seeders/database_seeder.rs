use crate::database::DbPool;
use anyhow::Result;
use crate::database::seeder::{Seeder, SeederContext};
use crate::database::seeders::{
    user_seeder::UserSeeder,
    role_permission_seeder::RolePermissionSeeder,
    OAuthScopeSeeder,
    country_seeder::Countryseeder,
    province_seeder::Provinceseeder,
    city_seeder::Cityseeder,
    organization_seeder::OrganizationSeeder,
    OrganizationPositionLevelSeeder,
    OrganizationPositionSeeder,
    UserOrganizationSeeder,
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

        // Geographic data seeders
        context.call(Countryseeder)?;
        context.call(Provinceseeder)?;
        context.call(Cityseeder)?;

        // Organization and position structure
        context.call(OrganizationSeeder)?;
        context.call(OrganizationPositionLevelSeeder)?;
        context.call(OrganizationPositionSeeder)?;
        context.call(UserOrganizationSeeder)?;


        println!("───────────────────────────");
        println!("✅ Database seeding completed successfully!");
        Ok(())
    }
}
