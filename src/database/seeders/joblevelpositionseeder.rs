use crate::database::seeder::Seeder;
use crate::database::DbPool;
use anyhow::Result;
use ulid::Ulid;
use diesel::prelude::*;
use chrono::Utc;

pub struct JobLevelPositionSeeder;

impl Seeder for JobLevelPositionSeeder {
    fn class_name(&self) -> &'static str {
        "JobLevelPositionSeeder"
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("Seeding job levels and positions...");
        let mut conn = pool.get()?;
        let now = Utc::now();

        // Create job levels
        let junior_level_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO job_levels (id, name, code, level, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(junior_level_id.to_string())
        .bind::<diesel::sql_types::Text, _>("Junior Level")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("JR"))
        .bind::<diesel::sql_types::Integer, _>(1)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Entry level positions"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        let mid_level_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO job_levels (id, name, code, level, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(mid_level_id.to_string())
        .bind::<diesel::sql_types::Text, _>("Mid Level")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("MID"))
        .bind::<diesel::sql_types::Integer, _>(2)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Mid-level positions with experience"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        let senior_level_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO job_levels (id, name, code, level, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(senior_level_id.to_string())
        .bind::<diesel::sql_types::Text, _>("Senior Level")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("SR"))
        .bind::<diesel::sql_types::Integer, _>(3)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Senior positions with leadership responsibilities"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        let lead_level_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO job_levels (id, name, code, level, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(lead_level_id.to_string())
        .bind::<diesel::sql_types::Text, _>("Lead Level")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("LEAD"))
        .bind::<diesel::sql_types::Integer, _>(4)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Team lead and management positions"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        let exec_level_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO job_levels (id, name, code, level, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(exec_level_id.to_string())
        .bind::<diesel::sql_types::Text, _>("Executive Level")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("EXEC"))
        .bind::<diesel::sql_types::Integer, _>(5)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Executive and C-level positions"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        // Create job positions
        // Junior positions
        let junior_dev_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(junior_dev_id.to_string())
        .bind::<diesel::sql_types::Text, _>("Junior Software Developer")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("JR-DEV"))
        .bind::<diesel::sql_types::Text, _>(junior_level_id.to_string())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Entry-level software development position"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        // Mid-level positions
        let software_dev_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(software_dev_id.to_string())
        .bind::<diesel::sql_types::Text, _>("Software Developer")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("DEV"))
        .bind::<diesel::sql_types::Text, _>(mid_level_id.to_string())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Mid-level software development position"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        let backend_dev_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(backend_dev_id.to_string())
        .bind::<diesel::sql_types::Text, _>("Backend Developer")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("BACK-DEV"))
        .bind::<diesel::sql_types::Text, _>(mid_level_id.to_string())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Backend systems development specialist"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        // Senior positions
        let senior_dev_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(senior_dev_id.to_string())
        .bind::<diesel::sql_types::Text, _>("Senior Software Developer")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("SR-DEV"))
        .bind::<diesel::sql_types::Text, _>(senior_level_id.to_string())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Senior software development position with mentoring responsibilities"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        let architect_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(architect_id.to_string())
        .bind::<diesel::sql_types::Text, _>("Software Architect")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("ARCHITECT"))
        .bind::<diesel::sql_types::Text, _>(senior_level_id.to_string())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Software architecture design and technical leadership"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        // Lead positions
        let tech_lead_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(tech_lead_id.to_string())
        .bind::<diesel::sql_types::Text, _>("Technical Lead")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("TECH-LEAD"))
        .bind::<diesel::sql_types::Text, _>(lead_level_id.to_string())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Technical team leadership and project oversight"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        let engineering_mgr_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(engineering_mgr_id.to_string())
        .bind::<diesel::sql_types::Text, _>("Engineering Manager")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("ENG-MGR"))
        .bind::<diesel::sql_types::Text, _>(lead_level_id.to_string())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Engineering team management and strategic planning"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        // Executive positions
        let cto_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(cto_id.to_string())
        .bind::<diesel::sql_types::Text, _>("Chief Technology Officer")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("CTO"))
        .bind::<diesel::sql_types::Text, _>(exec_level_id.to_string())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Chief Technology Officer - technology strategy and vision"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        let vp_eng_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(vp_eng_id.to_string())
        .bind::<diesel::sql_types::Text, _>("VP of Engineering")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("VP-ENG"))
        .bind::<diesel::sql_types::Text, _>(exec_level_id.to_string())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Vice President of Engineering - engineering organization leadership"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        println!("Job levels and positions seeded successfully!");
        Ok(())
    }
}