use crate::database::DbPool;
use anyhow::Result;
use ulid::Ulid;
use chrono::{Utc};
use crate::database::seeder::Seeder;
use crate::app::services::auth_service::AuthService;
use diesel::prelude::*;
use crate::schema::sys_users;

pub struct UserSeeder;

impl Seeder for UserSeeder {
    fn class_name(&self) -> &'static str {
        "UserSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seed default sys_users for the application")
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("🌱 Seeding sys_users...");

        let mut conn = pool.get()?;
        let now = Utc::now().naive_utc();

        // Create admin user
        let admin_id = Ulid::new().to_string();
        let admin_password = AuthService::hash_password("password")?;

        diesel::insert_into(sys_users::table)
            .values((
                sys_users::id.eq(&admin_id),
                sys_users::name.eq("Admin User"),
                sys_users::email.eq("admin@example.com"),
                sys_users::password.eq(&admin_password),
                sys_users::email_verified_at.eq(Some(now)),
                sys_users::created_at.eq(now),
                sys_users::updated_at.eq(now),
            ))
            .on_conflict(sys_users::email)
            .do_nothing()
            .execute(&mut conn)?;

        // Create regular user
        let user_id = Ulid::new().to_string();
        let user_password = AuthService::hash_password("password")?;

        diesel::insert_into(sys_users::table)
            .values((
                sys_users::id.eq(&user_id),
                sys_users::name.eq("Regular User"),
                sys_users::email.eq("user@example.com"),
                sys_users::password.eq(&user_password),
                sys_users::email_verified_at.eq(Some(now)),
                sys_users::created_at.eq(now),
                sys_users::updated_at.eq(now),
            ))
            .on_conflict(sys_users::email)
            .do_nothing()
            .execute(&mut conn)?;

        // Create test user
        let test_id = Ulid::new().to_string();
        let test_password = AuthService::hash_password("password")?;

        diesel::insert_into(sys_users::table)
            .values((
                sys_users::id.eq(&test_id),
                sys_users::name.eq("Test User"),
                sys_users::email.eq("test@example.com"),
                sys_users::password.eq(&test_password),
                sys_users::email_verified_at.eq::<Option<chrono::NaiveDateTime>>(None),
                sys_users::created_at.eq(now),
                sys_users::updated_at.eq(now),
            ))
            .on_conflict(sys_users::email)
            .do_nothing()
            .execute(&mut conn)?;

        println!("✅ Users seeded successfully!");
        Ok(())
    }
}
