use sqlx::PgPool;
use anyhow::Result;
use crate::database::seeder::Seeder;

pub struct Testseeder;

impl Seeder for Testseeder {
    fn name(&self) -> &'static str {
        "Testseeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("TODO: Add description for this seeder")
    }

    async fn run(&self, pool: &PgPool) -> Result<()> {
        // TODO: Add your seeding logic here
        // Example:
        // sqlx::query!(
        //     "INSERT INTO table_name (column1, column2) VALUES ($1, $2)",
        //     "value1",
        //     "value2"
        // )
        // .execute(pool)
        // .await?;

        println!("Testseeder seeder executed successfully");
        Ok(())
    }
}
