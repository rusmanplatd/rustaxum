pub mod migration_runner;
pub mod seeder;
pub mod seeders;

use sqlx::{PgPool, postgres::PgPoolOptions};
use anyhow::Result;
use crate::config::Config;

pub async fn create_pool(config: &Config) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.database.pool_max_connections)
        .min_connections(config.database.pool_min_connections)
        .acquire_timeout(std::time::Duration::from_secs(config.database.pool_acquire_timeout_seconds))
        .idle_timeout(std::time::Duration::from_secs(config.database.pool_idle_timeout_seconds))
        .max_lifetime(std::time::Duration::from_secs(config.database.pool_max_lifetime_seconds))
        .connect(&config.database.url)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./src/database/migrations")
        .run(pool)
        .await?;

    Ok(())
}

