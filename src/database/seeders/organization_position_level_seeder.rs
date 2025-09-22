use crate::database::seeder::Seeder;
use crate::database::DbPool;
use anyhow::Result;
use diesel::prelude::*;
use chrono::Utc;
use crate::app::models::{DieselUlid, DecimalWrapper};
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

        // Use a default organization ID for seeding - in real usage this would come from context
        let default_org_id = DieselUlid::new();

        // Create organization position levels
        let junior_level_id = DieselUlid::new();
        let new_junior_level = NewOrganizationPositionLevel {
            id: junior_level_id,
            organization_id: default_org_id,
            name: "Junior Level".to_string(),
            code: "JR".to_string(),
            level: 1,
            description: Some("Entry level positions".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        };

        diesel::insert_into(organization_position_levels::table)
            .values(&new_junior_level)
            .on_conflict(organization_position_levels::id)
            .do_nothing()
            .execute(&mut conn)?;

        let mid_level_id = DieselUlid::new();
        let new_mid_level = NewOrganizationPositionLevel {
            id: mid_level_id,
            organization_id: default_org_id,
            name: "Mid Level".to_string(),
            code: "MID".to_string(),
            level: 2,
            description: Some("Mid-level positions with experience".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        };

        diesel::insert_into(organization_position_levels::table)
            .values(&new_mid_level)
            .on_conflict(organization_position_levels::id)
            .do_nothing()
            .execute(&mut conn)?;

        let senior_level_id = DieselUlid::new();
        let new_senior_level = NewOrganizationPositionLevel {
            id: senior_level_id,
            organization_id: default_org_id,
            name: "Senior Level".to_string(),
            code: "SR".to_string(),
            level: 3,
            description: Some("Senior positions with leadership responsibilities".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        };

        diesel::insert_into(organization_position_levels::table)
            .values(&new_senior_level)
            .on_conflict(organization_position_levels::id)
            .do_nothing()
            .execute(&mut conn)?;

        let lead_level_id = DieselUlid::new();
        let new_lead_level = NewOrganizationPositionLevel {
            id: lead_level_id,
            organization_id: default_org_id,
            name: "Lead Level".to_string(),
            code: "LEAD".to_string(),
            level: 4,
            description: Some("Team lead and management positions".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        };

        diesel::insert_into(organization_position_levels::table)
            .values(&new_lead_level)
            .on_conflict(organization_position_levels::id)
            .do_nothing()
            .execute(&mut conn)?;

        let exec_level_id = DieselUlid::new();
        let new_exec_level = NewOrganizationPositionLevel {
            id: exec_level_id,
            organization_id: default_org_id,
            name: "Executive Level".to_string(),
            code: "EXEC".to_string(),
            level: 5,
            description: Some("Executive and C-level positions".to_string()),
            is_active: true,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        };

        diesel::insert_into(organization_position_levels::table)
            .values(&new_exec_level)
            .on_conflict(organization_position_levels::id)
            .do_nothing()
            .execute(&mut conn)?;

        // Create organization positions
        // Junior positions
        let junior_dev_id = DieselUlid::new();
        let new_junior_dev = NewOrganizationPosition {
            id: junior_dev_id,
            organization_id: default_org_id,
            organization_position_level_id: junior_level_id,
            code: "JR-DEV".to_string(),
            name: "Junior Software Developer".to_string(),
            description: Some("Entry-level software development position".to_string()),
            is_active: true,
            min_salary: DecimalWrapper::from(50000),
            max_salary: DecimalWrapper::from(70000),
            max_incumbents: 10,
            qualifications: serde_json::json!(["Bachelor's degree in Computer Science", "Basic programming skills"]),
            responsibilities: serde_json::json!(["Write clean, maintainable code", "Learn from senior developers", "Participate in code reviews"]),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_junior_dev)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        // Mid-level positions
        let software_dev_id = DieselUlid::new();
        let new_software_dev = NewOrganizationPosition {
            id: software_dev_id,
            organization_id: default_org_id,
            organization_position_level_id: mid_level_id,
            code: "DEV".to_string(),
            name: "Software Developer".to_string(),
            description: Some("Mid-level software development position".to_string()),
            is_active: true,
            min_salary: DecimalWrapper::from(70000),
            max_salary: DecimalWrapper::from(90000),
            max_incumbents: 8,
            qualifications: serde_json::json!(["3+ years programming experience", "Strong problem-solving skills"]),
            responsibilities: serde_json::json!(["Develop features independently", "Mentor junior developers", "Participate in technical decisions"]),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_software_dev)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        let backend_dev_id = DieselUlid::new();
        let new_backend_dev = NewOrganizationPosition {
            id: backend_dev_id,
            organization_id: default_org_id,
            organization_position_level_id: mid_level_id,
            code: "BACK-DEV".to_string(),
            name: "Backend Developer".to_string(),
            description: Some("Backend systems development specialist".to_string()),
            is_active: true,
            min_salary: DecimalWrapper::from(75000),
            max_salary: DecimalWrapper::from(95000),
            max_incumbents: 6,
            qualifications: serde_json::json!(["Strong backend development experience", "Database design skills", "API development experience"]),
            responsibilities: serde_json::json!(["Design and implement backend services", "Optimize database queries", "Ensure system scalability"]),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_backend_dev)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        // Senior positions
        let senior_dev_id = DieselUlid::new();
        let new_senior_dev = NewOrganizationPosition {
            id: senior_dev_id,
            organization_id: default_org_id,
            organization_position_level_id: senior_level_id,
            code: "SR-DEV".to_string(),
            name: "Senior Software Developer".to_string(),
            description: Some("Senior software development position with mentoring responsibilities".to_string()),
            is_active: true,
            min_salary: DecimalWrapper::from(90000),
            max_salary: DecimalWrapper::from(120000),
            max_incumbents: 4,
            qualifications: serde_json::json!(["5+ years software development experience", "Leadership skills", "Technical mentoring experience"]),
            responsibilities: serde_json::json!(["Lead technical projects", "Mentor team members", "Make architectural decisions", "Code review and quality assurance"]),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_senior_dev)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        let architect_id = DieselUlid::new();
        let new_architect = NewOrganizationPosition {
            id: architect_id,
            organization_id: default_org_id,
            organization_position_level_id: senior_level_id,
            code: "ARCHITECT".to_string(),
            name: "Software Architect".to_string(),
            description: Some("Software architecture design and technical leadership".to_string()),
            is_active: true,
            min_salary: DecimalWrapper::from(110000),
            max_salary: DecimalWrapper::from(140000),
            max_incumbents: 2,
            qualifications: serde_json::json!(["7+ years development experience", "System architecture experience", "Technical leadership skills"]),
            responsibilities: serde_json::json!(["Design system architecture", "Technical strategy and planning", "Cross-team collaboration", "Technology evaluation"]),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_architect)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        // Lead positions
        let tech_lead_id = DieselUlid::new();
        let new_tech_lead = NewOrganizationPosition {
            id: tech_lead_id,
            organization_id: default_org_id,
            organization_position_level_id: lead_level_id,
            code: "TECH-LEAD".to_string(),
            name: "Technical Lead".to_string(),
            description: Some("Technical team leadership and project oversight".to_string()),
            is_active: true,
            min_salary: DecimalWrapper::from(120000),
            max_salary: DecimalWrapper::from(150000),
            max_incumbents: 3,
            qualifications: serde_json::json!(["8+ years development experience", "Team leadership experience", "Project management skills"]),
            responsibilities: serde_json::json!(["Lead development teams", "Project planning and execution", "Technical decision making", "Performance management"]),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_tech_lead)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        let engineering_mgr_id = DieselUlid::new();
        let new_engineering_mgr = NewOrganizationPosition {
            id: engineering_mgr_id,
            organization_id: default_org_id,
            organization_position_level_id: lead_level_id,
            code: "ENG-MGR".to_string(),
            name: "Engineering Manager".to_string(),
            description: Some("Engineering team management and strategic planning".to_string()),
            is_active: true,
            min_salary: DecimalWrapper::from(130000),
            max_salary: DecimalWrapper::from(160000),
            max_incumbents: 2,
            qualifications: serde_json::json!(["5+ years management experience", "Strong technical background", "Strategic planning skills"]),
            responsibilities: serde_json::json!(["Manage engineering teams", "Strategic planning", "Budget management", "Hiring and development"]),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_engineering_mgr)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        // Executive positions
        let cto_id = DieselUlid::new();
        let new_cto = NewOrganizationPosition {
            id: cto_id,
            organization_id: default_org_id,
            organization_position_level_id: exec_level_id,
            code: "CTO".to_string(),
            name: "Chief Technology Officer".to_string(),
            description: Some("Chief Technology Officer - technology strategy and vision".to_string()),
            is_active: true,
            min_salary: DecimalWrapper::from(200000),
            max_salary: DecimalWrapper::from(300000),
            max_incumbents: 1,
            qualifications: serde_json::json!(["10+ years executive experience", "Technology vision and strategy", "Board-level communication"]),
            responsibilities: serde_json::json!(["Technology strategy and vision", "Executive leadership", "Board reporting", "Company-wide technical decisions"]),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        };

        diesel::insert_into(organization_positions::table)
            .values(&new_cto)
            .on_conflict(organization_positions::id)
            .do_nothing()
            .execute(&mut conn)?;

        let vp_eng_id = DieselUlid::new();
        let new_vp_eng = NewOrganizationPosition {
            id: vp_eng_id,
            organization_id: default_org_id,
            organization_position_level_id: exec_level_id,
            code: "VP-ENG".to_string(),
            name: "VP of Engineering".to_string(),
            description: Some("Vice President of Engineering - engineering organization leadership".to_string()),
            is_active: true,
            min_salary: DecimalWrapper::from(180000),
            max_salary: DecimalWrapper::from(250000),
            max_incumbents: 1,
            qualifications: serde_json::json!(["8+ years executive experience", "Large team management", "Strategic leadership"]),
            responsibilities: serde_json::json!(["Engineering organization leadership", "Strategic initiatives", "Cross-functional collaboration", "Talent development"]),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
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