use crate::database::DbPool;
use anyhow::Result;
use crate::database::seeder::Seeder;
use crate::app::models::{DieselUlid, DecimalWrapper, organization_position::{NewOrganizationPosition, CreateOrganizationPosition}};
use diesel::prelude::*;
use crate::schema::{organizations, organization_position_levels, organization_positions};
use serde_json::json;

pub struct OrganizationPositionSeeder;

impl Seeder for OrganizationPositionSeeder {
    fn class_name(&self) -> &'static str {
        "OrganizationPositionSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seed organization positions data")
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("🌱 Seeding organization positions...");
        let mut conn = pool.get()?;

        // Get all top-level organizations
        let organizations: Vec<(DieselUlid, Option<String>)> = organizations::table
            .filter(organizations::parent_id.is_null())
            .select((organizations::id, organizations::code))
            .load(&mut conn)?;

        if organizations.is_empty() {
            return Err(anyhow::anyhow!("No organizations found. Please seed organizations first."));
        }

        println!("   Creating positions for {} organizations", organizations.len());

        // Define specific positions for each level
        let positions = vec![
            // C-Level positions
            ("CEO", "Chief Executive Officer", "C-LEVEL", 250000, 500000, 1,
             json!(["MBA or equivalent", "15+ years executive experience"]),
             json!(["Strategic leadership", "Board reporting", "Company vision"])),

            ("CTO", "Chief Technology Officer", "C-LEVEL", 220000, 450000, 1,
             json!(["Advanced technical degree", "15+ years tech leadership"]),
             json!(["Technology strategy", "Engineering leadership", "Innovation"])),

            // VP Level positions
            ("VP_ENG", "VP of Engineering", "VP", 180000, 300000, 1,
             json!(["Engineering degree", "10+ years leadership"]),
             json!(["Engineering strategy", "Team management", "Technical oversight"])),

            ("VP_PRODUCT", "VP of Product", "VP", 170000, 290000, 1,
             json!(["Product management experience", "8+ years leadership"]),
             json!(["Product strategy", "Roadmap planning", "Market analysis"])),

            // Director positions
            ("DIR_BACKEND", "Director of Backend Engineering", "DIRECTOR", 150000, 220000, 1,
             json!(["Backend expertise", "8+ years experience"]),
             json!(["Backend architecture", "Team leadership", "System design"])),

            ("DIR_FRONTEND", "Director of Frontend Engineering", "DIRECTOR", 145000, 210000, 1,
             json!(["Frontend expertise", "8+ years experience"]),
             json!(["Frontend architecture", "UI/UX collaboration", "Team management"])),

            // Manager positions
            ("MGR_BACKEND", "Backend Engineering Manager", "MANAGER", 120000, 180000, 3,
             json!(["5+ years backend development", "Management experience"]),
             json!(["Team leadership", "Code reviews", "Sprint planning"])),

            ("MGR_FRONTEND", "Frontend Engineering Manager", "MANAGER", 115000, 175000, 3,
             json!(["5+ years frontend development", "Management experience"]),
             json!(["Team leadership", "UI/UX coordination", "Performance optimization"])),

            ("MGR_MOBILE", "Mobile Engineering Manager", "MANAGER", 125000, 185000, 2,
             json!(["5+ years mobile development", "iOS/Android expertise"]),
             json!(["Mobile strategy", "Team leadership", "App store optimization"])),

            // Team Lead positions
            ("LEAD_API", "API Team Lead", "TEAM_LEAD", 95000, 140000, 2,
             json!(["4+ years API development", "Leadership skills"]),
             json!(["API design", "Team coordination", "Code quality"])),

            ("LEAD_DEVOPS", "DevOps Team Lead", "TEAM_LEAD", 100000, 145000, 2,
             json!(["4+ years DevOps", "Cloud platforms", "Automation"]),
             json!(["Infrastructure", "CI/CD", "Team leadership"])),

            // Senior positions
            ("SR_BACKEND_DEV", "Senior Backend Developer", "SENIOR", 90000, 130000, 5,
             json!(["4+ years backend development", "System design knowledge"]),
             json!(["Backend development", "Code reviews", "Mentoring"])),

            ("SR_FRONTEND_DEV", "Senior Frontend Developer", "SENIOR", 85000, 125000, 5,
             json!(["4+ years frontend development", "Modern frameworks"]),
             json!(["Frontend development", "UI implementation", "Code reviews"])),

            ("SR_MOBILE_DEV", "Senior Mobile Developer", "SENIOR", 88000, 128000, 3,
             json!(["4+ years mobile development", "Native/Cross-platform"]),
             json!(["Mobile development", "App optimization", "Code reviews"])),

            ("SR_DEVOPS_ENG", "Senior DevOps Engineer", "SENIOR", 92000, 135000, 4,
             json!(["4+ years DevOps", "Cloud expertise", "Automation"]),
             json!(["Infrastructure management", "Deployment automation", "Monitoring"])),

            // Mid-level positions
            ("BACKEND_DEV", "Backend Developer", "MID_LEVEL", 70000, 95000, 8,
             json!(["2+ years backend development", "API development"]),
             json!(["Backend development", "Database design", "Testing"])),

            ("FRONTEND_DEV", "Frontend Developer", "MID_LEVEL", 65000, 90000, 8,
             json!(["2+ years frontend development", "JavaScript/TypeScript"]),
             json!(["Frontend development", "Component design", "Testing"])),

            ("MOBILE_DEV", "Mobile Developer", "MID_LEVEL", 68000, 93000, 5,
             json!(["2+ years mobile development", "Native or React Native"]),
             json!(["Mobile app development", "UI implementation", "Testing"])),

            ("DEVOPS_ENG", "DevOps Engineer", "MID_LEVEL", 72000, 98000, 6,
             json!(["2+ years DevOps", "Linux/Docker", "CI/CD"]),
             json!(["System administration", "Deployment", "Monitoring"])),

            // Junior positions
            ("JR_BACKEND_DEV", "Junior Backend Developer", "JUNIOR", 50000, 70000, 10,
             json!(["CS degree or bootcamp", "Basic programming skills"]),
             json!(["Learning backend development", "Code maintenance", "Testing"])),

            ("JR_FRONTEND_DEV", "Junior Frontend Developer", "JUNIOR", 48000, 68000, 10,
             json!(["CS degree or bootcamp", "HTML/CSS/JavaScript"]),
             json!(["Learning frontend development", "UI implementation", "Bug fixes"])),

            ("JR_MOBILE_DEV", "Junior Mobile Developer", "JUNIOR", 52000, 72000, 6,
             json!(["CS degree or bootcamp", "Mobile development basics"]),
             json!(["Learning mobile development", "Feature implementation", "Testing"])),

            // Support roles
            ("QA_ENG", "QA Engineer", "STAFF", 60000, 85000, 8,
             json!(["QA experience", "Testing frameworks", "Attention to detail"]),
             json!(["Test planning", "Bug reporting", "Quality assurance"])),

            ("SYS_ADMIN", "System Administrator", "STAFF", 65000, 90000, 4,
             json!(["System administration", "Linux/Windows", "Network knowledge"]),
             json!(["Server management", "Security", "Backup management"])),

            ("BUSINESS_ANALYST", "Business Analyst", "ANALYST", 70000, 95000, 6,
             json!(["Business analysis", "Requirements gathering", "Documentation"]),
             json!(["Requirements analysis", "Process improvement", "Stakeholder communication"])),
        ];

        let mut total_created = 0;

        // Create positions for each organization
        for (org_id, org_code) in organizations {
            // Get position levels for this organization
            let position_levels: Vec<(DieselUlid, String, i32)> = organization_position_levels::table
                .filter(organization_position_levels::organization_id.eq(org_id))
                .select((organization_position_levels::id, organization_position_levels::code, organization_position_levels::level))
                .load(&mut conn)?;

            for (code, name, level_code, min_sal, max_sal, max_incumbents, qualifications, responsibilities) in &positions {
                // Find the matching position level
                if let Some((level_id, _, _)) = position_levels.iter().find(|(_, lc, _)| lc == level_code) {
                    let position = CreateOrganizationPosition {
                        organization_id: org_id,
                        organization_position_level_id: *level_id,
                        code: code.to_string(),
                        name: name.to_string(),
                        description: Some(format!("Position: {}", name)),
                        min_salary: Some(DecimalWrapper::from(*min_sal)),
                        max_salary: Some(DecimalWrapper::from(*max_sal)),
                        max_incumbents: Some(*max_incumbents),
                        qualifications: Some(qualifications.clone()),
                        responsibilities: Some(responsibilities.clone()),
                    };

                    let new_position = NewOrganizationPosition::new(position, None);
                    diesel::insert_into(organization_positions::table)
                        .values(&new_position)
                        .on_conflict_do_nothing()
                        .execute(&mut conn)?;
                    total_created += 1;
                }
            }

            if let Some(code) = org_code {
                println!("   ✓ Created {} positions for {}", positions.len(), code);
            }
        }

        println!("✅ {} Organization Positions seeded successfully!", total_created);
        Ok(())
    }
}
