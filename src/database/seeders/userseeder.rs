use sqlx::PgPool;
use anyhow::Result;
use bcrypt::{hash, DEFAULT_COST};
use ulid::Ulid;
use chrono::{Utc, NaiveDateTime};
use crate::database::seeder::Seeder;

pub struct UserSeeder;

impl Seeder for UserSeeder {
    fn name(&self) -> &'static str {
        "UserSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seeds default users for the application")
    }

    async fn run(&self, pool: &PgPool) -> Result<()> {
        println!("🌱 Seeding users...");

        let now = Utc::now().naive_utc();

        // Create admin user
        let admin_id = Ulid::new().to_string();
        let admin_password = hash("password", DEFAULT_COST)?;

        sqlx::query(
            r#"
            INSERT INTO users (id, name, email, password, email_verified_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (email) DO NOTHING
            "#
        )
        .bind(admin_id)
        .bind("Admin User")
        .bind("admin@example.com")
        .bind(admin_password)
        .bind(now)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // Create regular user
        let user_id = Ulid::new().to_string();
        let user_password = hash("password", DEFAULT_COST)?;

        sqlx::query(
            r#"
            INSERT INTO users (id, name, email, password, email_verified_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (email) DO NOTHING
            "#
        )
        .bind(user_id)
        .bind("Regular User")
        .bind("user@example.com")
        .bind(user_password)
        .bind(now)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // Create test user
        let test_id = Ulid::new().to_string();
        let test_password = hash("password", DEFAULT_COST)?;

        sqlx::query(
            r#"
            INSERT INTO users (id, name, email, password, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (email) DO NOTHING
            "#
        )
        .bind(test_id)
        .bind("Test User")
        .bind("test@example.com")
        .bind(test_password)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        println!("✅ Users seeded successfully!");
        Ok(())
    }
}
