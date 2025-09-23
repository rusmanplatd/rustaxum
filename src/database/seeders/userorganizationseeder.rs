use crate::database::DbPool;
use anyhow::Result;
use crate::database::seeder::Seeder;
use crate::app::models::{DieselUlid, user_organization::{NewUserOrganization}};
use diesel::prelude::*;
use crate::schema::{sys_users, organizations, organization_positions, user_organizations};
use chrono::Utc;

pub struct UserOrganizationSeeder;

impl Seeder for UserOrganizationSeeder {
    fn class_name(&self) -> &'static str {
        "UserOrganizationSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seed user organization relationships")
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("🌱 Seeding user organization relationships...");
        let mut conn = pool.get()?;

        // Get users, organizations, and positions
        let users: Vec<DieselUlid> = sys_users::table
            .select(sys_users::id)
            .load(&mut conn)?;

        let organizations: Vec<DieselUlid> = organizations::table
            .select(organizations::id)
            .load(&mut conn)?;

        let positions: Vec<DieselUlid> = organization_positions::table
            .select(organization_positions::id)
            .load(&mut conn)?;

        if users.is_empty() || organizations.is_empty() || positions.is_empty() {
            return Err(anyhow::anyhow!("Users, organizations, and positions must be seeded first"));
        }

        // Create user-organization relationships
        let mut user_org_relationships = Vec::new();
        let now = Utc::now();

        // Assign each user to different positions/organizations
        for (i, user_id) in users.iter().enumerate() {
            let org_id = organizations[i % organizations.len()];
            let position_id = positions[i % positions.len()];

            let user_org = NewUserOrganization {
                id: DieselUlid::new(),
                user_id: *user_id,
                organization_id: org_id,
                organization_position_id: position_id,
                is_active: true,
                started_at: now,
                ended_at: None,
                created_at: now,
                updated_at: now,
                deleted_at: None,
                created_by: None,
                updated_by: None,
                deleted_by: None,
            };

            user_org_relationships.push(user_org);
        }

        // Create additional relationships for some users (multiple positions)
        for i in 0..std::cmp::min(10, users.len()) {
            let user_id = users[i];
            let org_id = organizations[(i + 1) % organizations.len()];
            let position_id = positions[(i + 5) % positions.len()];

            let user_org = NewUserOrganization {
                id: DieselUlid::new(),
                user_id,
                organization_id: org_id,
                organization_position_id: position_id,
                is_active: i < 5, // Some are inactive
                started_at: now,
                ended_at: if i >= 5 { Some(now) } else { None },
                created_at: now,
                updated_at: now,
                deleted_at: None,
                created_by: None,
                updated_by: None,
                deleted_by: None,
            };

            user_org_relationships.push(user_org);
        }

        // Insert all relationships
        for user_org in user_org_relationships {
            diesel::insert_into(user_organizations::table)
                .values(&user_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
        }

        println!("✅ {} User Organization relationships seeded successfully!", users.len() + 10);
        Ok(())
    }
}
