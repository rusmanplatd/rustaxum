use anyhow::Result;
use ulid::Ulid;
use chrono::Utc;
use crate::database::{seeder::Seeder, DbPool};
use crate::app::models::{HasModelType, user::User};
use diesel::prelude::*;
use diesel::insert_into;
use crate::schema::{sys_permissions, sys_roles, sys_model_has_permissions, sys_model_has_roles, sys_users};

#[derive(Insertable)]
#[diesel(table_name = sys_permissions)]
struct NewPermission {
    id: String,
    name: String,
    guard_name: String,
    resource: Option<String>,
    action: String,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = sys_roles)]
struct NewRole {
    id: String,
    name: String,
    description: Option<String>,
    guard_name: String,
    scope_type: Option<String>,
    scope_id: Option<String>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = sys_model_has_permissions)]
struct NewModelHasPermission {
    id: String,
    model_type: String,
    model_id: String,
    permission_id: String,
    scope_type: Option<String>,
    scope_id: Option<String>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = sys_model_has_roles)]
struct NewModelHasRole {
    id: String,
    model_type: String,
    model_id: String,
    role_id: String,
    scope_type: Option<String>,
    scope_id: Option<String>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

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

        let new_permissions: Vec<NewPermission> = permissions
            .into_iter()
            .map(|(name, guard_name, resource, action)| NewPermission {
                id: Ulid::new().to_string(),
                name: name.to_string(),
                guard_name: guard_name.to_string(),
                resource: resource.map(|r| r.to_string()),
                action: action.to_string(),
                created_at: now,
                updated_at: now,
            })
            .collect();

        for permission in new_permissions {
            insert_into(sys_permissions::table)
                .values(&permission)
                .on_conflict((sys_permissions::name, sys_permissions::guard_name))
                .do_nothing()
                .execute(&mut conn)?;
        }

        // Create roles
        let admin_role_id = Ulid::new().to_string();
        let admin_role = NewRole {
            id: admin_role_id.clone(),
            name: "admin".to_string(),
            description: Some("Administrator with full access".to_string()),
            guard_name: "api".to_string(),
            scope_type: None,
            scope_id: None,
            created_at: now,
            updated_at: now,
        };

        insert_into(sys_roles::table)
            .values(&admin_role)
            .on_conflict((sys_roles::name, sys_roles::guard_name))
            .do_nothing()
            .execute(&mut conn)?;

        let user_role_id = Ulid::new().to_string();
        let user_role = NewRole {
            id: user_role_id.clone(),
            name: "user".to_string(),
            description: Some("Regular user with limited access".to_string()),
            guard_name: "api".to_string(),
            scope_type: None,
            scope_id: None,
            created_at: now,
            updated_at: now,
        };

        insert_into(sys_roles::table)
            .values(&user_role)
            .on_conflict((sys_roles::name, sys_roles::guard_name))
            .do_nothing()
            .execute(&mut conn)?;

        let moderator_role_id = Ulid::new().to_string();
        let moderator_role = NewRole {
            id: moderator_role_id.clone(),
            name: "moderator".to_string(),
            description: Some("Moderator with content management access".to_string()),
            guard_name: "api".to_string(),
            scope_type: None,
            scope_id: None,
            created_at: now,
            updated_at: now,
        };

        insert_into(sys_roles::table)
            .values(&moderator_role)
            .on_conflict((sys_roles::name, sys_roles::guard_name))
            .do_nothing()
            .execute(&mut conn)?;

        // Assign all permissions to admin role using sys_model_has_permissions
        let admin_permissions: Vec<String> = sys_permissions::table
            .filter(sys_permissions::guard_name.eq("api"))
            .select(sys_permissions::id)
            .load::<String>(&mut conn)?;

        for permission_id in admin_permissions {
            let role_permission = NewModelHasPermission {
                id: Ulid::new().to_string(),
                model_type: "Role".to_string(),
                model_id: admin_role_id.clone(),
                permission_id,
                scope_type: None,
                scope_id: None,
                created_at: now,
                updated_at: now,
            };

            insert_into(sys_model_has_permissions::table)
                .values(&role_permission)
                .on_conflict((sys_model_has_permissions::model_type, sys_model_has_permissions::model_id, sys_model_has_permissions::permission_id))
                .do_nothing()
                .execute(&mut conn)?;
        }

        // Assign read permissions to user role using sys_model_has_permissions
        let user_permissions: Vec<String> = sys_permissions::table
            .filter(sys_permissions::action.eq("read"))
            .filter(sys_permissions::guard_name.eq("api"))
            .select(sys_permissions::id)
            .load::<String>(&mut conn)?;

        for permission_id in user_permissions {
            let role_permission = NewModelHasPermission {
                id: Ulid::new().to_string(),
                model_type: "Role".to_string(),
                model_id: user_role_id.clone(),
                permission_id,
                scope_type: None,
                scope_id: None,
                created_at: now,
                updated_at: now,
            };

            insert_into(sys_model_has_permissions::table)
                .values(&role_permission)
                .on_conflict((sys_model_has_permissions::model_type, sys_model_has_permissions::model_id, sys_model_has_permissions::permission_id))
                .do_nothing()
                .execute(&mut conn)?;
        }

        // Assign content management permissions to moderator role using sys_model_has_permissions
        let moderator_permission_names = vec!["posts.create", "posts.read", "posts.update", "posts.delete", "users.read"];

        for permission_name in moderator_permission_names {
            let permissions: Vec<String> = sys_permissions::table
                .filter(sys_permissions::name.eq(permission_name))
                .filter(sys_permissions::guard_name.eq("api"))
                .select(sys_permissions::id)
                .load::<String>(&mut conn)?;

            for permission_id in permissions {
                let role_permission = NewModelHasPermission {
                    id: Ulid::new().to_string(),
                    model_type: "Role".to_string(),
                    model_id: moderator_role_id.clone(),
                    permission_id,
                    scope_type: None,
                    scope_id: None,
                    created_at: now,
                    updated_at: now,
                };

                insert_into(sys_model_has_permissions::table)
                    .values(&role_permission)
                    .on_conflict((sys_model_has_permissions::model_type, sys_model_has_permissions::model_id, sys_model_has_permissions::permission_id))
                    .do_nothing()
                    .execute(&mut conn)?;
            }
        }

        // Assign sys_roles to users
        let admin_users: Vec<String> = sys_users::table
            .filter(sys_users::email.eq("admin@example.com"))
            .select(sys_users::id)
            .load::<String>(&mut conn)?;

        for admin_user_id in admin_users {
            let user_role = NewModelHasRole {
                id: Ulid::new().to_string(),
                model_type: User::model_type().to_string(),
                model_id: admin_user_id,
                role_id: admin_role_id.clone(),
                scope_type: None,
                scope_id: None,
                created_at: now,
                updated_at: now,
            };

            insert_into(sys_model_has_roles::table)
                .values(&user_role)
                .on_conflict((sys_model_has_roles::model_type, sys_model_has_roles::model_id, sys_model_has_roles::role_id))
                .do_nothing()
                .execute(&mut conn)?;
        }

        let regular_users: Vec<String> = sys_users::table
            .filter(sys_users::email.eq("user@example.com"))
            .select(sys_users::id)
            .load::<String>(&mut conn)?;

        for regular_user_id in regular_users {
            let user_role = NewModelHasRole {
                id: Ulid::new().to_string(),
                model_type: User::model_type().to_string(),
                model_id: regular_user_id,
                role_id: user_role_id.clone(),
                scope_type: None,
                scope_id: None,
                created_at: now,
                updated_at: now,
            };

            insert_into(sys_model_has_roles::table)
                .values(&user_role)
                .on_conflict((sys_model_has_roles::model_type, sys_model_has_roles::model_id, sys_model_has_roles::role_id))
                .do_nothing()
                .execute(&mut conn)?;
        }

        println!("✅ sys_Roles and sys_permissions seeded successfully!");
        Ok(())
    }
}