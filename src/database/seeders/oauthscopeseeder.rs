use crate::database::DbPool;
use anyhow::Result;
use crate::database::seeder::Seeder;
use diesel::prelude::*;
use crate::schema::oauth_scopes;
use crate::app::models::oauth::Scope;

pub struct OAuthScopeSeeder;

impl Seeder for OAuthScopeSeeder {
    fn class_name(&self) -> &'static str {
        "OAuthScopeSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seeds OAuth scopes for Laravel Passport-like functionality")
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        tracing::info!("Running {} seeder", self.class_name());

        let mut conn = pool.get()?;

        // Check if scopes already exist to avoid duplicates
        let existing_count: i64 = oauth_scopes::table.count().get_result(&mut conn)?;
        if existing_count > 0 {
            tracing::info!("OAuth scopes already exist, skipping seeding");
            return Ok(());
        }

        // Define comprehensive OAuth scopes similar to Laravel Passport
        let scopes = vec![
            Scope::new(
                "*".to_string(),
                Some("Full access to all resources".to_string()),
                false,
            ),
            Scope::new(
                "read".to_string(),
                Some("Read access to user resources".to_string()),
                true,
            ),
            Scope::new(
                "write".to_string(),
                Some("Write access to user resources".to_string()),
                false,
            ),
            Scope::new(
                "admin".to_string(),
                Some("Administrative access to all resources".to_string()),
                false,
            ),
            Scope::new(
                "user:read".to_string(),
                Some("Read user profile information".to_string()),
                false,
            ),
            Scope::new(
                "user:write".to_string(),
                Some("Modify user profile information".to_string()),
                false,
            ),
            Scope::new(
                "user:delete".to_string(),
                Some("Delete user account".to_string()),
                false,
            ),
            Scope::new(
                "oauth:clients".to_string(),
                Some("Manage OAuth clients".to_string()),
                false,
            ),
            Scope::new(
                "oauth:tokens".to_string(),
                Some("Manage OAuth tokens".to_string()),
                false,
            ),
            Scope::new(
                "api:read".to_string(),
                Some("Read access to API resources".to_string()),
                false,
            ),
            Scope::new(
                "api:write".to_string(),
                Some("Write access to API resources".to_string()),
                false,
            ),
            Scope::new(
                "roles:read".to_string(),
                Some("Read roles and permissions".to_string()),
                false,
            ),
            Scope::new(
                "roles:write".to_string(),
                Some("Manage roles and permissions".to_string()),
                false,
            ),
            Scope::new(
                "organizations:read".to_string(),
                Some("Read organization information".to_string()),
                false,
            ),
            Scope::new(
                "organizations:write".to_string(),
                Some("Manage organization information".to_string()),
                false,
            ),
            Scope::new(
                "notifications:read".to_string(),
                Some("Read notifications".to_string()),
                false,
            ),
            Scope::new(
                "notifications:write".to_string(),
                Some("Send and manage notifications".to_string()),
                false,
            ),
            Scope::new(
                "files:read".to_string(),
                Some("Read file resources".to_string()),
                false,
            ),
            Scope::new(
                "files:write".to_string(),
                Some("Upload and manage files".to_string()),
                false,
            ),
            Scope::new(
                "analytics:read".to_string(),
                Some("Read analytics data".to_string()),
                false,
            ),
        ];

        // Insert scopes in batches for better performance
        for chunk in scopes.chunks(10) {
            diesel::insert_into(oauth_scopes::table)
                .values(chunk)
                .execute(&mut conn)?;
        }

        tracing::info!("Successfully seeded {} OAuth scopes", scopes.len());
        println!("OAuth scopes seeder executed successfully - {} scopes created", scopes.len());
        Ok(())
    }
}
