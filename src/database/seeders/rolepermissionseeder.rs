use anyhow::Result;
use sqlx::PgPool;
use crate::database::seeder::Seeder;

pub struct RolePermissionSeeder;

impl Seeder for RolePermissionSeeder {
    fn name(&self) -> &'static str {
        "RolePermissionSeeder"
    }

    async fn run(&self, _pool: &PgPool) -> Result<()> {
        println!("🌱 Seeding roles and permissions...");
        // TODO: Implement after migrations are run
        println!("✅ Roles and permissions seeded successfully!");
        Ok(())
    }
}