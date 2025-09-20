use anyhow::Result;
use sqlx::{PgPool, Row};
use ulid::Ulid;
use chrono::Utc;
use serde_json::json;
use crate::database::seeder::Seeder;

pub struct AbacSeeder;

impl Seeder for AbacSeeder {
    fn class_name(&self) -> &'static str {
        "AbacSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seeds attributes and policies for ABAC (Attribute-Based Access Control)")
    }

    async fn run(&self, pool: &PgPool) -> Result<()> {
        println!("🌱 Seeding ABAC attributes and policies...");

        let now = Utc::now().naive_utc();

        // Create sample policies
        let admin_policy_id = Ulid::new().to_string();
        sqlx::query(
            r#"
            INSERT INTO policies (id, name, description, effect, target, condition, priority, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (name) DO NOTHING
            "#
        )
        .bind(admin_policy_id)
        .bind("admin_full_access")
        .bind("Admins have full access to all resources")
        .bind("permit")
        .bind("subject.role == 'admin'")
        .bind(None as Option<String>)
        .bind(100)
        .bind(true)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        let time_based_policy_id = Ulid::new().to_string();
        sqlx::query(
            r#"
            INSERT INTO policies (id, name, description, effect, target, condition, priority, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (name) DO NOTHING
            "#
        )
        .bind(time_based_policy_id)
        .bind("business_hours_only")
        .bind("Allow access only during business hours")
        .bind("permit")
        .bind("resource.type == 'sensitive_data'")
        .bind("environment.time >= '09:00' AND environment.time <= '17:00'")
        .bind(50)
        .bind(true)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        let department_policy_id = Ulid::new().to_string();
        sqlx::query(
            r#"
            INSERT INTO policies (id, name, description, effect, target, condition, priority, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (name) DO NOTHING
            "#
        )
        .bind(department_policy_id)
        .bind("department_data_access")
        .bind("Users can only access data from their department")
        .bind("permit")
        .bind("resource.type == 'department_data'")
        .bind("subject.department == resource.department")
        .bind(75)
        .bind(true)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // Create sample attributes for users
        if let Ok(admin_user) = sqlx::query(
            "SELECT id FROM users WHERE email = 'admin@example.com'"
        )
        .fetch_one(pool)
        .await
        {
            let admin_user_id: String = admin_user.get("id");

            // Admin role attribute
            let attr_id = Ulid::new().to_string();
            sqlx::query(
                r#"
                INSERT INTO attributes (id, name, attribute_type, value, subject_type, subject_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (name, subject_type, subject_id) DO NOTHING
                "#
            )
            .bind(attr_id)
            .bind("role")
            .bind("string")
            .bind(json!("admin"))
            .bind("user")
            .bind(&admin_user_id)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;

            // Department attribute
            let dept_attr_id = Ulid::new().to_string();
            sqlx::query(
                r#"
                INSERT INTO attributes (id, name, attribute_type, value, subject_type, subject_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (name, subject_type, subject_id) DO NOTHING
                "#
            )
            .bind(dept_attr_id)
            .bind("department")
            .bind("string")
            .bind(json!("IT"))
            .bind("user")
            .bind(&admin_user_id)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;

            // Clearance level attribute
            let clearance_attr_id = Ulid::new().to_string();
            sqlx::query(
                r#"
                INSERT INTO attributes (id, name, attribute_type, value, subject_type, subject_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (name, subject_type, subject_id) DO NOTHING
                "#
            )
            .bind(clearance_attr_id)
            .bind("clearance_level")
            .bind("number")
            .bind(json!(5))
            .bind("user")
            .bind(&admin_user_id)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;
        }

        if let Ok(regular_user) = sqlx::query(
            "SELECT id FROM users WHERE email = 'user@example.com'"
        )
        .fetch_one(pool)
        .await
        {
            let regular_user_id: String = regular_user.get("id");

            // User role attribute
            let attr_id = Ulid::new().to_string();
            sqlx::query(
                r#"
                INSERT INTO attributes (id, name, attribute_type, value, subject_type, subject_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (name, subject_type, subject_id) DO NOTHING
                "#
            )
            .bind(attr_id)
            .bind("role")
            .bind("string")
            .bind(json!("user"))
            .bind("user")
            .bind(&regular_user_id)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;

            // Department attribute
            let dept_attr_id = Ulid::new().to_string();
            sqlx::query(
                r#"
                INSERT INTO attributes (id, name, attribute_type, value, subject_type, subject_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (name, subject_type, subject_id) DO NOTHING
                "#
            )
            .bind(dept_attr_id)
            .bind("department")
            .bind("string")
            .bind(json!("Sales"))
            .bind("user")
            .bind(&regular_user_id)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;

            // Clearance level attribute
            let clearance_attr_id = Ulid::new().to_string();
            sqlx::query(
                r#"
                INSERT INTO attributes (id, name, attribute_type, value, subject_type, subject_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (name, subject_type, subject_id) DO NOTHING
                "#
            )
            .bind(clearance_attr_id)
            .bind("clearance_level")
            .bind("number")
            .bind(json!(2))
            .bind("user")
            .bind(&regular_user_id)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;
        }

        // Create sample resource attributes
        let resource_attr_id = Ulid::new().to_string();
        sqlx::query(
            r#"
            INSERT INTO attributes (id, name, attribute_type, value, subject_type, resource_type, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (name, subject_type, resource_type) DO NOTHING
            "#
        )
        .bind(resource_attr_id)
        .bind("sensitivity_level")
        .bind("number")
        .bind(json!(3))
        .bind("system")
        .bind("document")
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        let dept_resource_attr_id = Ulid::new().to_string();
        sqlx::query(
            r#"
            INSERT INTO attributes (id, name, attribute_type, value, subject_type, resource_type, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (name, subject_type, resource_type) DO NOTHING
            "#
        )
        .bind(dept_resource_attr_id)
        .bind("department")
        .bind("string")
        .bind(json!("IT"))
        .bind("system")
        .bind("document")
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // Create environment attributes
        let env_attr_id = Ulid::new().to_string();
        sqlx::query(
            r#"
            INSERT INTO attributes (id, name, attribute_type, value, subject_type, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (name, subject_type) DO NOTHING
            "#
        )
        .bind(env_attr_id)
        .bind("business_hours")
        .bind("object")
        .bind(json!({"start": "09:00", "end": "17:00", "timezone": "UTC"}))
        .bind("environment")
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        let location_attr_id = Ulid::new().to_string();
        sqlx::query(
            r#"
            INSERT INTO attributes (id, name, attribute_type, value, subject_type, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (name, subject_type) DO NOTHING
            "#
        )
        .bind(location_attr_id)
        .bind("allowed_locations")
        .bind("array")
        .bind(json!(["office", "home", "cafe"]))
        .bind("environment")
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        println!("✅ ABAC attributes and policies seeded successfully!");
        Ok(())
    }
}