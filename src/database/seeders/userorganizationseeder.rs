use crate::app::models::{user_organization::NewUserOrganization, DieselUlid};
use crate::database::seeder::Seeder;
use crate::database::DbPool;
use crate::schema::{organization_positions, organizations, sys_users, user_organizations};
use anyhow::Result;
use chrono::Utc;
use diesel::prelude::*;

pub struct UserOrganizationSeeder;

impl Seeder for UserOrganizationSeeder {
    fn class_name(&self) -> &'static str {
        "UserOrganizationSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seed user organization relationships with realistic assignments")
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("🌱 Seeding user organization relationships...");
        let mut conn = pool.get()?;

        // Get system user ID for audit tracking
        let system_user_id: String = sys_users::table
            .filter(sys_users::email.eq("system@seeder.internal"))
            .select(sys_users::id)
            .first(conn)?;

        // Get users with their emails for identification
        let users: Vec<(DieselUlid, String)> = sys_users::table
            .select((sys_users::id, sys_users::email))
            .load(&mut conn)?;

        // Get organizations with their codes for identification
        let organizations: Vec<(DieselUlid, Option<String>)> = organizations::table
            .select((organizations::id, organizations::code))
            .load(&mut conn)?;

        // Get positions
        let positions: Vec<DieselUlid> = organization_positions::table
            .select(organization_positions::id)
            .load(&mut conn)?;

        if users.is_empty() || organizations.is_empty() || positions.is_empty() {
            return Err(anyhow::anyhow!(
                "Users, organizations, and positions must be seeded first"
            ));
        }

        println!(
            "   Found {} users, {} organizations, {} positions",
            users.len(),
            organizations.len(),
            positions.len()
        );

        let now = Utc::now();
        let mut relationships = Vec::new();

        // Helper function to find organization by code
        let find_org_by_code = |code: &str| -> Option<DieselUlid> {
            organizations
                .iter()
                .find(|(_, org_code)| org_code.as_deref() == Some(code))
                .map(|(id, _)| *id)
        };

        // Helper function to find user by email prefix
        let find_user_by_email = |prefix: &str| -> Option<(DieselUlid, String)> {
            users
                .iter()
                .find(|(_, email)| email.starts_with(prefix))
                .map(|(id, email)| (*id, email.clone()))
        };

        // Executive Leadership in Holding Company
        let holding_id = find_org_by_code("TECH-HOLD");
        if let (Some(org_id), Some(pos_id)) = (holding_id, positions.get(0)) {
            // CEOs, CFOs, CTOs, COOs in holding company
            for email_prefix in &["jane.ceo", "john.cfo", "mary.cto", "bob.coo"] {
                if let Some((user_id, _)) = find_user_by_email(email_prefix) {
                    relationships.push(NewUserOrganization {
                        id: DieselUlid::new(),
                        user_id,
                        organization_id: org_id,
                        organization_position_id: *pos_id,
                        is_active: true,
                        started_at: now - chrono::Duration::days(2000),
                        ended_at: None,
                        created_at: now,
                        updated_at: now,
                        deleted_at: None,
                        created_by_id: system_user_id.clone(),
                        updated_by_id: system_user_id.clone(),
                        deleted_by_id: None,
                    });
                }
            }
        }

        // VPs in subsidiaries
        let subsidiaries = vec![
            ("TECH-SOFT", vec!["sarah.vp", "michael.vp"]),
            ("TECH-CONS", vec!["emily.vp"]),
            ("TECH-CLOUD", vec!["david.vp"]),
        ];

        for (code, vps) in subsidiaries {
            if let (Some(org_id), Some(pos_id)) = (find_org_by_code(code), positions.get(1)) {
                for vp_email in vps {
                    if let Some((user_id, _)) = find_user_by_email(vp_email) {
                        relationships.push(NewUserOrganization {
                            id: DieselUlid::new(),
                            user_id,
                            organization_id: org_id,
                            organization_position_id: *pos_id,
                            is_active: true,
                            started_at: now - chrono::Duration::days(1500),
                            ended_at: None,
                            created_at: now,
                            updated_at: now,
                            deleted_at: None,
                            created_by_id: system_user_id.clone(),
                            updated_by_id: system_user_id.clone(),
                            deleted_by_id: None,
                        });
                    }
                }
            }
        }

        // Directors in departments
        let department_assignments = vec![
            ("BACK-DEV", vec!["lisa.backend"]),
            ("FRONT-DEV", vec!["robert.frontend"]),
            ("MOBILE-DEV", vec!["jennifer.mobile"]),
            ("QA-DEPT", vec!["william.qa"]),
            ("DEVOPS", vec!["jessica.devops"]),
            ("PROD-MGT", vec!["christopher.pm"]),
            ("UX-DEPT", vec!["amanda.ux"]),
        ];

        for (code, directors) in department_assignments {
            if let (Some(org_id), Some(pos_id)) = (find_org_by_code(code), positions.get(2)) {
                for director_email in directors {
                    if let Some((user_id, _)) = find_user_by_email(director_email) {
                        relationships.push(NewUserOrganization {
                            id: DieselUlid::new(),
                            user_id,
                            organization_id: org_id,
                            organization_position_id: *pos_id,
                            is_active: true,
                            started_at: now - chrono::Duration::days(1000),
                            ended_at: None,
                            created_at: now,
                            updated_at: now,
                            deleted_at: None,
                            created_by_id: system_user_id.clone(),
                            updated_by_id: system_user_id.clone(),
                            deleted_by_id: None,
                        });
                    }
                }
            }
        }

        // Senior Engineers in departments
        let senior_assignments = vec![
            ("BACK-DEV", vec!["ashley.backend"]),
            ("FRONT-DEV", vec!["daniel.frontend"]),
            ("MOBILE-DEV", vec!["stephanie.mobile"]),
            ("DEVOPS", vec!["joshua.devops"]),
        ];

        for (code, seniors) in senior_assignments {
            if let (Some(org_id), Some(pos_id)) = (find_org_by_code(code), positions.get(3)) {
                for senior_email in seniors {
                    if let Some((user_id, _)) = find_user_by_email(senior_email) {
                        relationships.push(NewUserOrganization {
                            id: DieselUlid::new(),
                            user_id,
                            organization_id: org_id,
                            organization_position_id: *pos_id,
                            is_active: true,
                            started_at: now - chrono::Duration::days(800),
                            ended_at: None,
                            created_at: now,
                            updated_at: now,
                            deleted_at: None,
                            created_by_id: system_user_id.clone(),
                            updated_by_id: system_user_id.clone(),
                            deleted_by_id: None,
                        });
                    }
                }
            }
        }

        // Mid-level engineers in departments
        let mid_assignments = vec![
            ("BACK-DEV", vec!["michelle.backend"]),
            ("FRONT-DEV", vec!["andrew.frontend"]),
            ("MOBILE-DEV", vec!["elizabeth.ios", "ryan.android"]),
            ("QA-DEPT", vec!["nicole.qa"]),
        ];

        for (code, mids) in mid_assignments {
            if let (Some(org_id), Some(pos_id)) = (find_org_by_code(code), positions.get(4)) {
                for mid_email in mids {
                    if let Some((user_id, _)) = find_user_by_email(mid_email) {
                        relationships.push(NewUserOrganization {
                            id: DieselUlid::new(),
                            user_id,
                            organization_id: org_id,
                            organization_position_id: *pos_id,
                            is_active: true,
                            started_at: now - chrono::Duration::days(500),
                            ended_at: None,
                            created_at: now,
                            updated_at: now,
                            deleted_at: None,
                            created_by_id: system_user_id.clone(),
                            updated_by_id: system_user_id.clone(),
                            deleted_by_id: None,
                        });
                    }
                }
            }
        }

        // Junior engineers in teams
        let junior_assignments = vec![
            ("API-TEAM", vec!["brandon.junior"]),
            ("REACT-TEAM", vec!["samantha.junior"]),
            ("IOS-TEAM", vec!["kevin.junior"]),
            ("AUTO-TEST", vec!["rachel.junior"]),
            ("INFRA-TEAM", vec!["tyler.junior"]),
        ];

        for (code, juniors) in junior_assignments {
            if let (Some(org_id), Some(pos_id)) = (find_org_by_code(code), positions.get(5)) {
                for junior_email in juniors {
                    if let Some((user_id, _)) = find_user_by_email(junior_email) {
                        relationships.push(NewUserOrganization {
                            id: DieselUlid::new(),
                            user_id,
                            organization_id: org_id,
                            organization_position_id: *pos_id,
                            is_active: true,
                            started_at: now - chrono::Duration::days(300),
                            ended_at: None,
                            created_at: now,
                            updated_at: now,
                            deleted_at: None,
                            created_by_id: system_user_id.clone(),
                            updated_by_id: system_user_id.clone(),
                            deleted_by_id: None,
                        });
                    }
                }
            }
        }

        // Interns in teams
        let intern_assignments = vec![
            ("API-TEAM", vec!["madison.intern"]),
            ("REACT-TEAM", vec!["jordan.intern"]),
            ("IOS-TEAM", vec!["taylor.intern"]),
            ("AUTO-TEST", vec!["alex.intern"]),
            ("INFRA-TEAM", vec!["morgan.intern"]),
        ];

        for (code, interns) in intern_assignments {
            if let (Some(org_id), Some(pos_id)) = (find_org_by_code(code), positions.get(6)) {
                for intern_email in interns {
                    if let Some((user_id, _)) = find_user_by_email(intern_email) {
                        relationships.push(NewUserOrganization {
                            id: DieselUlid::new(),
                            user_id,
                            organization_id: org_id,
                            organization_position_id: *pos_id,
                            is_active: true,
                            started_at: now - chrono::Duration::days(90),
                            ended_at: None,
                            created_at: now,
                            updated_at: now,
                            deleted_at: None,
                            created_by_id: system_user_id.clone(),
                            updated_by_id: system_user_id.clone(),
                            deleted_by_id: None,
                        });
                    }
                }
            }
        }

        // Sales & Marketing in divisions
        let sales_marketing_assignments = vec![
            ("SALES-DIV", vec!["brian.sales", "patrick.account"]),
            ("MKTG-DIV", vec!["melissa.marketing"]),
            ("BIZ-DEV", vec!["rebecca.bizdev"]),
        ];

        for (code, people) in sales_marketing_assignments {
            if let (Some(org_id), Some(pos_id)) = (find_org_by_code(code), positions.get(2)) {
                for email in people {
                    if let Some((user_id, _)) = find_user_by_email(email) {
                        relationships.push(NewUserOrganization {
                            id: DieselUlid::new(),
                            user_id,
                            organization_id: org_id,
                            organization_position_id: *pos_id,
                            is_active: true,
                            started_at: now - chrono::Duration::days(600),
                            ended_at: None,
                            created_at: now,
                            updated_at: now,
                            deleted_at: None,
                            created_by_id: system_user_id.clone(),
                            updated_by_id: system_user_id.clone(),
                            deleted_by_id: None,
                        });
                    }
                }
            }
        }

        // Operations & Support
        let ops_assignments = vec![
            ("IT-SUPP", vec!["steven.it"]),
            ("NET-OPS", vec!["christina.network"]),
            ("DATA-ENG", vec!["kimberly.data"]),
        ];

        for (code, people) in ops_assignments {
            if let (Some(org_id), Some(pos_id)) = (find_org_by_code(code), positions.get(4)) {
                for email in people {
                    if let Some((user_id, _)) = find_user_by_email(email) {
                        relationships.push(NewUserOrganization {
                            id: DieselUlid::new(),
                            user_id,
                            organization_id: org_id,
                            organization_position_id: *pos_id,
                            is_active: true,
                            started_at: now - chrono::Duration::days(700),
                            ended_at: None,
                            created_at: now,
                            updated_at: now,
                            deleted_at: None,
                            created_by_id: system_user_id.clone(),
                            updated_by_id: system_user_id.clone(),
                            deleted_by_id: None,
                        });
                    }
                }
            }
        }

        // Consultants
        if let (Some(cons_org_id), Some(pos_id)) = (find_org_by_code("CONS-SERV"), positions.get(3))
        {
            for email in &["thomas.consultant", "angela.tech"] {
                if let Some((user_id, _)) = find_user_by_email(email) {
                    relationships.push(NewUserOrganization {
                        id: DieselUlid::new(),
                        user_id,
                        organization_id: cons_org_id,
                        organization_position_id: *pos_id,
                        is_active: true,
                        started_at: now - chrono::Duration::days(400),
                        ended_at: None,
                        created_at: now,
                        updated_at: now,
                        deleted_at: None,
                        created_by_id: system_user_id.clone(),
                        updated_by_id: system_user_id.clone(),
                        deleted_by_id: None,
                    });
                }
            }
        }

        // Cloud & Infrastructure
        let cloud_assignments = vec![
            ("PLAT-ENG", vec!["richard.cloud", "charles.platform"]),
            ("SRE", vec!["karen.sre"]),
        ];

        for (code, people) in cloud_assignments {
            if let (Some(org_id), Some(pos_id)) = (find_org_by_code(code), positions.get(3)) {
                for email in people {
                    if let Some((user_id, _)) = find_user_by_email(email) {
                        relationships.push(NewUserOrganization {
                            id: DieselUlid::new(),
                            user_id,
                            organization_id: org_id,
                            organization_position_id: *pos_id,
                            is_active: true,
                            started_at: now - chrono::Duration::days(650),
                            ended_at: None,
                            created_at: now,
                            updated_at: now,
                            deleted_at: None,
                            created_by_id: system_user_id.clone(),
                            updated_by_id: system_user_id.clone(),
                            deleted_by_id: None,
                        });
                    }
                }
            }
        }

        // Security Team
        let security_assignments = vec![
            ("SOC-OPS", vec!["nancy.soc", "james.security"]),
            ("INC-RESP", vec!["jason.incident", "laura.threat"]),
        ];

        for (code, people) in security_assignments {
            if let (Some(org_id), Some(pos_id)) = (find_org_by_code(code), positions.get(3)) {
                for email in people {
                    if let Some((user_id, _)) = find_user_by_email(email) {
                        relationships.push(NewUserOrganization {
                            id: DieselUlid::new(),
                            user_id,
                            organization_id: org_id,
                            organization_position_id: *pos_id,
                            is_active: true,
                            started_at: now - chrono::Duration::days(550),
                            ended_at: None,
                            created_at: now,
                            updated_at: now,
                            deleted_at: None,
                            created_by_id: system_user_id.clone(),
                            updated_by_id: system_user_id.clone(),
                            deleted_by_id: None,
                        });
                    }
                }
            }
        }

        // Create some historical relationships (people who changed positions)
        // Example: Some people had previous roles
        if let (Some(user), Some(org_id), Some(pos_id)) = (
            find_user_by_email("ashley.backend"),
            find_org_by_code("FRONT-DEV"),
            positions.get(5),
        ) {
            relationships.push(NewUserOrganization {
                id: DieselUlid::new(),
                user_id: user.0,
                organization_id: org_id,
                organization_position_id: *pos_id,
                is_active: false,
                started_at: now - chrono::Duration::days(1200),
                ended_at: Some(now - chrono::Duration::days(800)),
                created_at: now,
                updated_at: now,
                deleted_at: None,
                created_by_id: system_user_id.clone(),
                updated_by_id: system_user_id.clone(),
                deleted_by_id: None,
            });
        }

        // Insert all relationships
        println!(
            "   Creating {} user-organization relationships...",
            relationships.len()
        );
        for user_org in relationships {
            diesel::insert_into(user_organizations::table)
                .values(&user_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
        }

        let total_count: i64 = user_organizations::table.count().get_result(&mut conn)?;

        println!(
            "✅ {} User Organization relationships seeded successfully!",
            total_count
        );
        Ok(())
    }
}
