use anyhow::Result;
use sqlx::{PgPool, Row};
use ulid::Ulid;
use chrono::Utc;
use crate::database::seeder::Seeder;

pub struct RolePermissionSeeder;

impl Seeder for RolePermissionSeeder {
    fn class_name(&self) -> &'static str {
        "RolePermissionSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seeds roles, permissions, and their relationships for RBAC")
    }

    async fn run(&self, pool: &PgPool) -> Result<()> {
        println!("🌱 Seeding roles and permissions...");

        let now = Utc::now().naive_utc();

        // Create permissions
        let permissions = vec![
            ("users.create", "web", Some("users"), "create"),
            ("users.read", "web", Some("users"), "read"),
            ("users.update", "web", Some("users"), "update"),
            ("users.delete", "web", Some("users"), "delete"),
            ("posts.create", "web", Some("posts"), "create"),
            ("posts.read", "web", Some("posts"), "read"),
            ("posts.update", "web", Some("posts"), "update"),
            ("posts.delete", "web", Some("posts"), "delete"),
            ("roles.create", "web", Some("roles"), "create"),
            ("roles.read", "web", Some("roles"), "read"),
            ("roles.update", "web", Some("roles"), "update"),
            ("roles.delete", "web", Some("roles"), "delete"),
            ("permissions.create", "web", Some("permissions"), "create"),
            ("permissions.read", "web", Some("permissions"), "read"),
            ("permissions.update", "web", Some("permissions"), "update"),
            ("permissions.delete", "web", Some("permissions"), "delete"),
        ];

        for (name, guard_name, resource, action) in permissions {
            let permission_id = Ulid::new().to_string();
            sqlx::query(
                r#"
                INSERT INTO permissions (id, name, guard_name, resource, action, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (name, guard_name) DO NOTHING
                "#
            )
            .bind(permission_id)
            .bind(name)
            .bind(guard_name)
            .bind(resource)
            .bind(action)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;
        }

        // Create roles
        let admin_role_id = Ulid::new().to_string();
        sqlx::query(
            r#"
            INSERT INTO roles (id, name, description, guard_name, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (name, guard_name) DO NOTHING
            "#
        )
        .bind(&admin_role_id)
        .bind("admin")
        .bind("Administrator with full access")
        .bind("web")
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        let user_role_id = Ulid::new().to_string();
        sqlx::query(
            r#"
            INSERT INTO roles (id, name, description, guard_name, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (name, guard_name) DO NOTHING
            "#
        )
        .bind(&user_role_id)
        .bind("user")
        .bind("Regular user with limited access")
        .bind("web")
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        let moderator_role_id = Ulid::new().to_string();
        sqlx::query(
            r#"
            INSERT INTO roles (id, name, description, guard_name, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (name, guard_name) DO NOTHING
            "#
        )
        .bind(&moderator_role_id)
        .bind("moderator")
        .bind("Moderator with content management access")
        .bind("web")
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // Assign all permissions to admin role
        let admin_permissions = sqlx::query(
            "SELECT id FROM permissions WHERE guard_name = 'web'"
        )
        .fetch_all(pool)
        .await?;

        for permission in admin_permissions {
            let role_permission_id = Ulid::new().to_string();
            let permission_id: String = permission.get("id");
            sqlx::query(
                r#"
                INSERT INTO role_permissions (id, role_id, permission_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (role_id, permission_id) DO NOTHING
                "#
            )
            .bind(role_permission_id)
            .bind(&admin_role_id)
            .bind(permission_id)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;
        }

        // Assign read permissions to user role
        let user_permissions = sqlx::query(
            "SELECT id FROM permissions WHERE action = 'read' AND guard_name = 'web'"
        )
        .fetch_all(pool)
        .await?;

        for permission in user_permissions {
            let role_permission_id = Ulid::new().to_string();
            let permission_id: String = permission.get("id");
            sqlx::query(
                r#"
                INSERT INTO role_permissions (id, role_id, permission_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (role_id, permission_id) DO NOTHING
                "#
            )
            .bind(role_permission_id)
            .bind(&user_role_id)
            .bind(permission_id)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;
        }

        // Assign content management permissions to moderator role
        let moderator_permission_names = vec!["posts.create", "posts.read", "posts.update", "posts.delete", "users.read"];

        for permission_name in moderator_permission_names {
            if let Ok(permission) = sqlx::query(
                "SELECT id FROM permissions WHERE name = $1 AND guard_name = 'web'"
            )
            .bind(permission_name)
            .fetch_one(pool)
            .await
            {
                let role_permission_id = Ulid::new().to_string();
                let permission_id: String = permission.get("id");
                sqlx::query(
                    r#"
                    INSERT INTO role_permissions (id, role_id, permission_id, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT (role_id, permission_id) DO NOTHING
                    "#
                )
                .bind(role_permission_id)
                .bind(&moderator_role_id)
                .bind(permission_id)
                .bind(now)
                .bind(now)
                .execute(pool)
                .await?;
            }
        }

        // Assign roles to users
        if let Ok(admin_user) = sqlx::query(
            "SELECT id FROM users WHERE email = 'admin@example.com'"
        )
        .fetch_one(pool)
        .await
        {
            let user_role_id_record = Ulid::new().to_string();
            let admin_user_id: String = admin_user.get("id");
            sqlx::query(
                r#"
                INSERT INTO user_roles (id, user_id, role_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (user_id, role_id) DO NOTHING
                "#
            )
            .bind(user_role_id_record)
            .bind(admin_user_id)
            .bind(&admin_role_id)
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
            let user_role_id_record = Ulid::new().to_string();
            let regular_user_id: String = regular_user.get("id");
            sqlx::query(
                r#"
                INSERT INTO user_roles (id, user_id, role_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (user_id, role_id) DO NOTHING
                "#
            )
            .bind(user_role_id_record)
            .bind(regular_user_id)
            .bind(&user_role_id)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;
        }

        println!("✅ Roles and permissions seeded successfully!");
        Ok(())
    }
}