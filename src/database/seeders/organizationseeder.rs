use crate::database::seeder::Seeder;
use sqlx::PgPool;
use anyhow::Result;
use ulid::Ulid;

pub struct OrganizationSeeder;

impl Seeder for OrganizationSeeder {
    fn class_name(&self) -> &'static str {
        "OrganizationSeeder"
    }

    async fn run(&self, pool: &PgPool) -> Result<()> {
        println!("Seeding organizations...");

        // Create holding company
        let holding_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO organizations (id, name, type, parent_id, code, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(holding_id.to_string())
        .bind("ABC Holdings Corp")
        .bind("company")
        .bind(None::<String>)
        .bind(Some("ABC-HOLD"))
        .bind(Some("Main holding company for all subsidiaries"))
        .bind(true)
        .execute(pool)
        .await?;

        // Create technology subsidiary
        let subsidiary_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO organizations (id, name, type, parent_id, code, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(subsidiary_id.to_string())
        .bind("ABC Technology Solutions")
        .bind("company")
        .bind(Some(holding_id.to_string()))
        .bind(Some("ABC-TECH"))
        .bind(Some("Technology and software development subsidiary"))
        .bind(true)
        .execute(pool)
        .await?;

        // Create development division
        let division_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO organizations (id, name, type, parent_id, code, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(division_id.to_string())
        .bind("Software Development Division")
        .bind("division")
        .bind(Some(subsidiary_id.to_string()))
        .bind(Some("TECH-DEV"))
        .bind(Some("Software development and engineering division"))
        .bind(true)
        .execute(pool)
        .await?;

        // Create backend department
        let department_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO organizations (id, name, type, parent_id, code, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(department_id.to_string())
        .bind("Backend Development Department")
        .bind("department")
        .bind(Some(division_id.to_string()))
        .bind(Some("DEV-BACK"))
        .bind(Some("Backend systems and API development"))
        .bind(true)
        .execute(pool)
        .await?;

        // Create API branch
        let branch_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO organizations (id, name, type, parent_id, code, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(branch_id.to_string())
        .bind("API Development Branch")
        .bind("branch")
        .bind(Some(department_id.to_string()))
        .bind(Some("BACK-API"))
        .bind(Some("RESTful API development"))
        .bind(true)
        .execute(pool)
        .await?;

        // Create core API sub-branch
        let subbranch_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO organizations (id, name, type, parent_id, code, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(subbranch_id.to_string())
        .bind("Core API Sub-branch")
        .bind("subbranch")
        .bind(Some(branch_id.to_string()))
        .bind(Some("API-CORE"))
        .bind(Some("Core business logic APIs"))
        .bind(true)
        .execute(pool)
        .await?;

        // Create user management section
        let section_id = Ulid::new();
        sqlx::query(
            r#"
            INSERT INTO organizations (id, name, type, parent_id, code, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(section_id.to_string())
        .bind("User Management Section")
        .bind("section")
        .bind(Some(subbranch_id.to_string()))
        .bind(Some("CORE-USER"))
        .bind(Some("User management and profile APIs"))
        .bind(true)
        .execute(pool)
        .await?;

        println!("Organizations seeded successfully!");
        Ok(())
    }
}