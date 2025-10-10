use crate::database::DbPool;
use anyhow::Result;
use crate::database::seeder::Seeder;
use crate::app::models::{DieselUlid, organization_position_level::{CreateOrganizationPositionLevel, OrganizationPositionLevel}};
use diesel::prelude::*;
use crate::schema::{organizations, organization_position_levels};

pub struct OrganizationPositionLevelSeeder;

impl Seeder for OrganizationPositionLevelSeeder {
    fn class_name(&self) -> &'static str {
        "OrganizationPositionLevelSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seed organization position levels data")
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("🌱 Seeding organization position levels...");
        let mut conn = pool.get()?;

        // Get all top-level organizations (ones without parent)
        let organizations: Vec<(DieselUlid, Option<String>)> = organizations::table
            .filter(organizations::parent_id.is_null())
            .select((organizations::id, organizations::code))
            .load(&mut conn)?;

        if organizations.is_empty() {
            return Err(anyhow::anyhow!("No organizations found. Please seed organizations first."));
        }

        println!("   Creating position levels for {} organizations", organizations.len());

        // Define position levels for the organization hierarchy
        let position_levels = vec![
            ("C-LEVEL", "C-Level Executive", Some("Chief executive positions"), 1),
            ("SVP", "Senior Vice President", Some("Senior vice president level"), 2),
            ("VP", "Vice President", Some("Vice president level"), 3),
            ("DIRECTOR", "Director", Some("Director level management"), 4),
            ("SENIOR_MGR", "Senior Manager", Some("Senior management level"), 5),
            ("MANAGER", "Manager", Some("Middle management level"), 6),
            ("ASSISTANT_MGR", "Assistant Manager", Some("Assistant management level"), 7),
            ("TEAM_LEAD", "Team Lead", Some("Team leadership level"), 8),
            ("SENIOR_PRINCIPAL", "Senior Principal", Some("Senior principal level"), 9),
            ("PRINCIPAL", "Principal", Some("Principal level"), 10),
            ("SENIOR_STAFF", "Senior Staff", Some("Senior staff level"), 11),
            ("STAFF", "Staff", Some("Staff level"), 12),
            ("SENIOR", "Senior", Some("Senior level"), 13),
            ("MID_LEVEL", "Mid Level", Some("Mid-level positions"), 14),
            ("JUNIOR", "Junior", Some("Junior level positions"), 15),
            ("INTERN", "Intern", Some("Internship level"), 16),
            ("CONTRACTOR", "Contractor", Some("Contract positions"), 17),
            ("CONSULTANT", "Consultant", Some("Consulting positions"), 18),
            ("SPECIALIST", "Specialist", Some("Subject matter expert level"), 19),
            ("ANALYST", "Analyst", Some("Analysis and research level"), 20),
            ("ASSOCIATE", "Associate", Some("Associate level positions"), 21),
            ("COORDINATOR", "Coordinator", Some("Coordination level"), 22),
            ("SUPERVISOR", "Supervisor", Some("Supervisory level"), 23),
            ("LEAD", "Lead", Some("Lead level positions"), 24),
            ("EXPERT", "Expert", Some("Expert level positions"), 25),
        ];

        let mut total_created = 0;

        // Create position levels for each organization
        for (org_id, org_code) in organizations {
            for (code, name, description, level) in &position_levels {
                let position_level = CreateOrganizationPositionLevel {
                    organization_id: org_id,
                    code: code.to_string(),
                    name: name.to_string(),
                    description: description.map(|d| d.to_string()),
                    level: *level,
                };

                let new_position_level = OrganizationPositionLevel::new(position_level, None);
                diesel::insert_into(organization_position_levels::table)
                    .values(&new_position_level)
                    .on_conflict_do_nothing()
                    .execute(&mut conn)?;
                total_created += 1;
            }
            if let Some(code) = org_code {
                println!("   ✓ Created {} position levels for {}", position_levels.len(), code);
            }
        }

        println!("✅ {} Organization Position Levels seeded successfully!", total_created);
        Ok(())
    }
}
