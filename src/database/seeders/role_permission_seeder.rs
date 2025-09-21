use anyhow::Result;
use ulid::Ulid;
use chrono::Utc;
use crate::database::{seeder::Seeder, DbPool};
use crate::app::models::{HasModelType, user::User};
use diesel::prelude::*;
use crate::schema::{sys_permissions, sys_roles, sys_model_has_permissions, sys_model_has_roles, sys_users};

pub struct RolePermissionSeeder;

impl Seeder for RolePermissionSeeder {
    fn class_name(&self) -> &'static str {
        "RolePermissionSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seeds sys_roles, sys_permissions, and their relationships for RBAC")
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("🌱 Seeding sys_roles and permissions...");
        let mut conn = pool.get()?;

        let now = Utc::now();

        // Create permissions
        let permissions = vec![
            ("users.create", "api", Some("users"), "create"),
            ("users.read", "api", Some("users"), "read"),
            ("users.update", "api", Some("users"), "update"),
            ("users.delete", "api", Some("users"), "delete"),
            ("posts.create", "api", Some("posts"), "create"),
            ("posts.read", "api", Some("posts"), "read"),
            ("posts.update", "api", Some("posts"), "update"),
            ("posts.delete", "api", Some("posts"), "delete"),
            ("roles.create", "api", Some("roles"), "create"),
            ("roles.read", "api", Some("roles"), "read"),
            ("roles.update", "api", Some("roles"), "update"),
            ("roles.delete", "api", Some("roles"), "delete"),
            ("permissions.create", "api", Some("permissions"), "create"),
            ("permissions.read", "api", Some("permissions"), "read"),
            ("permissions.update", "api", Some("permissions"), "update"),
            ("permissions.delete", "api", Some("permissions"), "delete"),
        ];

        for (name, guard_name, resource, action) in permissions {
            let permission_id = Ulid::new().to_string();
            diesel::sql_query(
                r#"
                INSERT INTO sys_permissions (id, name, guard_name, resource, action, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (name, guard_name) DO NOTHING
                "#
            )
            .bind::<diesel::sql_types::Text, _>(permission_id)
            .bind::<diesel::sql_types::Text, _>(name)
            .bind::<diesel::sql_types::Text, _>(guard_name)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(resource)
            .bind::<diesel::sql_types::Text, _>(action)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .execute(&mut conn)?;
        }

        // Create roles
        let admin_role_id = Ulid::new().to_string();
        diesel::sql_query(
            r#"
            INSERT INTO sys_roles (id, name, description, guard_name, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (name, guard_name) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(&admin_role_id)
        .bind::<diesel::sql_types::Text, _>("admin")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Administrator with full access"))
        .bind::<diesel::sql_types::Text, _>("api")
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        let user_role_id = Ulid::new().to_string();
        diesel::sql_query(
            r#"
            INSERT INTO sys_roles (id, name, description, guard_name, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (name, guard_name) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(&user_role_id)
        .bind::<diesel::sql_types::Text, _>("user")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Regular user with limited access"))
        .bind::<diesel::sql_types::Text, _>("api")
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        let moderator_role_id = Ulid::new().to_string();
        diesel::sql_query(
            r#"
            INSERT INTO sys_roles (id, name, description, guard_name, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (name, guard_name) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(&moderator_role_id)
        .bind::<diesel::sql_types::Text, _>("moderator")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Moderator with content management access"))
        .bind::<diesel::sql_types::Text, _>("api")
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)?;

        // Assign all permissions to admin role using sys_model_has_permissions
        let admin_permissions: Vec<String> = diesel::sql_query(
            "SELECT id FROM sys_permissions WHERE guard_name = 'api'"
        )
        .load::<(String,)>(&mut conn)?
        .into_iter()
        .map(|(id,)| id)
        .collect();

        for permission_id in admin_permissions {
            let role_permission_id = Ulid::new().to_string();
            diesel::sql_query(
                r#"
                INSERT INTO sys_model_has_permissions (id, model_type, model_id, permission_id, scope_type, scope_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (model_type, model_id, permission_id) DO NOTHING
                "#
            )
            .bind::<diesel::sql_types::Text, _>(role_permission_id)
            .bind::<diesel::sql_types::Text, _>("Role")
            .bind::<diesel::sql_types::Text, _>(&admin_role_id)
            .bind::<diesel::sql_types::Text, _>(permission_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Option::<String>::None)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Option::<String>::None)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .execute(&mut conn)?;
        }

        // Assign read permissions to user role using sys_model_has_permissions
        let user_permissions: Vec<String> = diesel::sql_query(
            "SELECT id FROM sys_permissions WHERE action = 'read' AND guard_name = 'api'"
        )
        .load::<(String,)>(&mut conn)?
        .into_iter()
        .map(|(id,)| id)
        .collect();

        for permission_id in user_permissions {
            let role_permission_id = Ulid::new().to_string();
            diesel::sql_query(
                r#"
                INSERT INTO sys_model_has_permissions (id, model_type, model_id, permission_id, scope_type, scope_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (model_type, model_id, permission_id) DO NOTHING
                "#
            )
            .bind::<diesel::sql_types::Text, _>(role_permission_id)
            .bind::<diesel::sql_types::Text, _>("Role")
            .bind::<diesel::sql_types::Text, _>(&user_role_id)
            .bind::<diesel::sql_types::Text, _>(permission_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Option::<String>::None)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Option::<String>::None)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .execute(&mut conn)?;
        }

        // Assign content management permissions to moderator role using sys_model_has_permissions
        let moderator_permission_names = vec!["posts.create", "posts.read", "posts.update", "posts.delete", "users.read"];

        for permission_name in moderator_permission_names {
            let permissions: Vec<String> = diesel::sql_query(
                "SELECT id FROM sys_permissions WHERE name = $1 AND guard_name = 'api'"
            )
            .bind::<diesel::sql_types::Text, _>(permission_name)
            .load::<(String,)>(&mut conn)?
            .into_iter()
            .map(|(id,)| id)
            .collect();

            for permission_id in permissions {
                let role_permission_id = Ulid::new().to_string();
                diesel::sql_query(
                    r#"
                    INSERT INTO sys_model_has_permissions (id, model_type, model_id, permission_id, scope_type, scope_id, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    ON CONFLICT (model_type, model_id, permission_id) DO NOTHING
                    "#
                )
                .bind::<diesel::sql_types::Text, _>(role_permission_id)
                .bind::<diesel::sql_types::Text, _>("Role")
                .bind::<diesel::sql_types::Text, _>(&moderator_role_id)
                .bind::<diesel::sql_types::Text, _>(permission_id)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Option::<String>::None)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Option::<String>::None)
                .bind::<diesel::sql_types::Timestamptz, _>(now)
                .bind::<diesel::sql_types::Timestamptz, _>(now)
                .execute(&mut conn)?;
            }
        }

        // Assign sys_roles to users
        let admin_users: Vec<String> = diesel::sql_query(
            "SELECT id FROM sys_users WHERE email = 'admin@example.com'"
        )
        .load::<(String,)>(&mut conn)?
        .into_iter()
        .map(|(id,)| id)
        .collect();

        for admin_user_id in admin_users {
            let user_role_id_record = Ulid::new().to_string();
            diesel::sql_query(
                r#"
                INSERT INTO sys_model_has_roles (id, model_type, model_id, role_id, scope_type, scope_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (model_type, model_id, role_id) DO NOTHING
                "#
            )
            .bind::<diesel::sql_types::Text, _>(user_role_id_record)
            .bind::<diesel::sql_types::Text, _>(User::model_type())
            .bind::<diesel::sql_types::Text, _>(admin_user_id)
            .bind::<diesel::sql_types::Text, _>(&admin_role_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Option::<String>::None)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Option::<String>::None)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .execute(&mut conn)?;
        }

        let regular_users: Vec<String> = diesel::sql_query(
            "SELECT id FROM sys_users WHERE email = 'user@example.com'"
        )
        .load::<(String,)>(&mut conn)?
        .into_iter()
        .map(|(id,)| id)
        .collect();

        for regular_user_id in regular_users {
            let user_role_id_record = Ulid::new().to_string();
            diesel::sql_query(
                r#"
                INSERT INTO sys_model_has_roles (id, model_type, model_id, role_id, scope_type, scope_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (model_type, model_id, role_id) DO NOTHING
                "#
            )
            .bind::<diesel::sql_types::Text, _>(user_role_id_record)
            .bind::<diesel::sql_types::Text, _>(User::model_type())
            .bind::<diesel::sql_types::Text, _>(regular_user_id)
            .bind::<diesel::sql_types::Text, _>(&user_role_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Option::<String>::None)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Option::<String>::None)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .execute(&mut conn)?;
        }

        println!("✅ sys_Roles and sys_permissions seeded successfully!");
        Ok(())
    }
}