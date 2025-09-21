use crate::database::seeder::Seeder;
use crate::database::DbPool;
use anyhow::Result;
use ulid::Ulid;
use diesel::prelude::*;
use chrono::Utc;

pub struct OrganizationSeeder;

impl Seeder for OrganizationSeeder {
    fn class_name(&self) -> &'static str {
        "OrganizationSeeder"
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("Seeding organizations...");
        let mut conn = pool.get()?;
        let now = Utc::now();

        // Create holding company
        let holding_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO organizations (id, name, type, parent_id, code, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(holding_id.to_string())
        .bind::<diesel::sql_types::Text, _>("ABC Holdings Corp")
        .bind::<diesel::sql_types::Text, _>("company")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(None::<String>)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("ABC-HOLD"))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Main holding company for all subsidiaries"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        // Create technology subsidiary
        let subsidiary_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO organizations (id, name, type, parent_id, code, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(subsidiary_id.to_string())
        .bind::<diesel::sql_types::Text, _>("ABC Technology Solutions")
        .bind::<diesel::sql_types::Text, _>("company")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(holding_id.to_string()))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("ABC-TECH"))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Technology and software development subsidiary"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        // Create development division
        let division_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO organizations (id, name, type, parent_id, code, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(division_id.to_string())
        .bind::<diesel::sql_types::Text, _>("Software Development Division")
        .bind::<diesel::sql_types::Text, _>("division")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(subsidiary_id.to_string()))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("TECH-DEV"))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Software development and engineering division"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        // Create backend department
        let department_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO organizations (id, name, type, parent_id, code, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(department_id.to_string())
        .bind::<diesel::sql_types::Text, _>("Backend Development Department")
        .bind::<diesel::sql_types::Text, _>("department")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(division_id.to_string()))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("DEV-BACK"))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Backend systems and API development"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        // Create API branch
        let branch_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO organizations (id, name, type, parent_id, code, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(branch_id.to_string())
        .bind::<diesel::sql_types::Text, _>("API Development Branch")
        .bind::<diesel::sql_types::Text, _>("branch")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(department_id.to_string()))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("BACK-API"))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("RESTful API development"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        // Create core API sub-branch
        let subbranch_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO organizations (id, name, type, parent_id, code, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(subbranch_id.to_string())
        .bind::<diesel::sql_types::Text, _>("Core API Sub-branch")
        .bind::<diesel::sql_types::Text, _>("subbranch")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(branch_id.to_string()))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("API-CORE"))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Core business logic APIs"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        // Create user management section
        let section_id = Ulid::new();
        diesel::sql_query(
            r#"
            INSERT INTO organizations (id, name, type, parent_id, code, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(section_id.to_string())
        .bind::<diesel::sql_types::Text, _>("User Management Section")
        .bind::<diesel::sql_types::Text, _>("section")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(subbranch_id.to_string()))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("CORE-USER"))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("User management and profile APIs"))
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        println!("Organizations seeded successfully!");
        Ok(())
    }
}