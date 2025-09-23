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
    organization_id: Option<String>,
    name: String,
    guard_name: String,
    resource: Option<String>,
    action: String,
    scope_type: Option<String>,
    scope_id: Option<String>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = sys_roles)]
struct NewRole {
    id: String,
    organization_id: Option<String>,
    name: String,
    description: Option<String>,
    guard_name: String,
    scope_type: Option<String>,
    scope_id: Option<String>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
    deleted_at: Option<chrono::DateTime<Utc>>,
    created_by: Option<String>,
    updated_by: Option<String>,
    deleted_by: Option<String>,
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
        Some("Seeds comprehensive roles, permissions, and relationships for all existing models with RBAC/ABAC support")
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("🌱 Seeding sys_roles and permissions...");
        let mut conn = pool.get()?;

        let now = Utc::now();

        // For now, use None for organization_id (global permissions/roles)
        let default_org_id: Option<String> = None;

        // Create permissions for all existing models
        let permissions = vec![
            // User Management
            ("users.create", "api", Some("users"), "create"),
            ("users.read", "api", Some("users"), "read"),
            ("users.update", "api", Some("users"), "update"),
            ("users.delete", "api", Some("users"), "delete"),
            ("users.restore", "api", Some("users"), "restore"),
            ("users.force_delete", "api", Some("users"), "force_delete"),
            ("users.assign_role", "api", Some("users"), "assign_role"),
            ("users.remove_role", "api", Some("users"), "remove_role"),
            ("users.view_profile", "api", Some("users"), "view_profile"),
            ("users.update_profile", "api", Some("users"), "update_profile"),
            ("users.reset_password", "api", Some("users"), "reset_password"),

            // Organization Management
            ("organizations.create", "api", Some("organizations"), "create"),
            ("organizations.read", "api", Some("organizations"), "read"),
            ("organizations.update", "api", Some("organizations"), "update"),
            ("organizations.delete", "api", Some("organizations"), "delete"),
            ("organizations.restore", "api", Some("organizations"), "restore"),
            ("organizations.force_delete", "api", Some("organizations"), "force_delete"),
            ("organizations.view_hierarchy", "api", Some("organizations"), "view_hierarchy"),
            ("organizations.manage_structure", "api", Some("organizations"), "manage_structure"),

            // Organization Position Levels
            ("organization_position_levels.create", "api", Some("organization_position_levels"), "create"),
            ("organization_position_levels.read", "api", Some("organization_position_levels"), "read"),
            ("organization_position_levels.update", "api", Some("organization_position_levels"), "update"),
            ("organization_position_levels.delete", "api", Some("organization_position_levels"), "delete"),
            ("organization_position_levels.restore", "api", Some("organization_position_levels"), "restore"),
            ("organization_position_levels.force_delete", "api", Some("organization_position_levels"), "force_delete"),

            // Organization Positions
            ("organization_positions.create", "api", Some("organization_positions"), "create"),
            ("organization_positions.read", "api", Some("organization_positions"), "read"),
            ("organization_positions.update", "api", Some("organization_positions"), "update"),
            ("organization_positions.delete", "api", Some("organization_positions"), "delete"),
            ("organization_positions.restore", "api", Some("organization_positions"), "restore"),
            ("organization_positions.force_delete", "api", Some("organization_positions"), "force_delete"),
            ("organization_positions.manage_salary", "api", Some("organization_positions"), "manage_salary"),
            ("organization_positions.manage_qualifications", "api", Some("organization_positions"), "manage_qualifications"),

            // User Organization Relationships
            ("user_organizations.create", "api", Some("user_organizations"), "create"),
            ("user_organizations.read", "api", Some("user_organizations"), "read"),
            ("user_organizations.update", "api", Some("user_organizations"), "update"),
            ("user_organizations.delete", "api", Some("user_organizations"), "delete"),
            ("user_organizations.restore", "api", Some("user_organizations"), "restore"),
            ("user_organizations.force_delete", "api", Some("user_organizations"), "force_delete"),
            ("user_organizations.activate", "api", Some("user_organizations"), "activate"),
            ("user_organizations.deactivate", "api", Some("user_organizations"), "deactivate"),
            ("user_organizations.transfer", "api", Some("user_organizations"), "transfer"),

            // Role & Permission Management (RBAC)
            ("roles.create", "api", Some("roles"), "create"),
            ("roles.read", "api", Some("roles"), "read"),
            ("roles.update", "api", Some("roles"), "update"),
            ("roles.delete", "api", Some("roles"), "delete"),
            ("roles.restore", "api", Some("roles"), "restore"),
            ("roles.force_delete", "api", Some("roles"), "force_delete"),
            ("roles.assign_permission", "api", Some("roles"), "assign_permission"),
            ("roles.remove_permission", "api", Some("roles"), "remove_permission"),

            ("permissions.create", "api", Some("permissions"), "create"),
            ("permissions.read", "api", Some("permissions"), "read"),
            ("permissions.update", "api", Some("permissions"), "update"),
            ("permissions.delete", "api", Some("permissions"), "delete"),
            ("permissions.restore", "api", Some("permissions"), "restore"),
            ("permissions.force_delete", "api", Some("permissions"), "force_delete"),

            // Geographic Data
            ("countries.create", "api", Some("countries"), "create"),
            ("countries.read", "api", Some("countries"), "read"),
            ("countries.update", "api", Some("countries"), "update"),
            ("countries.delete", "api", Some("countries"), "delete"),

            ("provinces.create", "api", Some("provinces"), "create"),
            ("provinces.read", "api", Some("provinces"), "read"),
            ("provinces.update", "api", Some("provinces"), "update"),
            ("provinces.delete", "api", Some("provinces"), "delete"),

            ("cities.create", "api", Some("cities"), "create"),
            ("cities.read", "api", Some("cities"), "read"),
            ("cities.update", "api", Some("cities"), "update"),
            ("cities.delete", "api", Some("cities"), "delete"),

            // Notifications
            ("notifications.create", "api", Some("notifications"), "create"),
            ("notifications.read", "api", Some("notifications"), "read"),
            ("notifications.update", "api", Some("notifications"), "update"),
            ("notifications.delete", "api", Some("notifications"), "delete"),
            ("notifications.send", "api", Some("notifications"), "send"),
            ("notifications.mark_read", "api", Some("notifications"), "mark_read"),
            ("notifications.mark_unread", "api", Some("notifications"), "mark_unread"),

            // Activity Logs & Audit
            ("activity_logs.create", "api", Some("activity_logs"), "create"),
            ("activity_logs.read", "api", Some("activity_logs"), "read"),
            ("activity_logs.update", "api", Some("activity_logs"), "update"),
            ("activity_logs.delete", "api", Some("activity_logs"), "delete"),
            ("activity_logs.view_audit", "api", Some("activity_logs"), "view_audit"),
            ("activity_logs.export", "api", Some("activity_logs"), "export"),

            // System & Administrative
            ("system.read", "api", Some("system"), "read"),
            ("system.update", "api", Some("system"), "update"),
            ("system.backup", "api", Some("system"), "backup"),
            ("system.restore", "api", Some("system"), "restore"),
            ("system.maintenance", "api", Some("system"), "maintenance"),

            // Database Migrations
            ("migrations.read", "api", Some("migrations"), "read"),
            ("migrations.run", "api", Some("migrations"), "run"),
            ("migrations.rollback", "api", Some("migrations"), "rollback"),
            ("migrations.status", "api", Some("migrations"), "status"),

            // OAuth & Authentication
            ("oauth.read", "api", Some("oauth"), "read"),
            ("oauth.create", "api", Some("oauth"), "create"),
            ("oauth.update", "api", Some("oauth"), "update"),
            ("oauth.delete", "api", Some("oauth"), "delete"),
            ("oauth.revoke_tokens", "api", Some("oauth"), "revoke_tokens"),

            // Reports & Analytics
            ("reports.create", "api", Some("reports"), "create"),
            ("reports.read", "api", Some("reports"), "read"),
            ("reports.update", "api", Some("reports"), "update"),
            ("reports.delete", "api", Some("reports"), "delete"),
            ("reports.export", "api", Some("reports"), "export"),
            ("reports.schedule", "api", Some("reports"), "schedule"),

            // Dashboard & Analytics
            ("dashboard.read", "api", Some("dashboard"), "read"),
            ("dashboard.create", "api", Some("dashboard"), "create"),
            ("dashboard.update", "api", Some("dashboard"), "update"),
            ("dashboard.delete", "api", Some("dashboard"), "delete"),
            ("dashboard.share", "api", Some("dashboard"), "share"),

            // Settings & Configuration
            ("settings.read", "api", Some("settings"), "read"),
            ("settings.update", "api", Some("settings"), "update"),
            ("settings.system", "api", Some("settings"), "system"),
            ("settings.user", "api", Some("settings"), "user"),
            ("settings.organization", "api", Some("settings"), "organization"),
        ];

        let new_permissions: Vec<NewPermission> = permissions
            .into_iter()
            .map(|(name, guard_name, resource, action)| NewPermission {
                id: Ulid::new().to_string(),
                organization_id: default_org_id.clone(),
                name: name.to_string(),
                guard_name: guard_name.to_string(),
                resource: resource.map(|r| r.to_string()),
                action: action.to_string(),
                scope_type: None,
                scope_id: None,
                created_at: now,
                updated_at: now,
            })
            .collect();

        for permission in new_permissions {
            insert_into(sys_permissions::table)
                .values(&permission)
                .on_conflict((sys_permissions::name, sys_permissions::guard_name, sys_permissions::scope_type, sys_permissions::scope_id))
                .do_nothing()
                .execute(&mut conn)?;
        }

        // Create roles
        let admin_role_id = Ulid::new().to_string();
        let admin_role = NewRole {
            id: admin_role_id.clone(),
            organization_id: default_org_id.clone(),
            name: "admin".to_string(),
            description: Some("Administrator with full access".to_string()),
            guard_name: "api".to_string(),
            scope_type: None,
            scope_id: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        };

        insert_into(sys_roles::table)
            .values(&admin_role)
            .on_conflict((sys_roles::organization_id, sys_roles::name, sys_roles::guard_name, sys_roles::scope_type, sys_roles::scope_id))
            .do_nothing()
            .execute(&mut conn)?;

        let user_role_id = Ulid::new().to_string();
        let user_role = NewRole {
            id: user_role_id.clone(),
            organization_id: default_org_id.clone(),
            name: "user".to_string(),
            description: Some("Regular user with limited access".to_string()),
            guard_name: "api".to_string(),
            scope_type: None,
            scope_id: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        };

        insert_into(sys_roles::table)
            .values(&user_role)
            .on_conflict((sys_roles::organization_id, sys_roles::name, sys_roles::guard_name, sys_roles::scope_type, sys_roles::scope_id))
            .do_nothing()
            .execute(&mut conn)?;

        let moderator_role_id = Ulid::new().to_string();
        let moderator_role = NewRole {
            id: moderator_role_id.clone(),
            organization_id: default_org_id.clone(),
            name: "moderator".to_string(),
            description: Some("Moderator with content management access".to_string()),
            guard_name: "api".to_string(),
            scope_type: None,
            scope_id: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
        };

        insert_into(sys_roles::table)
            .values(&moderator_role)
            .on_conflict((sys_roles::organization_id, sys_roles::name, sys_roles::guard_name, sys_roles::scope_type, sys_roles::scope_id))
            .do_nothing()
            .execute(&mut conn)?;

        // Create additional organizational roles
        let additional_roles = vec![
            ("CEO", "Chief Executive Officer"),
            ("CTO", "Chief Technology Officer"),
            ("CFO", "Chief Financial Officer"),
            ("CHRO", "Chief Human Resources Officer"),
            ("VP_Engineering", "Vice President of Engineering"),
            ("VP_Sales", "Vice President of Sales"),
            ("VP_Marketing", "Vice President of Marketing"),
            ("Director_IT", "Director of Information Technology"),
            ("Director_HR", "Director of Human Resources"),
            ("Director_Finance", "Director of Finance"),
            ("Manager_Development", "Development Manager"),
            ("Manager_QA", "Quality Assurance Manager"),
            ("Manager_DevOps", "DevOps Manager"),
            ("Manager_Product", "Product Manager"),
            ("Team_Lead_Frontend", "Frontend Team Lead"),
            ("Team_Lead_Backend", "Backend Team Lead"),
            ("Team_Lead_Mobile", "Mobile Team Lead"),
            ("Senior_Developer", "Senior Software Developer"),
            ("Junior_Developer", "Junior Software Developer"),
            ("QA_Engineer", "Quality Assurance Engineer"),
            ("DevOps_Engineer", "DevOps Engineer"),
            ("System_Administrator", "System Administrator"),
            ("Business_Analyst", "Business Analyst"),
            ("Project_Manager", "Project Manager"),
            ("Scrum_Master", "Scrum Master"),
        ];

        for (role_name, description) in additional_roles {
            let role_id = Ulid::new().to_string();
            let role = NewRole {
                id: role_id,
                organization_id: default_org_id.clone(),
                name: role_name.to_string(),
                description: Some(description.to_string()),
                guard_name: "api".to_string(),
                scope_type: None,
                scope_id: None,
                created_at: now,
                updated_at: now,
                deleted_at: None,
                created_by: None,
                updated_by: None,
                deleted_by: None,
            };

            insert_into(sys_roles::table)
                .values(&role)
                .on_conflict((sys_roles::organization_id, sys_roles::name, sys_roles::guard_name, sys_roles::scope_type, sys_roles::scope_id))
                .do_nothing()
                .execute(&mut conn)?;
        }

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

        // Assign organizational management permissions to moderator role using sys_model_has_permissions
        let moderator_permission_names = vec![
            "users.read", "users.update", "users.view_profile",
            "organizations.read", "organizations.update", "organizations.view_hierarchy",
            "organization_positions.read", "organization_positions.update",
            "organization_position_levels.read", "organization_position_levels.update",
            "user_organizations.read", "user_organizations.update", "user_organizations.activate", "user_organizations.deactivate",
            "notifications.create", "notifications.read", "notifications.send",
            "activity_logs.read", "activity_logs.view_audit",
            "reports.read", "reports.create",
            "dashboard.read"
        ];

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

        // Assign specific permissions to organizational roles

        // CEO role gets strategic permissions
        let ceo_permission_names = vec![
            "organizations.read", "organizations.update", "organizations.create", "organizations.view_hierarchy", "organizations.manage_structure",
            "users.read", "users.update", "users.assign_role", "users.remove_role",
            "reports.read", "reports.create", "reports.export", "reports.schedule",
            "dashboard.read", "dashboard.create", "dashboard.update", "dashboard.share",
            "settings.read", "settings.update", "settings.system", "settings.organization",
            "activity_logs.read", "activity_logs.view_audit", "activity_logs.export"
        ];
        assign_permissions_to_role(&mut conn, "CEO", &ceo_permission_names, now)?;

        // CTO role gets technical permissions
        let cto_permission_names = vec![
            "organizations.read", "organizations.update", "organizations.view_hierarchy",
            "organization_positions.read", "organization_positions.update", "organization_positions.create",
            "organization_position_levels.read", "organization_position_levels.update", "organization_position_levels.create",
            "users.read", "users.update", "users.assign_role",
            "system.read", "system.update", "system.backup", "system.restore", "system.maintenance",
            "migrations.read", "migrations.run", "migrations.rollback", "migrations.status",
            "reports.read", "reports.create", "reports.export",
            "dashboard.read", "dashboard.create", "dashboard.update",
            "settings.read", "settings.update", "settings.system"
        ];
        assign_permissions_to_role(&mut conn, "CTO", &cto_permission_names, now)?;

        // Manager roles get team management permissions
        let manager_permission_names = vec![
            "users.read", "users.update", "users.view_profile", "users.update_profile",
            "organizations.read", "organizations.view_hierarchy",
            "organization_positions.read", "organization_position_levels.read",
            "user_organizations.read", "user_organizations.update", "user_organizations.activate", "user_organizations.deactivate",
            "notifications.create", "notifications.read", "notifications.send",
            "reports.read", "reports.create",
            "dashboard.read", "dashboard.create"
        ];
        let manager_roles = vec!["Manager_Development", "Manager_QA", "Manager_DevOps", "Manager_Product"];
        for role_name in manager_roles {
            assign_permissions_to_role(&mut conn, role_name, &manager_permission_names, now)?;
        }

        // Team Lead roles get limited management permissions
        let team_lead_permission_names = vec![
            "users.read", "users.view_profile",
            "organizations.read", "organizations.view_hierarchy",
            "organization_positions.read", "organization_position_levels.read",
            "user_organizations.read",
            "notifications.create", "notifications.read",
            "reports.read",
            "dashboard.read"
        ];
        let team_lead_roles = vec!["Team_Lead_Frontend", "Team_Lead_Backend", "Team_Lead_Mobile"];
        for role_name in team_lead_roles {
            assign_permissions_to_role(&mut conn, role_name, &team_lead_permission_names, now)?;
        }

        // Developer roles get basic permissions
        let developer_permission_names = vec![
            "users.read", "users.view_profile", "users.update_profile",
            "organizations.read",
            "organization_positions.read", "organization_position_levels.read",
            "user_organizations.read",
            "notifications.read", "notifications.mark_read", "notifications.mark_unread",
            "dashboard.read",
            "settings.read", "settings.user"
        ];
        let developer_roles = vec!["Senior_Developer", "Junior_Developer", "QA_Engineer", "DevOps_Engineer", "Business_Analyst"];
        for role_name in developer_roles {
            assign_permissions_to_role(&mut conn, role_name, &developer_permission_names, now)?;
        }

        let total_permissions = sys_permissions::table
            .filter(sys_permissions::guard_name.eq("api"))
            .count()
            .get_result::<i64>(&mut conn)?;

        println!("✅ Roles and permissions seeded successfully with {} permissions for all existing models!", total_permissions);
        Ok(())
    }
}

// Helper function to assign permissions to a role
fn assign_permissions_to_role(
    conn: &mut diesel::PgConnection,
    role_name: &str,
    permission_names: &[&str],
    now: chrono::DateTime<Utc>
) -> Result<()> {
    use diesel::prelude::*;
    use crate::schema::{sys_roles, sys_permissions, sys_model_has_permissions};

    // Get role ID
    let role_id: Option<String> = sys_roles::table
        .filter(sys_roles::name.eq(role_name))
        .filter(sys_roles::guard_name.eq("api"))
        .select(sys_roles::id)
        .first(conn)
        .optional()?;

    if let Some(role_id) = role_id {
        for permission_name in permission_names {
            let permission_ids: Vec<String> = sys_permissions::table
                .filter(sys_permissions::name.eq(permission_name))
                .filter(sys_permissions::guard_name.eq("api"))
                .select(sys_permissions::id)
                .load(conn)?;

            for permission_id in permission_ids {
                let role_permission = NewModelHasPermission {
                    id: Ulid::new().to_string(),
                    model_type: "Role".to_string(),
                    model_id: role_id.clone(),
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
                    .execute(conn)?;
            }
        }
    }

    Ok(())
}