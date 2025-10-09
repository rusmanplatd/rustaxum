use crate::app::models::organization_domain::NewOrganizationDomain;
use crate::app::models::DieselUlid;
use crate::database::seeder::Seeder;
use crate::database::DbPool;
use crate::schema::{organization_domains, sys_users};
use anyhow::Result;
use diesel::prelude::*;

pub struct OrganizationDomainSeeder;

impl Seeder for OrganizationDomainSeeder {
    fn class_name(&self) -> &'static str {
        "OrganizationDomainSeeder"
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("🌱 Seeding organization domains...");
        let mut conn = pool.get()?;

        // Get system user ID for audit tracking
        let system_user_id: String = sys_users::table
            .filter(sys_users::email.eq("system@seeder.internal"))
            .select(sys_users::id)
            .first(&mut conn)?;

        let domains = vec![
            (
                "GOV",
                "Government",
                "Government and public sector organizations",
            ),
            (
                "PVT",
                "Private Sector",
                "Private companies and corporations",
            ),
            (
                "NGO",
                "Non-Governmental",
                "Non-profit and NGO organizations",
            ),
            ("EDU", "Education", "Educational institutions"),
            ("HEA", "Healthcare", "Healthcare and medical institutions"),
            ("MIL", "Military", "Military and defense organizations"),
            (
                "REL",
                "Religious",
                "Religious organizations and institutions",
            ),
        ];

        for (code, name, description) in domains {
            let new_domain = NewOrganizationDomain {
                id: DieselUlid::new(),
                code: Some(code.to_string()),
                name: name.to_string(),
                description: Some(description.to_string()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                deleted_at: None,
                created_by_id: system_user_id.clone(),
                updated_by_id: system_user_id.clone(),
                deleted_by_id: None,
            };

            diesel::insert_into(organization_domains::table)
                .values(&new_domain)
                .on_conflict(organization_domains::code)
                .do_nothing()
                .execute(&mut conn)?;

            println!("   ✓ Created domain: {} ({})", name, code);
        }

        println!("✅ Organization domains seeded successfully!");
        Ok(())
    }
}
