use crate::app::models::organization_type::NewOrganizationType;
use crate::app::models::DieselUlid;
use crate::database::seeder::Seeder;
use crate::database::DbPool;
use crate::schema::{organization_domains, organization_types, sys_users};
use anyhow::Result;
use diesel::prelude::*;

pub struct OrganizationTypeSeeder;

impl Seeder for OrganizationTypeSeeder {
    fn class_name(&self) -> &'static str {
        "OrganizationTypeSeeder"
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("🌱 Seeding organization types...");
        let mut conn = pool.get()?;

        // Get system user ID for audit tracking
        let system_user_id: String = sys_users::table
            .filter(sys_users::email.eq("system@seeder.internal"))
            .select(sys_users::id)
            .first(&mut conn)?;

        // Define organization types by domain with hierarchical levels
        let types_data = vec![
            // Government types (hierarchical levels 1-7)
            (
                "GOV",
                vec![
                    ("MIN", "Ministry", "Cabinet-level ministry", 1),
                    ("AGN", "Agency", "Government agency or autonomous body", 2),
                    ("DEPT", "Department", "Department within ministry/agency", 3),
                    ("DIR", "Directorate", "Directorate or main division", 4),
                    ("DIV", "Division", "Division or sub-directorate", 5),
                    ("SEC", "Section", "Section or unit", 6),
                    ("OFF", "Office", "Regional or field office", 7),
                ],
            ),
            // Private Sector types (hierarchical levels 1-8)
            (
                "PVT",
                vec![
                    ("HOLD", "Holding Company", "Parent holding company", 1),
                    ("CORP", "Corporation", "Large corporation or group", 2),
                    ("COMP", "Company", "Private company or subsidiary", 3),
                    ("BOD", "Business Unit", "Strategic business unit", 4),
                    ("DIV", "Division", "Business division", 5),
                    ("DEPT", "Department", "Department or functional unit", 6),
                    ("TEAM", "Team", "Team, squad, or small unit", 7),
                    ("BRAN", "Branch", "Branch office or location", 8),
                ],
            ),
            // NGO types (hierarchical levels 1-5)
            (
                "NGO",
                vec![
                    ("FOUN", "Foundation", "Non-profit foundation", 1),
                    ("ASSOC", "Association", "Association or society", 2),
                    ("PROG", "Program", "Program or initiative", 3),
                    ("PROJ", "Project", "Project or campaign", 4),
                    ("UNIT", "Unit", "Working unit or team", 5),
                ],
            ),
            // Education types (hierarchical levels 1-7)
            (
                "EDU",
                vec![
                    (
                        "UNIV",
                        "University",
                        "University or higher education institution",
                        1,
                    ),
                    ("INST", "Institute", "Specialized institute or academy", 2),
                    ("COLL", "College", "College or faculty", 3),
                    ("SCH", "School", "School or department", 4),
                    ("DEPT", "Department", "Academic department", 5),
                    ("LAB", "Laboratory", "Research laboratory or center", 6),
                    ("PROG", "Program", "Academic program or course", 7),
                ],
            ),
            // Healthcare types (hierarchical levels 1-5)
            (
                "HEA",
                vec![
                    ("HOSP", "Hospital", "General or specialized hospital", 1),
                    ("CLIN", "Clinic", "Clinic or health center", 2),
                    ("DEPT", "Department", "Medical department or service", 3),
                    ("UNIT", "Unit", "Medical unit or ward", 4),
                    ("TEAM", "Team", "Medical team or practice group", 5),
                ],
            ),
            // Military types (hierarchical levels 1-7)
            (
                "MIL",
                vec![
                    ("COMMAND", "Command", "Major command or force", 1),
                    ("CORPS", "Corps", "Corps or major formation", 2),
                    ("DIVISION", "Division", "Division", 3),
                    ("BRIGADE", "Brigade", "Brigade or regiment", 4),
                    ("BATTALION", "Battalion", "Battalion or squadron", 5),
                    ("COMPANY", "Company", "Company, battery, or troop", 6),
                    ("PLATOON", "Platoon", "Platoon or section", 7),
                ],
            ),
            // Religious types (hierarchical levels 1-5)
            (
                "REL",
                vec![
                    ("DIOC", "Diocese", "Diocese or archdiocese", 1),
                    ("REGION", "Region", "Regional organization", 2),
                    ("PARISH", "Parish", "Parish or congregation", 3),
                    ("COMM", "Community", "Community or ministry", 4),
                    ("GROUP", "Group", "Small group or cell", 5),
                ],
            ),
        ];

        for (domain_code, types) in types_data {
            // Get domain_id by code
            let domain_id: DieselUlid = organization_domains::table
                .filter(organization_domains::code.eq(domain_code))
                .select(organization_domains::id)
                .first(&mut conn)?;

            println!("   📁 Seeding types for domain: {}", domain_code);

            for (code, name, description, level) in types {
                let new_type = NewOrganizationType {
                    id: DieselUlid::new(),
                    domain_id,
                    code: Some(code.to_string()),
                    name: name.to_string(),
                    description: Some(description.to_string()),
                    level,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    deleted_at: None,
                    created_by_id: system_user_id.clone(),
                    updated_by_id: system_user_id.clone(),
                    deleted_by_id: None,
                };

                diesel::insert_into(organization_types::table)
                    .values(&new_type)
                    .on_conflict((organization_types::domain_id, organization_types::code))
                    .do_nothing()
                    .execute(&mut conn)?;

                println!(
                    "      ✓ Created type: {} ({}) - Level {}",
                    name, code, level
                );
            }
        }

        println!("✅ Organization types seeded successfully!");
        Ok(())
    }
}
