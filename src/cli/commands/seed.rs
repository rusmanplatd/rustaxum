use anyhow::Result;
use crate::config::Config;
use crate::database::create_pool;
use crate::database::seeder::{call, seed, all};
use crate::cli::commands::migrate::{handle_migrate_reset_command, handle_migrate_command};

pub async fn handle_seed_command(class: Option<String>, fresh: bool) -> Result<()> {
    dotenv::dotenv().ok();
    let config = Config::load()?;
    let pool = create_pool(&config).await?;

    if fresh {
        println!("🔄 Fresh seeding: Resetting database...");

        // Reset and re-run migrations
        handle_migrate_reset_command().await?;
        handle_migrate_command(false, false).await?;

        println!("✅ Database reset and migrations completed");
    }

    match class {
        Some(seeder_name) => {
            call(&seeder_name, &pool).await?;
        }
        None => {
            seed(&pool).await?;
        }
    }

    println!("✅ Seeding completed successfully!");
    Ok(())
}

pub async fn handle_seed_list_command() -> Result<()> {
    let seeders = all();

    if seeders.is_empty() {
        println!("No seeders available.");
        return Ok(());
    }

    println!("Available seeders:");
    println!("==================");

    for seeder_name in seeders {
        println!("📦 {}", seeder_name);
    }

    println!("\nUsage:");
    println!("  cargo run --bin artisan -- db:seed                    # Run all seeders");
    println!("  cargo run --bin artisan -- db:seed --class <SEEDER>   # Run specific seeder");
    println!("  cargo run --bin artisan -- db:seed --fresh            # Reset DB and seed");

    Ok(())
}