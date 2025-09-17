use anyhow::Result;
use crate::{config, database};
use crate::database::migration_runner::MigrationRunner;

pub async fn handle_migrate_command(fresh: bool) -> Result<()> {
    let config = config::Config::from_env()?;
    let pool = database::create_pool(&config).await?;
    let runner = MigrationRunner::new(pool, "./src/database/migrations".to_string());

    if fresh {
        println!("Running fresh migrations (reset + migrate)...");
        runner.refresh_migrations().await?;
    } else {
        println!("Running database migrations...");
        runner.run_migrations().await?;
    }

    Ok(())
}

pub async fn handle_migrate_rollback_command(step: i32) -> Result<()> {
    println!("Rolling back {} migration batch(es)...", step);

    let config = config::Config::from_env()?;
    let pool = database::create_pool(&config).await?;
    let runner = MigrationRunner::new(pool, "./src/database/migrations".to_string());

    runner.rollback_migrations(Some(step)).await?;
    Ok(())
}

pub async fn handle_migrate_reset_command() -> Result<()> {
    println!("Resetting all migrations...");

    let config = config::Config::from_env()?;
    let pool = database::create_pool(&config).await?;
    let runner = MigrationRunner::new(pool, "./src/database/migrations".to_string());

    runner.reset_migrations().await?;
    Ok(())
}

pub async fn handle_migrate_refresh_command() -> Result<()> {
    println!("Refreshing migrations (reset + migrate)...");

    let config = config::Config::from_env()?;
    let pool = database::create_pool(&config).await?;
    let runner = MigrationRunner::new(pool, "./src/database/migrations".to_string());

    runner.refresh_migrations().await?;
    Ok(())
}

pub async fn handle_migrate_status_command() -> Result<()> {
    let config = config::Config::from_env()?;
    let pool = database::create_pool(&config).await?;
    let runner = MigrationRunner::new(pool, "./src/database/migrations".to_string());

    runner.show_status().await?;
    Ok(())
}