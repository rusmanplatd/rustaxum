use sqlx::PgPool;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::OnceLock;

pub trait Seeder {
    async fn run(&self, pool: &PgPool) -> Result<()>;
    fn name(&self) -> &'static str;
    fn description(&self) -> Option<&'static str> {
        None
    }
}

// Use an enum to work around async trait dyn compatibility issues
#[derive(Clone)]
pub enum SeederType {
    Country,
    Province,
    City,
    Database,
    User,
}

impl SeederType {
    pub async fn run(&self, pool: &PgPool) -> Result<()> {
        use crate::database::seeders::{
            countryseeder::Countryseeder,
            provinceseeder::Provinceseeder,
            cityseeder::Cityseeder,
            databaseseeder::Databaseseeder,
            userseeder::Userseeder,
        };

        match self {
            SeederType::Country => {
                let seeder = Countryseeder;
                seeder.run(pool).await
            }
            SeederType::Province => {
                let seeder = Provinceseeder;
                seeder.run(pool).await
            }
            SeederType::City => {
                let seeder = Cityseeder;
                seeder.run(pool).await
            }
            SeederType::Database => {
                let seeder = Databaseseeder;
                seeder.run(pool).await
            }
            SeederType::User => {
                let seeder = Userseeder;
                seeder.run(pool).await
            }
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            SeederType::Country => "CountrySeeder",
            SeederType::Province => "ProvinceSeeder",
            SeederType::City => "CitySeeder",
            SeederType::Database => "DatabaseSeeder",
            SeederType::User => "UserSeeder",
        }
    }

    pub fn description(&self) -> Option<&'static str> {
        match self {
            SeederType::Country => Some("Seeds countries table from CSV data"),
            SeederType::Province => Some("Seeds provinces table from CSV data, requires countries"),
            SeederType::City => Some("Seeds cities table from CSV data with coordinates, requires provinces"),
            SeederType::Database => Some("Runs all geographic data seeders in the correct order"),
            SeederType::User => Some("Example user seeder"),
        }
    }
}

pub struct SeederRegistry {
    seeders: HashMap<String, SeederType>,
}

impl SeederRegistry {
    fn new() -> Self {
        let mut registry = Self {
            seeders: HashMap::new(),
        };

        // Register all seeders
        registry.seeders.insert("CountrySeeder".to_string(), SeederType::Country);
        registry.seeders.insert("ProvinceSeeder".to_string(), SeederType::Province);
        registry.seeders.insert("CitySeeder".to_string(), SeederType::City);
        registry.seeders.insert("DatabaseSeeder".to_string(), SeederType::Database);
        registry.seeders.insert("UserSeeder".to_string(), SeederType::User);

        registry
    }

    pub fn get(&self, name: &str) -> Option<&SeederType> {
        self.seeders.get(name)
    }

    pub fn list(&self) -> Vec<&String> {
        self.seeders.keys().collect()
    }
}

static SEEDER_REGISTRY: OnceLock<SeederRegistry> = OnceLock::new();

pub fn registry() -> &'static SeederRegistry {
    SEEDER_REGISTRY.get_or_init(|| SeederRegistry::new())
}

pub async fn run_seeder(name: &str, pool: &PgPool) -> Result<()> {
    let registry = registry();

    match registry.get(name) {
        Some(seeder_type) => {
            println!("Running seeder: {}", seeder_type.name());
            if let Some(desc) = seeder_type.description() {
                println!("Description: {}", desc);
            }
            let start = std::time::Instant::now();
            seeder_type.run(pool).await?;
            let duration = start.elapsed();
            println!("Seeder {} completed in {:?}", seeder_type.name(), duration);
            Ok(())
        }
        None => {
            anyhow::bail!("Seeder '{}' not found", name);
        }
    }
}

pub async fn run_all_seeders(pool: &PgPool) -> Result<()> {
    // Run DatabaseSeeder if it exists, otherwise run all registered seeders
    if let Ok(()) = run_seeder("DatabaseSeeder", pool).await {
        return Ok(());
    }

    let registry = registry();
    let seeder_names: Vec<String> = registry.list().into_iter().cloned().collect();

    if seeder_names.is_empty() {
        println!("No seeders registered.");
        return Ok(());
    }

    println!("Running {} registered seeders...", seeder_names.len());

    for name in seeder_names {
        run_seeder(&name, pool).await?;
    }

    Ok(())
}

pub fn list_seeders() -> Vec<String> {
    let registry = registry();
    registry.list().into_iter().cloned().collect()
}