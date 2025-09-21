use crate::database::seeder::Seeder;
use crate::database::DbPool;
use anyhow::Result;
use ulid::Ulid;
use diesel::prelude::*;
use chrono::Utc;
use crate::app::models::organization_position_level::NewOrganizationPositionLevel;
use crate::app::models::organization_position::NewOrganizationPosition;
use crate::schema::{organization_position_levels, organization_positions};

pub struct OrganizationPositionLevelSeeder;

impl Seeder for OrganizationPositionLevelSeeder {
    fn class_name(&self) -> &'static str {
        "OrganizationPositionLevelSeeder"
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("Seeding organization position levels and positions...");
        let mut conn = pool.get()?;
        let now = Utc::now();

        // Create organization position levels
        let junior_level_id = Ulid::new();
        let new_junior_level = NewOrganizationPositionLevel {
            id: junior_level_id.to_string(),
            name: "Junior Level".to_string(),
            code: Some("JR".to_string()),
            level: 1,
            description: Some("Entry level positions".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(organization_position_levels::table)
            .values(&new_junior_level)
            .on_conflict(organization_position_levels::id)
            .do_nothing()
            .execute(&mut conn)?;

        let mid_level_id = Ulid::new();
        let new_mid_level = NewOrganizationPositionLevel {
            id: mid_level_id.to_string(),
            name: "Mid Level".to_string(),
            code: Some("MID".to_string()),
            level: 2,
            description: Some("Mid-level positions with experience".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(organization_position_levels::table)
            .values(&new_mid_level)
            .on_conflict(organization_position_levels::id)
            .do_nothing()
            .execute(&mut conn)?;

        let senior_level_id = Ulid::new();
        let new_senior_level = NewOrganizationPositionLevel {
            id: senior_level_id.to_string(),
            name: "Senior Level".to_string(),
            code: Some("SR".to_string()),
            level: 3,
            description: Some("Senior positions with leadership responsibilities".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(organization_position_levels::table)
            .values(&new_senior_level)
            .on_conflict(organization_position_levels::id)
            .do_nothing()
            .execute(&mut conn)?;

        let lead_level_id = Ulid::new();
        let new_lead_level = NewOrganizationPositionLevel {
            id: lead_level_id.to_string(),
            name: "Lead Level".to_string(),
            code: Some("LEAD".to_string()),
            level: 4,
            description: Some("Team lead and management positions".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(organization_position_levels::table)
            .values(&new_lead_level)
            .on_conflict(organization_position_levels::id)
            .do_nothing()
            .execute(&mut conn)?;

        let exec_level_id = Ulid::new();
        let new_exec_level = NewOrganizationPositionLevel {
            id: exec_level_id.to_string(),
            name: "Executive Level".to_string(),
            code: Some("EXEC".to_string()),
            level: 5,
            description: Some("Executive and C-level positions".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(organization_position_levels::table)
            .values(&new_exec_level)
            .on_conflict(organization_position_levels::id)
            .do_nothing()
            .execute(&mut conn)?;

        // Create organization positions
        // Junior positions
        let junior_dev_id = Ulid::new();
        let new_junior_dev = NewOrganizationPosition {
            id: junior_dev_id.to_string(),
            name: "Junior Software Developer".to_string(),
            code: Some("JR-DEV".to_string()),
            organization_position_level_id: junior_level_id.to_string(),
            description: Some("Entry-level software development position".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_junior_dev)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        // Mid-level positions
        let software_dev_id = Ulid::new();
        let new_software_dev = NewOrganizationPosition {
            id: software_dev_id.to_string(),
            name: "Software Developer".to_string(),
            code: Some("DEV".to_string()),
            organization_position_level_id: mid_level_id.to_string(),
            description: Some("Mid-level software development position".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_software_dev)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        let backend_dev_id = Ulid::new();
        let new_backend_dev = NewOrganizationPosition {
            id: backend_dev_id.to_string(),
            name: "Backend Developer".to_string(),
            code: Some("BACK-DEV".to_string()),
            organization_position_level_id: mid_level_id.to_string(),
            description: Some("Backend systems development specialist".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_backend_dev)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        // Senior positions
        let senior_dev_id = Ulid::new();
        let new_senior_dev = NewOrganizationPosition {
            id: senior_dev_id.to_string(),
            name: "Senior Software Developer".to_string(),
            code: Some("SR-DEV".to_string()),
            organization_position_level_id: senior_level_id.to_string(),
            description: Some("Senior software development position with mentoring responsibilities".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_senior_dev)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        let architect_id = Ulid::new();
        let new_architect = NewOrganizationPosition {
            id: architect_id.to_string(),
            name: "Software Architect".to_string(),
            code: Some("ARCHITECT".to_string()),
            organization_position_level_id: senior_level_id.to_string(),
            description: Some("Software architecture design and technical leadership".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_architect)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        // Lead positions
        let tech_lead_id = Ulid::new();
        let new_tech_lead = NewOrganizationPosition {
            id: tech_lead_id.to_string(),
            name: "Technical Lead".to_string(),
            code: Some("TECH-LEAD".to_string()),
            organization_position_level_id: lead_level_id.to_string(),
            description: Some("Technical team leadership and project oversight".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_tech_lead)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        let engineering_mgr_id = Ulid::new();
        let new_engineering_mgr = NewOrganizationPosition {
            id: engineering_mgr_id.to_string(),
            name: "Engineering Manager".to_string(),
            code: Some("ENG-MGR".to_string()),
            organization_position_level_id: lead_level_id.to_string(),
            description: Some("Engineering team management and strategic planning".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_engineering_mgr)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        // Executive positions
        let cto_id = Ulid::new();
        let new_cto = NewOrganizationPosition {
            id: cto_id.to_string(),
            name: "Chief Technology Officer".to_string(),
            code: Some("CTO".to_string()),
            organization_position_level_id: exec_level_id.to_string(),
            description: Some("Chief Technology Officer - technology strategy and vision".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_cto)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        let vp_eng_id = Ulid::new();
        let new_vp_eng = NewOrganizationPosition {
            id: vp_eng_id.to_string(),
            name: "VP of Engineering".to_string(),
            code: Some("VP-ENG".to_string()),
            organization_position_level_id: exec_level_id.to_string(),
            description: Some("Vice President of Engineering - engineering organization leadership".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_vp_eng)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        println!("Job levels and positions seeded successfully!");
        Ok(())
    }
}