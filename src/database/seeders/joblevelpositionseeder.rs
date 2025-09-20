use crate::database::seeder::Seeder;
use sqlx::PgPool;
use anyhow::Result;
use ulid::Ulid;

pub struct JobLevelPositionSeeder;

impl Seeder for JobLevelPositionSeeder {
    fn class_name(&self) -> &'static str {
        "JobLevelPositionSeeder"
    }

    async fn run(&self, pool: &PgPool) -> Result<()> {
        println!("Seeding job levels and positions...");

        // Create job levels
        let junior_level_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO job_levels (id, name, code, level, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(junior_level_id.to_string())
        .bind("Junior Level")
        .bind(Some("JR"))
        .bind(1)
        .bind(Some("Entry level positions"))
        .bind(true)
        .execute(pool)
        .await?;

        let mid_level_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO job_levels (id, name, code, level, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(mid_level_id.to_string())
        .bind("Mid Level")
        .bind(Some("MID"))
        .bind(2)
        .bind(Some("Mid-level positions with experience"))
        .bind(true)
        .execute(pool)
        .await?;

        let senior_level_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO job_levels (id, name, code, level, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(senior_level_id.to_string())
        .bind("Senior Level")
        .bind(Some("SR"))
        .bind(3)
        .bind(Some("Senior positions with leadership responsibilities"))
        .bind(true)
        .execute(pool)
        .await?;

        let lead_level_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO job_levels (id, name, code, level, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(lead_level_id.to_string())
        .bind("Lead Level")
        .bind(Some("LEAD"))
        .bind(4)
        .bind(Some("Team lead and management positions"))
        .bind(true)
        .execute(pool)
        .await?;

        let exec_level_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO job_levels (id, name, code, level, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(exec_level_id.to_string())
        .bind("Executive Level")
        .bind(Some("EXEC"))
        .bind(5)
        .bind(Some("Executive and C-level positions"))
        .bind(true)
        .execute(pool)
        .await?;

        // Create job positions
        // Junior positions
        let junior_dev_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(junior_dev_id.to_string())
        .bind("Junior Software Developer")
        .bind(Some("JR-DEV"))
        .bind(junior_level_id.to_string())
        .bind(Some("Entry-level software development position"))
        .bind(true)
        .execute(pool)
        .await?;

        // Mid-level positions
        let software_dev_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(software_dev_id.to_string())
        .bind("Software Developer")
        .bind(Some("DEV"))
        .bind(mid_level_id.to_string())
        .bind(Some("Mid-level software development position"))
        .bind(true)
        .execute(pool)
        .await?;

        let backend_dev_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(backend_dev_id.to_string())
        .bind("Backend Developer")
        .bind(Some("BACK-DEV"))
        .bind(mid_level_id.to_string())
        .bind(Some("Backend systems development specialist"))
        .bind(true)
        .execute(pool)
        .await?;

        // Senior positions
        let senior_dev_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(senior_dev_id.to_string())
        .bind("Senior Software Developer")
        .bind(Some("SR-DEV"))
        .bind(senior_level_id.to_string())
        .bind(Some("Senior software development position with mentoring responsibilities"))
        .bind(true)
        .execute(pool)
        .await?;

        let architect_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(architect_id.to_string())
        .bind("Software Architect")
        .bind(Some("ARCHITECT"))
        .bind(senior_level_id.to_string())
        .bind(Some("Software architecture design and technical leadership"))
        .bind(true)
        .execute(pool)
        .await?;

        // Lead positions
        let tech_lead_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(tech_lead_id.to_string())
        .bind("Technical Lead")
        .bind(Some("TECH-LEAD"))
        .bind(lead_level_id.to_string())
        .bind(Some("Technical team leadership and project oversight"))
        .bind(true)
        .execute(pool)
        .await?;

        let engineering_mgr_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(engineering_mgr_id.to_string())
        .bind("Engineering Manager")
        .bind(Some("ENG-MGR"))
        .bind(lead_level_id.to_string())
        .bind(Some("Engineering team management and strategic planning"))
        .bind(true)
        .execute(pool)
        .await?;

        // Executive positions
        let cto_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(cto_id.to_string())
        .bind("Chief Technology Officer")
        .bind(Some("CTO"))
        .bind(exec_level_id.to_string())
        .bind(Some("Chief Technology Officer - technology strategy and vision"))
        .bind(true)
        .execute(pool)
        .await?;

        let vp_eng_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(vp_eng_id.to_string())
        .bind("VP of Engineering")
        .bind(Some("VP-ENG"))
        .bind(exec_level_id.to_string())
        .bind(Some("Vice President of Engineering - engineering organization leadership"))
        .bind(true)
        .execute(pool)
        .await?;

        println!("Job levels and positions seeded successfully!");
        Ok(())
    }
}