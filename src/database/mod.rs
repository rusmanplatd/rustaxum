use sqlx::{PgPool, postgres::PgPoolOptions};
use anyhow::Result;
use crate::config::Config;

pub async fn create_pool(config: &Config) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./src/database/migrations")
        .run(pool)
        .await?;

    Ok(())
}