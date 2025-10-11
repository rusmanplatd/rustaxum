use anyhow::Result;
use chrono::Utc;
use crate::database::{seeder::Seeder, DbPool};
use crate::app::models::{
    HasModelType,
    DieselUlid,
    permission::Permission,
    role::Role,
    sys_model_has_permission::SysModelHasPermission,
    sys_model_has_role::SysModelHasRole,
    user::User,
};
use diesel::prelude::*;
use diesel::insert_into;
use crate::schema::{sys_permissions, sys_roles, sys_model_has_permissions, sys_model_has_roles, sys_users};

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

        // Create or find system seeder user for audit tracking
        let system_user_id = get_or_create_system_user(&mut conn, now)?;

        // use None for organization_id (global permissions/roles)
        let default_org_id: Option<String> = None;

        // Create comprehensive permissions for all system features
        let permissions = vec![
            // User Management - Core CRUD
            ("api", Some("users"), "create"),
            ("api", Some("users"), "read"),
            ("api", Some("users"), "update"),
            ("api", Some("users"), "delete"),
            ("api", Some("users"), "restore"),
            ("api", Some("users"), "force_delete"),

            // User Management - Advanced Features
            ("api", Some("users"), "assign_role"),
            ("api", Some("users"), "remove_role"),
            ("api", Some("users"), "view_profile"),
            ("api", Some("users"), "update_profile"),
            ("api", Some("users"), "reset_password"),
            ("api", Some("users"), "change_password"),
            ("api", Some("users"), "lock_account"),
            ("api", Some("users"), "unlock_account"),
            ("api", Some("users"), "verify_email"),
            ("api", Some("users"), "send_verification"),
            ("api", Some("users"), "impersonate"),
            ("api", Some("users"), "export_data"),
            ("api", Some("users"), "import_data"),
            ("api", Some("users"), "bulk_update"),
            ("api", Some("users"), "bulk_delete"),

            // Organization Management
            ("api", Some("organizations"), "create"),
            ("api", Some("organizations"), "read"),
            ("api", Some("organizations"), "update"),
            ("api", Some("organizations"), "delete"),
            ("api", Some("organizations"), "restore"),
            ("api", Some("organizations"), "force_delete"),
            ("api", Some("organizations"), "view_hierarchy"),
            ("api", Some("organizations"), "manage_structure"),

            // Organization Position Levels
            ("api", Some("organization_position_levels"), "create"),
            ("api", Some("organization_position_levels"), "read"),
            ("api", Some("organization_position_levels"), "update"),
            ("api", Some("organization_position_levels"), "delete"),
            ("api", Some("organization_position_levels"), "restore"),
            ("api", Some("organization_position_levels"), "force_delete"),

            // Organization Positions
            ("api", Some("organization_positions"), "create"),
            ("api", Some("organization_positions"), "read"),
            ("api", Some("organization_positions"), "update"),
            ("api", Some("organization_positions"), "delete"),
            ("api", Some("organization_positions"), "restore"),
            ("api", Some("organization_positions"), "force_delete"),
            ("api", Some("organization_positions"), "manage_salary"),
            ("api", Some("organization_positions"), "manage_qualifications"),

            // User Organization Relationships
            ("api", Some("user_organizations"), "create"),
            ("api", Some("user_organizations"), "read"),
            ("api", Some("user_organizations"), "update"),
            ("api", Some("user_organizations"), "delete"),
            ("api", Some("user_organizations"), "restore"),
            ("api", Some("user_organizations"), "force_delete"),
            ("api", Some("user_organizations"), "activate"),
            ("api", Some("user_organizations"), "deactivate"),
            ("api", Some("user_organizations"), "transfer"),

            // Role & Permission Management (RBAC)
            ("api", Some("roles"), "create"),
            ("api", Some("roles"), "read"),
            ("api", Some("roles"), "update"),
            ("api", Some("roles"), "delete"),
            ("api", Some("roles"), "restore"),
            ("api", Some("roles"), "force_delete"),
            ("api", Some("roles"), "assign_permission"),
            ("api", Some("roles"), "remove_permission"),

            ("api", Some("permissions"), "create"),
            ("api", Some("permissions"), "read"),
            ("api", Some("permissions"), "update"),
            ("api", Some("permissions"), "delete"),
            ("api", Some("permissions"), "restore"),
            ("api", Some("permissions"), "force_delete"),

            // Geographic Data
            ("api", Some("countries"), "create"),
            ("api", Some("countries"), "read"),
            ("api", Some("countries"), "update"),
            ("api", Some("countries"), "delete"),

            ("api", Some("provinces"), "create"),
            ("api", Some("provinces"), "read"),
            ("api", Some("provinces"), "update"),
            ("api", Some("provinces"), "delete"),

            ("api", Some("cities"), "create"),
            ("api", Some("cities"), "read"),
            ("api", Some("cities"), "update"),
            ("api", Some("cities"), "delete"),

            // Notifications
            ("api", Some("notifications"), "create"),
            ("api", Some("notifications"), "read"),
            ("api", Some("notifications"), "update"),
            ("api", Some("notifications"), "delete"),
            ("api", Some("notifications"), "send"),
            ("api", Some("notifications"), "mark_read"),
            ("api", Some("notifications"), "mark_unread"),

            // Activity Logs & Audit
            ("api", Some("activity_logs"), "create"),
            ("api", Some("activity_logs"), "read"),
            ("api", Some("activity_logs"), "update"),
            ("api", Some("activity_logs"), "delete"),
            ("api", Some("activity_logs"), "view_audit"),
            ("api", Some("activity_logs"), "export"),

            // System & Administrative
            ("api", Some("system"), "read"),
            ("api", Some("system"), "update"),
            ("api", Some("system"), "backup"),
            ("api", Some("system"), "restore"),
            ("api", Some("system"), "maintenance"),

            // Database Migrations
            ("api", Some("migrations"), "read"),
            ("api", Some("migrations"), "run"),
            ("api", Some("migrations"), "rollback"),
            ("api", Some("migrations"), "status"),

            // OAuth & Authentication
            ("api", Some("oauth"), "read"),
            ("api", Some("oauth"), "create"),
            ("api", Some("oauth"), "update"),
            ("api", Some("oauth"), "delete"),
            ("api", Some("oauth"), "revoke_tokens"),

            // Reports & Analytics
            ("api", Some("reports"), "create"),
            ("api", Some("reports"), "read"),
            ("api", Some("reports"), "update"),
            ("api", Some("reports"), "delete"),
            ("api", Some("reports"), "export"),
            ("api", Some("reports"), "schedule"),

            // Dashboard & Analytics
            ("api", Some("dashboard"), "read"),
            ("api", Some("dashboard"), "create"),
            ("api", Some("dashboard"), "update"),
            ("api", Some("dashboard"), "delete"),
            ("api", Some("dashboard"), "share"),

            // Settings & Configuration
            ("api", Some("settings"), "read"),
            ("api", Some("settings"), "update"),
            ("api", Some("settings"), "system"),
            ("api", Some("settings"), "user"),
            ("api", Some("settings"), "organization"),
            ("api", Some("settings"), "security"),
            ("api", Some("settings"), "privacy"),
            ("api", Some("settings"), "backup"),
            ("api", Some("settings"), "restore"),
            ("api", Some("settings"), "import"),
            ("api", Some("settings"), "export"),

            // Multi-Factor Authentication (MFA)
            ("api", Some("mfa"), "enable"),
            ("api", Some("mfa"), "disable"),
            ("api", Some("mfa"), "verify"),
            ("api", Some("mfa"), "setup_totp"),
            ("api", Some("mfa"), "generate_backup_codes"),
            ("api", Some("mfa"), "use_backup_code"),
            ("api", Some("mfa"), "reset_methods"),
            ("api", Some("mfa"), "admin_disable"),
            ("api", Some("mfa"), "require_for_user"),
            ("api", Some("mfa"), "view_attempts"),

            // Devices & E2EE Management
            ("api", Some("devices"), "create"),
            ("api", Some("devices"), "read"),
            ("api", Some("devices"), "update"),
            ("api", Some("devices"), "delete"),
            ("api", Some("devices"), "register"),
            ("api", Some("devices"), "unregister"),
            ("api", Some("devices"), "list_user_devices"),
            ("api", Some("devices"), "rotate_keys"),
            ("api", Some("devices"), "verify_identity"),
            ("api", Some("devices"), "admin_revoke"),

            // Conversations & Messaging
            ("api", Some("conversations"), "create"),
            ("api", Some("conversations"), "read"),
            ("api", Some("conversations"), "update"),
            ("api", Some("conversations"), "delete"),
            ("api", Some("conversations"), "join"),
            ("api", Some("conversations"), "leave"),
            ("api", Some("conversations"), "invite_user"),
            ("api", Some("conversations"), "remove_user"),
            ("api", Some("conversations"), "change_type"),
            ("api", Some("conversations"), "enable_encryption"),
            ("api", Some("conversations"), "disable_encryption"),
            ("api", Some("conversations"), "set_disappearing_timer"),
            ("api", Some("conversations"), "admin_access"),

            // Messages
            ("api", Some("messages"), "create"),
            ("api", Some("messages"), "read"),
            ("api", Some("messages"), "update"),
            ("api", Some("messages"), "delete"),
            ("api", Some("messages"), "send"),
            ("api", Some("messages"), "edit"),
            ("api", Some("messages"), "recall"),
            ("api", Some("messages"), "forward"),
            ("api", Some("messages"), "react"),
            ("api", Some("messages"), "pin"),
            ("api", Some("messages"), "unpin"),
            ("api", Some("messages"), "mark_read"),
            ("api", Some("messages"), "mark_unread"),
            ("api", Some("messages"), "search"),
            ("api", Some("messages"), "export_history"),
            ("api", Some("messages"), "admin_delete"),

            // Scheduled Messages
            ("api", Some("scheduled_messages"), "create"),
            ("api", Some("scheduled_messages"), "read"),
            ("api", Some("scheduled_messages"), "update"),
            ("api", Some("scheduled_messages"), "delete"),
            ("api", Some("scheduled_messages"), "cancel"),
            ("api", Some("scheduled_messages"), "send_now"),

            // Conversation Participants
            ("api", Some("conversation_participants"), "create"),
            ("api", Some("conversation_participants"), "read"),
            ("api", Some("conversation_participants"), "update"),
            ("api", Some("conversation_participants"), "delete"),
            ("api", Some("conversation_participants"), "promote"),
            ("api", Some("conversation_participants"), "demote"),
            ("api", Some("conversation_participants"), "mute"),
            ("api", Some("conversation_participants"), "unmute"),

            // Signal Protocol & E2EE
            ("api", Some("signal_sessions"), "create"),
            ("api", Some("signal_sessions"), "read"),
            ("api", Some("signal_sessions"), "update"),
            ("api", Some("signal_sessions"), "delete"),
            ("api", Some("signal_sessions"), "reset"),

            ("api", Some("prekey_bundles"), "create"),
            ("api", Some("prekey_bundles"), "read"),
            ("api", Some("prekey_bundles"), "update"),
            ("api", Some("prekey_bundles"), "delete"),
            ("api", Some("prekey_bundles"), "rotate"),

            // OAuth Advanced Features
            ("api", Some("oauth_clients"), "create"),
            ("api", Some("oauth_clients"), "read"),
            ("api", Some("oauth_clients"), "update"),
            ("api", Some("oauth_clients"), "delete"),
            ("api", Some("oauth_clients"), "regenerate_secret"),
            ("api", Some("oauth_clients"), "revoke_tokens"),

            ("api", Some("oauth_tokens"), "create"),
            ("api", Some("oauth_tokens"), "read"),
            ("api", Some("oauth_tokens"), "update"),
            ("api", Some("oauth_tokens"), "delete"),
            ("api", Some("oauth_tokens"), "revoke"),
            ("api", Some("oauth_tokens"), "introspect"),

            ("api", Some("oauth_scopes"), "create"),
            ("api", Some("oauth_scopes"), "read"),
            ("api", Some("oauth_scopes"), "update"),
            ("api", Some("oauth_scopes"), "delete"),

            // Events & Notifications Advanced
            ("api", Some("events"), "create"),
            ("api", Some("events"), "read"),
            ("api", Some("events"), "update"),
            ("api", Some("events"), "delete"),
            ("api", Some("events"), "publish"),
            ("api", Some("events"), "subscribe"),
            ("api", Some("events"), "unsubscribe"),

            ("api", Some("push_subscriptions"), "create"),
            ("api", Some("push_subscriptions"), "read"),
            ("api", Some("push_subscriptions"), "update"),
            ("api", Some("push_subscriptions"), "delete"),
            ("api", Some("push_subscriptions"), "test"),

            // Jobs & Background Processing
            ("api", Some("jobs"), "create"),
            ("api", Some("jobs"), "read"),
            ("api", Some("jobs"), "update"),
            ("api", Some("jobs"), "delete"),
            ("api", Some("jobs"), "retry"),
            ("api", Some("jobs"), "cancel"),
            ("api", Some("jobs"), "pause_queue"),
            ("api", Some("jobs"), "resume_queue"),
            ("api", Some("jobs"), "clear_queue"),

            // Sessions Management
            ("api", Some("sessions"), "create"),
            ("api", Some("sessions"), "read"),
            ("api", Some("sessions"), "update"),
            ("api", Some("sessions"), "delete"),
            ("api", Some("sessions"), "revoke"),
            ("api", Some("sessions"), "revoke_all"),
            ("api", Some("sessions"), "admin_revoke"),

            // Geographic Data - Villages and Districts
            ("api", Some("districts"), "create"),
            ("api", Some("districts"), "read"),
            ("api", Some("districts"), "update"),
            ("api", Some("districts"), "delete"),

            ("api", Some("villages"), "create"),
            ("api", Some("villages"), "read"),
            ("api", Some("villages"), "update"),
            ("api", Some("villages"), "delete"),

            // Advanced System Operations
            ("api", Some("system"), "health_check"),
            ("api", Some("system"), "metrics"),
            ("api", Some("system"), "logs"),
            ("api", Some("system"), "clear_cache"),
            ("api", Some("system"), "optimize_database"),
            ("api", Some("system"), "cleanup_files"),
            ("api", Some("system"), "generate_report"),

            // Advanced Reports & Analytics
            ("api", Some("analytics"), "view"),
            ("api", Some("analytics"), "export"),
            ("api", Some("analytics"), "custom_query"),
            ("api", Some("analytics"), "user_behavior"),
            ("api", Some("analytics"), "security_events"),
            ("api", Some("analytics"), "performance_metrics"),
        ];

        let system_user_ulid = DieselUlid::from_string(&system_user_id).unwrap();

        let new_permissions: Vec<Permission> = permissions
            .into_iter()
            .map(|(guard_name, resource, action)| Permission {
                id: DieselUlid::new(),
                organization_id: default_org_id.as_ref().map(|id| DieselUlid::from_string(id).unwrap()),
                guard_name: guard_name.to_string(),
                resource: resource.map(|r| r.to_string()),
                action: action.to_string(),
                scope_type: None,
                scope_id: None,
                created_at: now,
                updated_at: now,
                created_by_id: system_user_ulid,
                updated_by_id: system_user_ulid,
                deleted_by_id: None,
            })
            .collect();

        // Insert permissions in batches of 50 for better performance
        for chunk in new_permissions.chunks(50) {
            insert_into(sys_permissions::table)
                .values(chunk)
                .on_conflict((sys_permissions::guard_name, sys_permissions::resource, sys_permissions::action, sys_permissions::scope_type, sys_permissions::scope_id))
                .do_nothing()
                .execute(&mut conn)?;
        }

        // Create roles
        let admin_role_id = DieselUlid::new();
        let admin_role = Role {
            id: admin_role_id,
            organization_id: default_org_id.as_ref().map(|id| DieselUlid::from_string(id).unwrap()),
            name: "admin".to_string(),
            description: Some("Administrator with full access".to_string()),
            guard_name: "api".to_string(),
            scope_type: None,
            scope_id: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by_id: system_user_ulid,
            updated_by_id: system_user_ulid,
            deleted_by_id: None,
        };

        insert_into(sys_roles::table)
            .values(&admin_role)
            .on_conflict((sys_roles::organization_id, sys_roles::name, sys_roles::guard_name, sys_roles::scope_type, sys_roles::scope_id))
            .do_nothing()
            .execute(&mut conn)?;

        let user_role_id = DieselUlid::new();
        let user_role = Role {
            id: user_role_id,
            organization_id: default_org_id.as_ref().map(|id| DieselUlid::from_string(id).unwrap()),
            name: "user".to_string(),
            description: Some("Regular user with limited access".to_string()),
            guard_name: "api".to_string(),
            scope_type: None,
            scope_id: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by_id: system_user_ulid,
            updated_by_id: system_user_ulid,
            deleted_by_id: None,
        };

        insert_into(sys_roles::table)
            .values(&user_role)
            .on_conflict((sys_roles::organization_id, sys_roles::name, sys_roles::guard_name, sys_roles::scope_type, sys_roles::scope_id))
            .do_nothing()
            .execute(&mut conn)?;

        let moderator_role_id = DieselUlid::new();
        let moderator_role = Role {
            id: moderator_role_id,
            organization_id: default_org_id.as_ref().map(|id| DieselUlid::from_string(id).unwrap()),
            name: "moderator".to_string(),
            description: Some("Moderator with content management access".to_string()),
            guard_name: "api".to_string(),
            scope_type: None,
            scope_id: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by_id: system_user_ulid,
            updated_by_id: system_user_ulid,
            deleted_by_id: None,
        };

        insert_into(sys_roles::table)
            .values(&moderator_role)
            .on_conflict((sys_roles::organization_id, sys_roles::name, sys_roles::guard_name, sys_roles::scope_type, sys_roles::scope_id))
            .do_nothing()
            .execute(&mut conn)?;

        // Create comprehensive organizational and functional roles
        let additional_roles = vec![
            // C-Level Executive Roles
            ("CEO", "Chief Executive Officer"),
            ("CTO", "Chief Technology Officer"),
            ("CFO", "Chief Financial Officer"),
            ("CHRO", "Chief Human Resources Officer"),
            ("COO", "Chief Operating Officer"),
            ("CMO", "Chief Marketing Officer"),
            ("CISO", "Chief Information Security Officer"),
            ("CDO", "Chief Data Officer"),

            // Vice President Roles
            ("VP_Engineering", "Vice President of Engineering"),
            ("VP_Sales", "Vice President of Sales"),
            ("VP_Marketing", "Vice President of Marketing"),
            ("VP_Operations", "Vice President of Operations"),
            ("VP_Finance", "Vice President of Finance"),
            ("VP_HR", "Vice President of Human Resources"),
            ("VP_Security", "Vice President of Security"),

            // Director Level Roles
            ("Director_IT", "Director of Information Technology"),
            ("Director_HR", "Director of Human Resources"),
            ("Director_Finance", "Director of Finance"),
            ("Director_Security", "Director of Security"),
            ("Director_Data", "Director of Data Analytics"),
            ("Director_Product", "Director of Product"),
            ("Director_Engineering", "Director of Engineering"),
            ("Director_Operations", "Director of Operations"),

            // Management Roles
            ("Manager_Development", "Development Manager"),
            ("Manager_QA", "Quality Assurance Manager"),
            ("Manager_DevOps", "DevOps Manager"),
            ("Manager_Product", "Product Manager"),
            ("Manager_Security", "Security Manager"),
            ("Manager_Data", "Data Manager"),
            ("Manager_Infrastructure", "Infrastructure Manager"),
            ("Manager_Support", "Support Manager"),

            // Team Lead Roles
            ("Team_Lead_Frontend", "Frontend Team Lead"),
            ("Team_Lead_Backend", "Backend Team Lead"),
            ("Team_Lead_Mobile", "Mobile Team Lead"),
            ("Team_Lead_Security", "Security Team Lead"),
            ("Team_Lead_Data", "Data Team Lead"),
            ("Team_Lead_DevOps", "DevOps Team Lead"),
            ("Team_Lead_QA", "QA Team Lead"),

            // Senior Technical Roles
            ("Senior_Developer", "Senior Software Developer"),
            ("Senior_Security_Engineer", "Senior Security Engineer"),
            ("Senior_Data_Engineer", "Senior Data Engineer"),
            ("Senior_DevOps_Engineer", "Senior DevOps Engineer"),
            ("Senior_QA_Engineer", "Senior QA Engineer"),
            ("Senior_Product_Manager", "Senior Product Manager"),
            ("Principal_Engineer", "Principal Software Engineer"),
            ("Staff_Engineer", "Staff Software Engineer"),
            ("Architect_Software", "Software Architect"),
            ("Architect_Security", "Security Architect"),
            ("Architect_Data", "Data Architect"),

            // Mid-Level Technical Roles
            ("Developer", "Software Developer"),
            ("QA_Engineer", "Quality Assurance Engineer"),
            ("DevOps_Engineer", "DevOps Engineer"),
            ("Security_Engineer", "Security Engineer"),
            ("Data_Engineer", "Data Engineer"),
            ("Data_Scientist", "Data Scientist"),
            ("Business_Analyst", "Business Analyst"),
            ("Product_Owner", "Product Owner"),

            // Junior Technical Roles
            ("Junior_Developer", "Junior Software Developer"),
            ("Junior_QA_Engineer", "Junior QA Engineer"),
            ("Junior_DevOps_Engineer", "Junior DevOps Engineer"),
            ("Junior_Security_Engineer", "Junior Security Engineer"),
            ("Junior_Data_Engineer", "Junior Data Engineer"),
            ("Intern_Developer", "Developer Intern"),
            ("Intern_QA", "QA Intern"),
            ("Intern_DevOps", "DevOps Intern"),

            // Operations & Support Roles
            ("System_Administrator", "System Administrator"),
            ("Database_Administrator", "Database Administrator"),
            ("Network_Administrator", "Network Administrator"),
            ("Support_Engineer", "Support Engineer"),
            ("Technical_Writer", "Technical Writer"),
            ("Release_Manager", "Release Manager"),
            ("Configuration_Manager", "Configuration Manager"),

            // Project & Process Roles
            ("Project_Manager", "Project Manager"),
            ("Scrum_Master", "Scrum Master"),
            ("Agile_Coach", "Agile Coach"),
            ("Process_Improvement", "Process Improvement Specialist"),

            // Functional Specialized Roles
            ("Security_Analyst", "Security Analyst"),
            ("Penetration_Tester", "Penetration Tester"),
            ("Compliance_Officer", "Compliance Officer"),
            ("Privacy_Officer", "Privacy Officer"),
            ("Audit_Manager", "Audit Manager"),
            ("Risk_Manager", "Risk Manager"),

            // Client & Business Roles
            ("Account_Manager", "Account Manager"),
            ("Sales_Engineer", "Sales Engineer"),
            ("Customer_Success", "Customer Success Manager"),
            ("Solutions_Architect", "Solutions Architect"),
            ("Consultant", "Technical Consultant"),

            // Special Access Roles
            ("Security_Admin", "Security Administrator - Full Security Access"),
            ("Audit_Viewer", "Audit Viewer - Read-only Audit Access"),
            ("Emergency_Response", "Emergency Response Team Member"),
            ("Compliance_Auditor", "Compliance Auditor - Full Audit Rights"),
            ("Data_Privacy_Officer", "Data Privacy Officer - Privacy Controls"),
            ("Incident_Responder", "Security Incident Responder"),
        ];

        for (role_name, description) in additional_roles {
            let role = Role {
                id: DieselUlid::new(),
                organization_id: default_org_id.as_ref().map(|id| DieselUlid::from_string(id).unwrap()),
                name: role_name.to_string(),
                description: Some(description.to_string()),
                guard_name: "api".to_string(),
                scope_type: None,
                scope_id: None,
                created_at: now,
                updated_at: now,
                deleted_at: None,
                created_by_id: system_user_ulid,
                updated_by_id: system_user_ulid,
                deleted_by_id: None,
            };

            insert_into(sys_roles::table)
                .values(&role)
                .on_conflict((sys_roles::organization_id, sys_roles::name, sys_roles::guard_name, sys_roles::scope_type, sys_roles::scope_id))
                .do_nothing()
                .execute(&mut conn)?;
        }

        // Assign all permissions to admin role using sys_model_has_permissions
        let admin_permissions: Vec<DieselUlid> = sys_permissions::table
            .filter(sys_permissions::guard_name.eq("api"))
            .select(sys_permissions::id)
            .load::<DieselUlid>(&mut conn)?;

        let admin_role_permissions: Vec<SysModelHasPermission> = admin_permissions
            .into_iter()
            .map(|permission_id| SysModelHasPermission {
                id: DieselUlid::new(),
                model_type: "Role".to_string(),
                model_id: admin_role_id,
                permission_id,
                scope_type: None,
                scope_id: None,
                created_at: now,
                updated_at: now,
                deleted_at: None,
                created_by_id: system_user_ulid,
                updated_by_id: system_user_ulid,
                deleted_by_id: None,
            })
            .collect();

        for chunk in admin_role_permissions.chunks(50) {
            insert_into(sys_model_has_permissions::table)
                .values(chunk)
                .on_conflict((sys_model_has_permissions::model_type, sys_model_has_permissions::model_id, sys_model_has_permissions::permission_id))
                .do_nothing()
                .execute(&mut conn)?;
        }

        // Assign read permissions to user role using sys_model_has_permissions
        let user_permissions: Vec<DieselUlid> = sys_permissions::table
            .filter(sys_permissions::action.eq("read"))
            .filter(sys_permissions::guard_name.eq("api"))
            .select(sys_permissions::id)
            .load::<DieselUlid>(&mut conn)?;

        let user_role_permissions: Vec<SysModelHasPermission> = user_permissions
            .into_iter()
            .map(|permission_id| SysModelHasPermission {
                id: DieselUlid::new(),
                model_type: "Role".to_string(),
                model_id: user_role_id,
                permission_id,
                scope_type: None,
                scope_id: None,
                created_at: now,
                updated_at: now,
                deleted_at: None,
                created_by_id: system_user_ulid,
                updated_by_id: system_user_ulid,
                deleted_by_id: None,
            })
            .collect();

        for chunk in user_role_permissions.chunks(50) {
            insert_into(sys_model_has_permissions::table)
                .values(chunk)
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
            let parts: Vec<&str> = permission_name.split('.').collect();
            if parts.len() != 2 { continue; }
            let (resource, action) = (parts[0], parts[1]);
            let permissions: Vec<DieselUlid> = sys_permissions::table
                .filter(sys_permissions::resource.eq(resource))
                .filter(sys_permissions::action.eq(action))
                .filter(sys_permissions::guard_name.eq("api"))
                .select(sys_permissions::id)
                .load::<DieselUlid>(&mut conn)?;

            for permission_id in permissions {
                let role_permission = SysModelHasPermission {
                    id: DieselUlid::new(),
                    model_type: "Role".to_string(),
                    model_id: moderator_role_id,
                    permission_id,
                    scope_type: None,
                    scope_id: None,
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                    created_by_id: system_user_ulid,
                    updated_by_id: system_user_ulid,
                    deleted_by_id: None,
                };

                insert_into(sys_model_has_permissions::table)
                    .values(&role_permission)
                    .on_conflict((sys_model_has_permissions::model_type, sys_model_has_permissions::model_id, sys_model_has_permissions::permission_id))
                    .do_nothing()
                    .execute(&mut conn)?;
            }
        }

        // Assign comprehensive roles to users based on email patterns

        // Get all users for role assignment
        let all_users: Vec<(String, String)> = sys_users::table
            .select((sys_users::id, sys_users::email))
            .load::<(String, String)>(&mut conn)?;

        println!("Found {} users for role assignment", all_users.len());

        // Admin users get admin role
        let admin_users: Vec<String> = all_users.iter()
            .filter(|(_, email)| email.contains("admin"))
            .map(|(id, _)| id.clone())
            .collect();

        for admin_user_id in admin_users {
            assign_user_role(&mut conn, &admin_user_id, &admin_role_id.to_string(), now)?;
        }

        // Regular users get user role (fallback for users without specific patterns)
        let user_pattern_users: Vec<String> = all_users.iter()
            .filter(|(_, email)| email.contains("user") && !email.contains("admin"))
            .map(|(id, _)| id.clone())
            .collect();

        for regular_user_id in user_pattern_users {
            assign_user_role(&mut conn, &regular_user_id, &user_role_id.to_string(), now)?;
        }

        // Assign roles based on email patterns for realistic organizational structure
        let role_email_patterns = vec![
            ("CEO", vec!["ceo", "chief.executive"]),
            ("CTO", vec!["cto", "chief.technology"]),
            ("CISO", vec!["ciso", "chief.security"]),
            ("Security_Admin", vec!["security.admin", "sec.admin"]),
            ("System_Administrator", vec!["sysadmin", "system.admin"]),
            ("Manager_Development", vec!["dev.manager", "development.manager"]),
            ("Senior_Developer", vec!["senior.dev", "sr.developer"]),
            ("Developer", vec!["developer", "dev"]),
            ("QA_Engineer", vec!["qa", "quality", "tester"]),
            ("DevOps_Engineer", vec!["devops", "ops"]),
            ("Data_Scientist", vec!["data.scientist", "analyst"]),
            ("Security_Engineer", vec!["security.engineer", "sec.eng"]),
            ("Compliance_Officer", vec!["compliance", "audit"]),
        ];

        let mut users_assigned = 0;
        for (role_name, email_patterns) in role_email_patterns {
            // Get role ID
            let role_id_opt: Option<DieselUlid> = sys_roles::table
                .filter(sys_roles::name.eq(role_name))
                .filter(sys_roles::guard_name.eq("api"))
                .select(sys_roles::id)
                .first(&mut conn)
                .optional()?;

            if let Some(role_id) = role_id_opt {
                for pattern in email_patterns {
                    let matching_users: Vec<String> = all_users.iter()
                        .filter(|(_, email)| email.to_lowercase().contains(pattern))
                        .map(|(id, _)| id.clone())
                        .collect();

                    for user_id in matching_users {
                        assign_user_role(&mut conn, &user_id, &role_id.to_string(), now)?;
                        users_assigned += 1;
                    }
                }
            }
        }

        // Assign moderator role to some users
        let moderator_users: Vec<String> = all_users.iter()
            .filter(|(_, email)| email.contains("moderator") || email.contains("mod"))
            .map(|(id, _)| id.clone())
            .collect();

        for mod_user_id in moderator_users {
            assign_user_role(&mut conn, &mod_user_id, &moderator_role_id.to_string(), now)?;
            users_assigned += 1;
        }

        // Assign remaining users to user role as fallback
        let assigned_user_ids: std::collections::HashSet<String> = sys_model_has_roles::table
            .filter(sys_model_has_roles::model_type.eq("User"))
            .select(sys_model_has_roles::model_id)
            .load::<String>(&mut conn)?
            .into_iter()
            .collect();

        let unassigned_users: Vec<String> = all_users.iter()
            .filter(|(id, _)| !assigned_user_ids.contains(id))
            .map(|(id, _)| id.clone())
            .collect();

        for unassigned_user_id in unassigned_users {
            assign_user_role(&mut conn, &unassigned_user_id, &user_role_id.to_string(), now)?;
            users_assigned += 1;
        }

        println!("Total user-role assignments created: {}", users_assigned);


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

        // Security Roles - Comprehensive Security Access
        let security_admin_permissions = vec![
            "users.read", "users.update", "users.lock_account", "users.unlock_account",
            "organizations.read", "organizations.update", "organizations.view_hierarchy",
            "mfa.admin_disable", "mfa.require_for_user", "mfa.view_attempts",
            "devices.read", "devices.admin_revoke", "devices.verify_identity",
            "sessions.read", "sessions.admin_revoke", "sessions.revoke_all",
            "oauth_tokens.read", "oauth_tokens.revoke", "oauth_tokens.introspect",
            "oauth_clients.read", "oauth_clients.revoke_tokens",
            "activity_logs.read", "activity_logs.view_audit", "activity_logs.export",
            "system.read", "system.health_check", "system.metrics", "system.logs",
            "analytics.security_events", "analytics.view", "analytics.export",
            "settings.read", "settings.security", "settings.privacy"
        ];
        let security_roles = vec!["CISO", "Director_Security", "Security_Admin", "Manager_Security"];
        for role_name in security_roles {
            assign_permissions_to_role(&mut conn, role_name, &security_admin_permissions, now)?;
        }

        // Data Roles - Data Management Access
        let data_permissions = vec![
            "users.export_data", "users.import_data", "users.read",
            "organizations.read", "organizations.view_hierarchy",
            "reports.read", "reports.create", "reports.export", "reports.schedule",
            "analytics.view", "analytics.export", "analytics.custom_query", "analytics.user_behavior", "analytics.performance_metrics",
            "dashboard.read", "dashboard.create", "dashboard.update", "dashboard.share",
            "system.generate_report", "system.metrics",
            "settings.read", "settings.privacy", "settings.backup", "settings.restore"
        ];
        let data_roles = vec!["CDO", "Director_Data", "Manager_Data", "Data_Scientist", "Data_Engineer", "Senior_Data_Engineer"];
        for role_name in data_roles {
            assign_permissions_to_role(&mut conn, role_name, &data_permissions, now)?;
        }

        // Senior Technical Roles
        let senior_tech_permissions = vec![
            "users.read", "users.view_profile", "users.update_profile",
            "organizations.read", "organizations.view_hierarchy",
            "organization_positions.read", "organization_position_levels.read",
            "user_organizations.read", "user_organizations.update",
            "devices.read", "devices.register", "devices.rotate_keys",
            "conversations.create", "conversations.read", "conversations.update",
            "messages.create", "messages.read", "messages.update", "messages.send",
            "notifications.read", "notifications.create", "notifications.mark_read",
            "jobs.read", "jobs.create", "jobs.retry", "jobs.cancel",
            "system.read", "system.health_check", "system.metrics",
            "reports.read", "reports.create",
            "dashboard.read", "dashboard.create",
            "settings.read", "settings.user", "settings.organization"
        ];
        let senior_tech_roles = vec!["Principal_Engineer", "Staff_Engineer", "Architect_Software", "Senior_Developer", "Senior_Security_Engineer", "Senior_DevOps_Engineer"];
        for role_name in senior_tech_roles {
            assign_permissions_to_role(&mut conn, role_name, &senior_tech_permissions, now)?;
        }

        // Regular Developer/Engineer Permissions
        let developer_permission_names = vec![
            "users.read", "users.view_profile", "users.update_profile",
            "organizations.read",
            "organization_positions.read", "organization_position_levels.read",
            "user_organizations.read",
            "devices.read", "devices.register", "devices.list_user_devices",
            "conversations.read", "conversations.create", "conversations.join", "conversations.leave",
            "messages.create", "messages.read", "messages.send", "messages.mark_read",
            "notifications.read", "notifications.mark_read", "notifications.mark_unread",
            "jobs.read",
            "dashboard.read",
            "settings.read", "settings.user"
        ];
        let developer_roles = vec!["Developer", "Junior_Developer", "QA_Engineer", "DevOps_Engineer", "Business_Analyst", "Support_Engineer"];
        for role_name in developer_roles {
            assign_permissions_to_role(&mut conn, role_name, &developer_permission_names, now)?;
        }

        // Operations & Admin Roles
        let ops_permissions = vec![
            "users.read", "users.update", "users.reset_password",
            "organizations.read", "organizations.update", "organizations.view_hierarchy",
            "user_organizations.read", "user_organizations.update", "user_organizations.activate", "user_organizations.deactivate",
            "sessions.read", "sessions.revoke", "sessions.admin_revoke",
            "oauth_tokens.read", "oauth_tokens.revoke",
            "jobs.read", "jobs.retry", "jobs.cancel", "jobs.pause_queue", "jobs.resume_queue",
            "system.read", "system.update", "system.maintenance", "system.backup", "system.restore",
            "system.health_check", "system.metrics", "system.logs", "system.clear_cache",
            "migrations.read", "migrations.run", "migrations.rollback", "migrations.status",
            "activity_logs.read", "activity_logs.view_audit",
            "settings.read", "settings.update", "settings.system", "settings.organization"
        ];
        let ops_roles = vec!["System_Administrator", "Database_Administrator", "Network_Administrator", "Release_Manager"];
        for role_name in ops_roles {
            assign_permissions_to_role(&mut conn, role_name, &ops_permissions, now)?;
        }

        // Audit & Compliance Roles
        let audit_permissions = vec![
            "users.read", "users.export_data",
            "organizations.read", "organizations.view_hierarchy",
            "user_organizations.read",
            "activity_logs.read", "activity_logs.view_audit", "activity_logs.export",
            "oauth_tokens.read", "oauth_tokens.introspect",
            "sessions.read",
            "mfa.view_attempts",
            "system.read", "system.metrics", "system.logs", "system.generate_report",
            "reports.read", "reports.create", "reports.export",
            "analytics.view", "analytics.export", "analytics.security_events",
            "dashboard.read"
        ];
        let audit_roles = vec!["Compliance_Officer", "Audit_Manager", "Compliance_Auditor", "Audit_Viewer", "Privacy_Officer"];
        for role_name in audit_roles {
            assign_permissions_to_role(&mut conn, role_name, &audit_permissions, now)?;
        }

        // Emergency Response Team
        let emergency_permissions = vec![
            "users.read", "users.lock_account", "users.unlock_account",
            "sessions.revoke_all", "sessions.admin_revoke",
            "oauth_tokens.revoke", "oauth_clients.revoke_tokens",
            "devices.admin_revoke",
            "mfa.admin_disable", "mfa.reset_methods",
            "system.read", "system.maintenance", "system.health_check",
            "activity_logs.read", "activity_logs.view_audit",
            "analytics.security_events"
        ];
        assign_permissions_to_role(&mut conn, "Emergency_Response", &emergency_permissions, now)?;
        assign_permissions_to_role(&mut conn, "Incident_Responder", &emergency_permissions, now)?;

        // Assign roles to users based on email patterns
        assign_roles_to_users(&mut conn, now)?;

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
    use crate::schema::{sys_users};

    // Get system user ID for audit tracking
    let system_user_id: DieselUlid = sys_users::table
        .filter(sys_users::email.eq("system@seeder.internal"))
        .select(sys_users::id)
        .first(conn)?;

    use crate::schema::{sys_roles, sys_permissions, sys_model_has_permissions};

    // Get role ID
    let role_id: Option<DieselUlid> = sys_roles::table
        .filter(sys_roles::name.eq(role_name))
        .filter(sys_roles::guard_name.eq("api"))
        .select(sys_roles::id)
        .first(conn)
        .optional()?;

    if let Some(role_id) = role_id {
        for permission_name in permission_names {
            let parts: Vec<&str> = permission_name.split('.').collect();
            if parts.len() != 2 { continue; }
            let (resource, action) = (parts[0], parts[1]);
            let permission_ids: Vec<DieselUlid> = sys_permissions::table
                .filter(sys_permissions::resource.eq(resource))
                .filter(sys_permissions::action.eq(action))
                .filter(sys_permissions::guard_name.eq("api"))
                .select(sys_permissions::id)
                .load(conn)?;

            for permission_id in permission_ids {
                let role_permission = SysModelHasPermission {
                    id: DieselUlid::new(),
                    model_type: "Role".to_string(),
                    model_id: role_id,
                    permission_id,
                    scope_type: None,
                    scope_id: None,
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                    created_by_id: system_user_id,
                    updated_by_id: system_user_id,
                    deleted_by_id: None,
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

// Helper function to get or create system user for audit tracking
fn get_or_create_system_user(
    conn: &mut diesel::PgConnection,
    now: chrono::DateTime<Utc>
) -> Result<String> {
    use diesel::prelude::*;
    use crate::schema::sys_users;
    use argon2::{Argon2, PasswordHasher, password_hash::{SaltString, rand_core::OsRng}};

    // Try to find existing system user
    let existing_user: Option<DieselUlid> = sys_users::table
        .filter(sys_users::email.eq("system@seeder.internal"))
        .select(sys_users::id)
        .first(conn)
        .optional()?;

    if let Some(user_id) = existing_user {
        return Ok(user_id.to_string());
    }

    // Create system user if not exists
    let system_user_id = DieselUlid::new();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2.hash_password(b"system-seeder-password", &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();

    let new_user = User {
        id: system_user_id,
        name: "System Seeder".to_string(),
        email: "system@seeder.internal".to_string(),
        email_verified_at: Some(now),
        username: Some("system_seeder".to_string()),
        password: hashed_password,
        remember_token: None,
        password_reset_token: None,
        password_reset_expires_at: None,
        refresh_token: None,
        refresh_token_expires_at: None,
        avatar: None,
        birthdate: None,
        failed_login_attempts: 0,
        google_id: None,
        last_login_at: None,
        last_seen_at: now,
        locale: None,
        locked_until: None,
        phone_number: None,
        phone_verified_at: None,
        zoneinfo: None,
        created_at: now,
        updated_at: now,
        deleted_at: None,
        created_by_id: system_user_id,
        updated_by_id: system_user_id,
        deleted_by_id: None,
        identity_public_key: None,
        identity_key_created_at: None,
        mfa_enabled: false,
        mfa_secret: None,
        mfa_backup_codes: None,
        mfa_required: false,
        email_notifications: Some(true),
        database_notifications: Some(true),
        broadcast_notifications: Some(true),
        web_push_notifications: Some(true),
        sms_notifications: Some(false),
        slack_notifications: Some(false),
        marketing_emails: Some(false),
        security_alerts: Some(true),
        order_updates: Some(false),
        newsletter: Some(false),
        promotional_emails: Some(false),
        account_notifications: Some(true),
    };

    insert_into(sys_users::table)
        .values(&new_user)
        .execute(conn)?;

    println!("Created system seeder user with ID: {}", system_user_id);
    Ok(system_user_id.to_string())
}

// Helper function to assign a role to a user
fn assign_user_role(
    conn: &mut diesel::PgConnection,
    user_id: &str,
    role_id: &str,
    now: chrono::DateTime<Utc>
) -> Result<()> {
    use diesel::prelude::*;
    use crate::schema::sys_users;

    // Get system user ID for audit tracking
    let system_user_id: DieselUlid = sys_users::table
        .filter(sys_users::email.eq("system@seeder.internal"))
        .select(sys_users::id)
        .first(conn)?;

    use crate::schema::sys_model_has_roles;
    use crate::app::models::{HasModelType, user::User};

    let model_id_ulid = DieselUlid::from_string(user_id.trim())
        .map_err(|e| anyhow::anyhow!("Invalid user_id ULID '{}': {}", user_id, e))?;
    let role_id_ulid = DieselUlid::from_string(role_id.trim())
        .map_err(|e| anyhow::anyhow!("Invalid role_id ULID '{}': {}", role_id, e))?;

    let user_role = SysModelHasRole {
        id: DieselUlid::new(),
        model_type: User::model_type().to_string(),
        model_id: model_id_ulid,
        role_id: role_id_ulid,
        scope_type: None,
        scope_id: None,
        created_at: now,
        updated_at: now,
        deleted_at: None,
        created_by_id: system_user_id,
        updated_by_id: system_user_id,
        deleted_by_id: None,
    };

    insert_into(sys_model_has_roles::table)
        .values(&user_role)
        .on_conflict((sys_model_has_roles::model_type, sys_model_has_roles::model_id, sys_model_has_roles::role_id))
        .do_nothing()
        .execute(conn)?;

    Ok(())
}

// Helper function to assign roles to users based on email patterns
fn assign_roles_to_users(
    conn: &mut diesel::PgConnection,
    now: chrono::DateTime<Utc>
) -> Result<()> {
    use diesel::prelude::*;
    use crate::schema::{sys_users, sys_roles};

    // Get all users except system user
    let users: Vec<(String, String)> = sys_users::table
        .filter(sys_users::email.ne("system@seeder.internal"))
        .filter(sys_users::deleted_at.is_null())
        .select((sys_users::id, sys_users::email))
        .load(conn)?;

    // Get all roles for assignment
    let all_roles: Vec<(String, String)> = sys_roles::table
        .filter(sys_roles::guard_name.eq("api"))
        .filter(sys_roles::deleted_at.is_null())
        .select((sys_roles::id, sys_roles::name))
        .load(conn)?;

    println!("Found {} users and {} roles for assignment", users.len(), all_roles.len());

    for (user_id, email) in users {
        // Assign roles based on email patterns
        let role_names = get_role_names_for_email(&email);

        for role_name in role_names {
            if let Some((role_id, _)) = all_roles.iter().find(|(_, name)| name == &role_name) {
                if let Err(e) = assign_user_role(conn, &user_id, role_id, now) {
                    eprintln!("Failed to assign role {} to user {}: {}", role_name, email, e);
                } else {
                    println!("Assigned role '{}' to user '{}'", role_name, email);
                }
            }
        }
    }

    // Count final role assignments
    let total_role_assignments = sys_model_has_roles::table
        .count()
        .get_result::<i64>(conn)?;

    println!("✅ Assigned roles to users. Total role assignments: {}", total_role_assignments);
    Ok(())
}

// Helper function to determine role names based on email patterns
fn get_role_names_for_email(email: &str) -> Vec<String> {
    let email_lower = email.to_lowercase();

    // Executive level - based on email patterns
    if email_lower.contains("ceo") || email_lower.contains("chief.executive") {
        return vec!["CEO".to_string()];
    }
    if email_lower.contains("cto") || email_lower.contains("chief.technology") {
        return vec!["CTO".to_string()];
    }
    if email_lower.contains("cfo") || email_lower.contains("chief.financial") {
        return vec!["CFO".to_string()];
    }
    if email_lower.contains("cio") || email_lower.contains("chief.information") {
        return vec!["CIO".to_string()];
    }
    if email_lower.contains("ciso") || email_lower.contains("chief.security") {
        return vec!["CISO".to_string()];
    }
    if email_lower.contains("cdo") || email_lower.contains("chief.data") {
        return vec!["CDO".to_string()];
    }

    // VP level
    if email_lower.contains("vp.engineering") || email_lower.contains("vice.president.engineering") {
        return vec!["VP_Engineering".to_string()];
    }
    if email_lower.contains("vp.security") || email_lower.contains("vice.president.security") {
        return vec!["VP_Security".to_string()];
    }
    if email_lower.contains("vp.data") || email_lower.contains("vice.president.data") {
        return vec!["VP_Data".to_string()];
    }

    // Director level
    if email_lower.contains("director.engineering") {
        return vec!["Director_Engineering".to_string()];
    }
    if email_lower.contains("director.security") {
        return vec!["Director_Security".to_string()];
    }
    if email_lower.contains("director.data") {
        return vec!["Director_Data".to_string()];
    }

    // Manager level
    if email_lower.contains("manager.development") || email_lower.contains("dev.manager") {
        return vec!["Manager_Development".to_string()];
    }
    if email_lower.contains("manager.qa") || email_lower.contains("qa.manager") {
        return vec!["Manager_QA".to_string()];
    }
    if email_lower.contains("manager.devops") || email_lower.contains("devops.manager") {
        return vec!["Manager_DevOps".to_string()];
    }
    if email_lower.contains("manager.security") || email_lower.contains("security.manager") {
        return vec!["Manager_Security".to_string()];
    }
    if email_lower.contains("manager.data") || email_lower.contains("data.manager") {
        return vec!["Manager_Data".to_string()];
    }

    // Technical roles
    if email_lower.contains("principal.engineer") {
        return vec!["Principal_Engineer".to_string()];
    }
    if email_lower.contains("staff.engineer") {
        return vec!["Staff_Engineer".to_string()];
    }
    if email_lower.contains("architect") {
        return vec!["Architect_Software".to_string()];
    }
    if email_lower.contains("senior.developer") || email_lower.contains("senior.engineer") {
        return vec!["Senior_Developer".to_string()];
    }
    if email_lower.contains("security.engineer") {
        return vec!["Senior_Security_Engineer".to_string()];
    }
    if email_lower.contains("devops.engineer") {
        return vec!["DevOps_Engineer".to_string()];
    }
    if email_lower.contains("data.scientist") {
        return vec!["Data_Scientist".to_string()];
    }
    if email_lower.contains("data.engineer") {
        return vec!["Data_Engineer".to_string()];
    }

    // Team leads
    if email_lower.contains("lead.frontend") || email_lower.contains("frontend.lead") {
        return vec!["Team_Lead_Frontend".to_string()];
    }
    if email_lower.contains("lead.backend") || email_lower.contains("backend.lead") {
        return vec!["Team_Lead_Backend".to_string()];
    }
    if email_lower.contains("lead.mobile") || email_lower.contains("mobile.lead") {
        return vec!["Team_Lead_Mobile".to_string()];
    }

    // Compliance and audit roles
    if email_lower.contains("compliance") {
        return vec!["Compliance_Officer".to_string()];
    }
    if email_lower.contains("audit") {
        return vec!["Audit_Manager".to_string()];
    }
    if email_lower.contains("privacy") {
        return vec!["Privacy_Officer".to_string()];
    }

    // System administration
    if email_lower.contains("sysadmin") || email_lower.contains("system.admin") {
        return vec!["System_Administrator".to_string()];
    }
    if email_lower.contains("dba") || email_lower.contains("database.admin") {
        return vec!["Database_Administrator".to_string()];
    }

    // General roles based on common patterns
    if email_lower.contains("developer") || email_lower.contains("dev") {
        return vec!["Developer".to_string()];
    }
    if email_lower.contains("qa") || email_lower.contains("quality") {
        return vec!["QA_Engineer".to_string()];
    }
    if email_lower.contains("analyst") {
        return vec!["Business_Analyst".to_string()];
    }
    if email_lower.contains("support") {
        return vec!["Support_Engineer".to_string()];
    }

    // Default role for all users
    vec!["user".to_string()]
}