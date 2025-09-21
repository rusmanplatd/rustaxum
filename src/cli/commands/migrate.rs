use anyhow::Result;
use crate::{config, database};
use crate::database::migration_runner::MigrationRunner;
use crate::database::seeder::seed;

pub fn handle_migrate_command(fresh: bool, run_seed: bool) -> Result<()> {
    let config = config::Config::load()?;
    let pool = database::create_pool(&config)?;
    let runner = MigrationRunner::new(pool.clone(), "./src/database/migrations".to_string());

    if fresh {
        println!("🔄 Running fresh migrations (reset + migrate)...");
        runner.refresh_migrations()?;
    } else {
        println!("🚀 Running database migrations...");
        runner.run_migrations()?;
    }

    if run_seed {
        println!("\n🌱 Running seeders after migrations...");
        seed(&pool)?;
        println!("✅ Migrations and seeding completed successfully!");
    } else {
        println!("✅ Migrations completed successfully!");
    }

    Ok(())
}

pub fn handle_migrate_rollback_command(step: i32) -> Result<()> {
    println!("Rolling back {} migration batch(es)...", step);

    let config = config::Config::load()?;
    let pool = database::create_pool(&config)?;
    let runner = MigrationRunner::new(pool, "./src/database/migrations".to_string());

    runner.rollback_migrations(Some(step))?;
    Ok(())
}

pub fn handle_migrate_reset_command() -> Result<()> {
    println!("Resetting all migrations...");

    let config = config::Config::load()?;
    let pool = database::create_pool(&config)?;
    let runner = MigrationRunner::new(pool, "./src/database/migrations".to_string());

    runner.reset_migrations()?;
    Ok(())
}

pub fn handle_migrate_refresh_command(run_seed: bool) -> Result<()> {
    println!("🔄 Refreshing migrations (reset + migrate)...");

    let config = config::Config::load()?;
    let pool = database::create_pool(&config)?;
    let runner = MigrationRunner::new(pool.clone(), "./src/database/migrations".to_string());

    runner.refresh_migrations()?;

    if run_seed {
        println!("\n🌱 Running seeders after refresh...");
        seed(&pool)?;
        println!("✅ Migration refresh and seeding completed successfully!");
    } else {
        println!("✅ Migration refresh completed successfully!");
    }

    Ok(())
}

pub fn handle_migrate_status_command() -> Result<()> {
    let config = config::Config::load()?;
    let pool = database::create_pool(&config)?;
    let runner = MigrationRunner::new(pool, "./src/database/migrations".to_string());

    runner.show_status()?;
    Ok(())
}