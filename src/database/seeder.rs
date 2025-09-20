use sqlx::PgPool;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::future::Future;

/// Base trait for database seeders, similar to Laravel's Seeder class
pub trait Seeder {
    /// Run the database seeds
    fn run(&self, pool: &PgPool) -> impl Future<Output = Result<()>> + Send;

    /// Get the seeder class name
    fn class_name(&self) -> &'static str;

    /// Get an optional description of what this seeder does
    fn description(&self) -> Option<&'static str> {
        None
    }
}

/// Seeder context provides Laravel-like helper methods for seeders
pub struct SeederContext<'a> {
    pool: &'a PgPool,
}

impl<'a> SeederContext<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Call another seeder class (Laravel's $this->call() equivalent)
    pub async fn call<T: Seeder>(&self, seeder: T) -> Result<()> {
        println!("🌱 Seeding: {}", seeder.class_name());
        let start = std::time::Instant::now();

        seeder.run(self.pool).await?;

        let duration = start.elapsed();
        println!("✅ Seeded: {} ({:?})", seeder.class_name(), duration);
        Ok(())
    }

    /// Call multiple seeder classes
    pub async fn call_many<T: Seeder>(&self, seeders: Vec<T>) -> Result<()> {
        for seeder in seeders {
            self.call(seeder).await?;
        }
        Ok(())
    }

    /// Get the database pool reference
    pub fn db(&self) -> &PgPool {
        self.pool
    }
}

/// Seeder registry for dynamic seeder management
#[derive(Clone)]
pub enum RegisteredSeeder {
    Country,
    Province,
    City,
    Database,
    User,
    RolePermission,
    Abac,
    Organization,
    JobLevelPosition,
}

impl RegisteredSeeder {
    pub async fn run(&self, pool: &PgPool) -> Result<()> {
        use crate::database::seeders::{
            countryseeder::Countryseeder,
            provinceseeder::Provinceseeder,
            cityseeder::Cityseeder,
            databaseseeder::Databaseseeder,
            userseeder::UserSeeder,
            rolepermissionseeder::RolePermissionSeeder,
            abacseeder::AbacSeeder,
            organizationseeder::OrganizationSeeder,
            joblevelpositionseeder::JobLevelPositionSeeder,
        };

        match self {
            RegisteredSeeder::Country => {
                let seeder = Countryseeder;
                seeder.run(pool).await
            }
            RegisteredSeeder::Province => {
                let seeder = Provinceseeder;
                seeder.run(pool).await
            }
            RegisteredSeeder::City => {
                let seeder = Cityseeder;
                seeder.run(pool).await
            }
            RegisteredSeeder::Database => {
                let seeder = Databaseseeder;
                seeder.run(pool).await
            }
            RegisteredSeeder::User => {
                let seeder = UserSeeder;
                seeder.run(pool).await
            }
            RegisteredSeeder::RolePermission => {
                let seeder = RolePermissionSeeder;
                seeder.run(pool).await
            }
            RegisteredSeeder::Abac => {
                let seeder = AbacSeeder;
                seeder.run(pool).await
            }
            RegisteredSeeder::Organization => {
                let seeder = OrganizationSeeder;
                seeder.run(pool).await
            }
            RegisteredSeeder::JobLevelPosition => {
                let seeder = JobLevelPositionSeeder;
                seeder.run(pool).await
            }
        }
    }

    pub fn class_name(&self) -> &'static str {
        match self {
            RegisteredSeeder::Country => "CountrySeeder",
            RegisteredSeeder::Province => "ProvinceSeeder",
            RegisteredSeeder::City => "CitySeeder",
            RegisteredSeeder::Database => "DatabaseSeeder",
            RegisteredSeeder::User => "UserSeeder",
            RegisteredSeeder::RolePermission => "RolePermissionSeeder",
            RegisteredSeeder::Abac => "AbacSeeder",
            RegisteredSeeder::Organization => "OrganizationSeeder",
            RegisteredSeeder::JobLevelPosition => "JobLevelPositionSeeder",
        }
    }

    pub fn description(&self) -> Option<&'static str> {
        match self {
            RegisteredSeeder::Country => Some("Seed countries table from CSV data"),
            RegisteredSeeder::Province => Some("Seed provinces table from CSV data, requires countries"),
            RegisteredSeeder::City => Some("Seed cities table from CSV data with coordinates, requires provinces"),
            RegisteredSeeder::Database => Some("Run all seeders in the correct order"),
            RegisteredSeeder::User => Some("Seed default users"),
            RegisteredSeeder::RolePermission => Some("Seed roles and permissions for RBAC"),
            RegisteredSeeder::Abac => Some("Seed attributes and policies for ABAC"),
            RegisteredSeeder::Organization => Some("Seed organization data"),
            RegisteredSeeder::JobLevelPosition => Some("Seed job levels and positions"),
        }
    }
}

/// Seeder registry manages all available seeders
pub struct SeederRegistry {
    seeders: HashMap<String, RegisteredSeeder>,
}

impl SeederRegistry {
    fn new() -> Self {
        let mut registry = Self {
            seeders: HashMap::new(),
        };

        // Auto-register all available seeders
        registry.register_seeder("CountrySeeder", RegisteredSeeder::Country);
        registry.register_seeder("ProvinceSeeder", RegisteredSeeder::Province);
        registry.register_seeder("CitySeeder", RegisteredSeeder::City);
        registry.register_seeder("DatabaseSeeder", RegisteredSeeder::Database);
        registry.register_seeder("UserSeeder", RegisteredSeeder::User);
        registry.register_seeder("RolePermissionSeeder", RegisteredSeeder::RolePermission);
        registry.register_seeder("AbacSeeder", RegisteredSeeder::Abac);
        registry.register_seeder("OrganizationSeeder", RegisteredSeeder::Organization);
        registry.register_seeder("JobLevelPositionSeeder", RegisteredSeeder::JobLevelPosition);

        registry
    }

    fn register_seeder(&mut self, name: &str, seeder: RegisteredSeeder) {
        self.seeders.insert(name.to_string(), seeder);
    }

    pub fn find(&self, class_name: &str) -> Option<&RegisteredSeeder> {
        self.seeders.get(class_name)
    }

    pub fn all(&self) -> Vec<&String> {
        self.seeders.keys().collect()
    }

    pub fn exists(&self, class_name: &str) -> bool {
        self.seeders.contains_key(class_name)
    }
}

static SEEDER_REGISTRY: OnceLock<SeederRegistry> = OnceLock::new();

/// Get the global seeder registry
pub fn registry() -> &'static SeederRegistry {
    SEEDER_REGISTRY.get_or_init(|| SeederRegistry::new())
}

/// Run a specific seeder by class name
pub async fn call(class_name: &str, pool: &PgPool) -> Result<()> {
    let registry = registry();

    match registry.find(class_name) {
        Some(seeder) => {
            println!("🌱 Seeding: {}", seeder.class_name());
            if let Some(desc) = seeder.description() {
                println!("   {}", desc);
            }
            let start = std::time::Instant::now();
            seeder.run(pool).await?;
            let duration = start.elapsed();
            println!("✅ Seeded: {} ({:?})", seeder.class_name(), duration);
            Ok(())
        }
        None => {
            anyhow::bail!("Seeder class '{}' not found", class_name);
        }
    }
}

/// Run all seeders using DatabaseSeeder as entry point (Laravel approach)
pub async fn seed(pool: &PgPool) -> Result<()> {
    // Try to run DatabaseSeeder first (Laravel convention)
    if registry().exists("DatabaseSeeder") {
        return call("DatabaseSeeder", pool).await;
    }

    // Fallback: run all registered seeders if no DatabaseSeeder exists
    let registry = registry();
    let seeder_names: Vec<String> = registry.all().into_iter().cloned().collect();

    if seeder_names.is_empty() {
        println!("No seeders registered.");
        return Ok(());
    }

    println!("🌱 Running {} registered seeders...", seeder_names.len());

    for name in seeder_names {
        if name != "DatabaseSeeder" { // Avoid infinite recursion
            call(&name, pool).await?;
        }
    }

    println!("✅ Database seeding completed!");
    Ok(())
}

/// List all available seeder classes
pub fn all() -> Vec<String> {
    let registry = registry();
    registry.all().into_iter().cloned().collect()
}