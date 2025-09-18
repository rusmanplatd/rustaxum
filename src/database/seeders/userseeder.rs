use sqlx::PgPool;
use anyhow::Result;
use crate::database::seeder::Seeder;

pub struct Userseeder;

impl Seeder for Userseeder {
    fn name(&self) -> &'static str {
        "UserSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Example user seeder template")
    }

    async fn run(&self, _pool: &PgPool) -> Result<()> {
        // TODO: Add your seeding logic here
        // Example:
        // sqlx::query!(
        //     "INSERT INTO table_name (column1, column2) VALUES ($1, $2)",
        //     "value1",
        //     "value2"
        // )
        // .execute(pool)
        // .await?;

        println!("Userseeder seeder executed successfully");
        Ok(())
    }
}
