use crate::app::models::{user_organization::UserOrganization, DieselUlid};
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
        let system_user_id_str: String = sys_users::table
            .filter(sys_users::email.eq("system@seeder.internal"))
            .select(sys_users::id)
            .first(&mut conn)
            .map_err(|e| anyhow::anyhow!("Failed to get system user: {}", e))?;
        let system_user_id = DieselUlid::from_string(&system_user_id_str)?;

        // Get users with their emails for identification
        let users_raw: Vec<(String, String)> = sys_users::table
            .select((sys_users::id, sys_users::email))
            .load(&mut conn)
            .map_err(|e| anyhow::anyhow!("Failed to load users: {}", e))?;

        let users: Vec<(DieselUlid, String)> = users_raw
            .into_iter()
            .filter_map(|(id_str, email)| {
                DieselUlid::from_string(&id_str).ok().map(|id| (id, email))
            })
            .collect();

        // Get organizations with their codes for identification
        let orgs_raw: Vec<(String, Option<String>)> = organizations::table
            .select((organizations::id, organizations::code))
            .load(&mut conn)
            .map_err(|e| anyhow::anyhow!("Failed to load organizations: {}", e))?;

        let organizations: Vec<(DieselUlid, Option<String>)> = orgs_raw
            .into_iter()
            .filter_map(|(id_str, code)| {
                DieselUlid::from_string(&id_str).ok().map(|id| (id, code))
            })
            .collect();

        // Get positions
        let positions_raw: Vec<String> = organization_positions::table
            .select(organization_positions::id)
            .load(&mut conn)
            .map_err(|e| anyhow::anyhow!("Failed to load positions: {}", e))?;

        let positions: Vec<DieselUlid> = positions_raw
            .into_iter()
            .filter_map(|id_str| DieselUlid::from_string(&id_str).ok())
            .collect();

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
                    relationships.push(UserOrganization {
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
                        relationships.push(UserOrganization {
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
                        relationships.push(UserOrganization {
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
                        relationships.push(UserOrganization {
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
                        relationships.push(UserOrganization {
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
                        relationships.push(UserOrganization {
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
                        relationships.push(UserOrganization {
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
                        relationships.push(UserOrganization {
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
                        relationships.push(UserOrganization {
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
                    relationships.push(UserOrganization {
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
                        relationships.push(UserOrganization {
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
                        relationships.push(UserOrganization {
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

        // ==================== GOVERNMENT (GOV) - Ministry of Digital Affairs ====================
        // Get Government org and positions
        let gov_ministry_id = find_org_by_code("MDA");
        let gov_agencies = vec![
            ("ITA", vec!["budi.it"]),
            ("DGA", vec!["siti.data"]),
            ("CSA", vec!["rudi.cyber"]),
        ];
        let gov_depts = vec![
            ("INFRA-DEV", vec!["rina.infra"]),
            ("DIG-SERV", vec!["fitri.services"]),
            ("SEC-OPS", vec!["agus.security"]),
        ];

        // Ministry Leadership
        if let (Some(org_id), Some(pos_id)) = (gov_ministry_id, positions.first()) {
            for email in &["sarah.minister", "ahmad.secgen", "dewi.dirgen"] {
                if let Some((user_id, _)) = find_user_by_email(email) {
                    relationships.push(UserOrganization {
                        id: DieselUlid::new(),
                        user_id,
                        organization_id: org_id,
                        organization_position_id: *pos_id,
                        is_active: true,
                        started_at: now - chrono::Duration::days(2500),
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

        // Government Agency Directors
        for (code, directors) in gov_agencies {
            if let (Some(org_id), Some(pos_id)) = (find_org_by_code(code), positions.get(3)) {
                for director in directors {
                    if let Some((user_id, _)) = find_user_by_email(director) {
                        relationships.push(UserOrganization {
                            id: DieselUlid::new(),
                            user_id,
                            organization_id: org_id,
                            organization_position_id: *pos_id,
                            is_active: true,
                            started_at: now - chrono::Duration::days(1800),
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

        // Government Department Staff
        for (code, staff) in gov_depts {
            if let (Some(org_id), Some(pos_id)) = (find_org_by_code(code), positions.get(5)) {
                for person in staff {
                    if let Some((user_id, _)) = find_user_by_email(person) {
                        relationships.push(UserOrganization {
                            id: DieselUlid::new(),
                            user_id,
                            organization_id: org_id,
                            organization_position_id: *pos_id,
                            is_active: true,
                            started_at: now - chrono::Duration::days(1200),
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

        // Government Analysts
        if let Some(org_id) = find_org_by_code("DIG-SERV") {
            if let Some(pos_id) = positions.get(12) {
                for email in &["hendra.policy", "maya.analyst", "doni.support"] {
                    if let Some((user_id, _)) = find_user_by_email(email) {
                        relationships.push(UserOrganization {
                            id: DieselUlid::new(),
                            user_id,
                            organization_id: org_id,
                            organization_position_id: *pos_id,
                            is_active: true,
                            started_at: now - chrono::Duration::days(900),
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

        // ==================== EDUCATION (EDU) - National University ====================
        let university_id = find_org_by_code("NATUNIV");

        // University Leadership
        if let (Some(org_id), Some(pos_id)) = (university_id, positions.first()) {
            for email in &["david.rector", "lisa.vice", "michael.dean"] {
                if let Some((user_id, _)) = find_user_by_email(email) {
                    relationships.push(UserOrganization {
                        id: DieselUlid::new(),
                        user_id,
                        organization_id: org_id,
                        organization_position_id: *pos_id,
                        is_active: true,
                        started_at: now - chrono::Duration::days(3000),
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

        // Faculty Professors in CS Department
        if let Some(org_id) = find_org_by_code("CS-DEPT") {
            if let Some(pos_id) = positions.get(3) {
                for email in &["jennifer.ai", "robert.networks", "patricia.db"] {
                    if let Some((user_id, _)) = find_user_by_email(email) {
                        relationships.push(UserOrganization {
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
        }

        // Research Staff in AI Lab
        if let Some(org_id) = find_org_by_code("AI-LAB") {
            if let Some(pos_id) = positions.get(11) {
                for email in &["william.research", "maria.lab", "thomas.research"] {
                    if let Some((user_id, _)) = find_user_by_email(email) {
                        relationships.push(UserOrganization {
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

        // Administrative Staff
        if let (Some(org_id), Some(pos_id)) = (university_id, positions.get(12)) {
            for email in &["susan.registrar", "george.admin"] {
                if let Some((user_id, _)) = find_user_by_email(email) {
                    relationships.push(UserOrganization {
                        id: DieselUlid::new(),
                        user_id,
                        organization_id: org_id,
                        organization_position_id: *pos_id,
                        is_active: true,
                        started_at: now - chrono::Duration::days(1800),
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

        // ==================== HEALTHCARE (HEA) - Regional Medical Center ====================
        let hospital_id = find_org_by_code("RMC");

        // Hospital Leadership
        if let (Some(org_id), Some(pos_id)) = (hospital_id, positions.first()) {
            for email in &["elizabeth.director", "richard.cmo", "margaret.cno"] {
                if let Some((user_id, _)) = find_user_by_email(email) {
                    relationships.push(UserOrganization {
                        id: DieselUlid::new(),
                        user_id,
                        organization_id: org_id,
                        organization_position_id: *pos_id,
                        is_active: true,
                        started_at: now - chrono::Duration::days(2800),
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

        // Department Heads
        let hospital_depts = vec![
            ("ER", vec!["james.emergency"]),
            ("SURG", vec!["linda.surgery"]),
            ("RAD", vec!["mark.radiology"]),
        ];

        for (code, heads) in hospital_depts {
            if let (Some(org_id), Some(pos_id)) = (find_org_by_code(code), positions.get(3)) {
                for head in heads {
                    if let Some((user_id, _)) = find_user_by_email(head) {
                        relationships.push(UserOrganization {
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
        }

        // Medical Staff
        if let Some(org_id) = find_org_by_code("ER") {
            if let Some(pos_id) = positions.get(11) {
                for email in &["karen.physician", "paul.surgeon", "jessica.nurse"] {
                    if let Some((user_id, _)) = find_user_by_email(email) {
                        relationships.push(UserOrganization {
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

        // Support Staff
        if let (Some(org_id), Some(pos_id)) = (hospital_id, positions.get(12)) {
            for email in &["angela.lab", "daniel.pharmacy", "rachel.it"] {
                if let Some((user_id, _)) = find_user_by_email(email) {
                    relationships.push(UserOrganization {
                        id: DieselUlid::new(),
                        user_id,
                        organization_id: org_id,
                        organization_position_id: *pos_id,
                        is_active: true,
                        started_at: now - chrono::Duration::days(1200),
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

        // ==================== NGO - Global Aid Foundation ====================
        let foundation_id = find_org_by_code("GAF");

        // NGO Leadership
        if let (Some(org_id), Some(pos_id)) = (foundation_id, positions.first()) {
            for email in &["catherine.ed", "mohammed.pd", "fatima.od"] {
                if let Some((user_id, _)) = find_user_by_email(email) {
                    relationships.push(UserOrganization {
                        id: DieselUlid::new(),
                        user_id,
                        organization_id: org_id,
                        organization_position_id: *pos_id,
                        is_active: true,
                        started_at: now - chrono::Duration::days(2200),
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

        // Program Managers
        let ngo_programs = vec![
            ("HEALTH", vec!["ahmed.field", "aisha.health"]),
            ("EDU", vec!["hassan.edu"]),
        ];

        for (code, managers) in ngo_programs {
            if let (Some(org_id), Some(pos_id)) = (find_org_by_code(code), positions.get(5)) {
                for manager in managers {
                    if let Some((user_id, _)) = find_user_by_email(manager) {
                        relationships.push(UserOrganization {
                            id: DieselUlid::new(),
                            user_id,
                            organization_id: org_id,
                            organization_position_id: *pos_id,
                            is_active: true,
                            started_at: now - chrono::Duration::days(1600),
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

        // Project Officers
        if let (Some(org_id), Some(pos_id)) = (foundation_id, positions.get(12)) {
            for email in &["leila.project", "omar.monitoring", "zainab.eval"] {
                if let Some((user_id, _)) = find_user_by_email(email) {
                    relationships.push(UserOrganization {
                        id: DieselUlid::new(),
                        user_id,
                        organization_id: org_id,
                        organization_position_id: *pos_id,
                        is_active: true,
                        started_at: now - chrono::Duration::days(1200),
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

        // Support Staff
        if let (Some(org_id), Some(pos_id)) = (foundation_id, positions.get(12)) {
            for email in &["yusuf.logistics", "mariam.fundraising"] {
                if let Some((user_id, _)) = find_user_by_email(email) {
                    relationships.push(UserOrganization {
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

        // ==================== MILITARY (MIL) - Defense Command ====================
        let command_id = find_org_by_code("CYBERCOM");

        // Command Leadership
        if let (Some(org_id), Some(pos_id)) = (command_id, positions.first()) {
            for email in &["robert.commander", "william.deputy", "barbara.chief"] {
                if let Some((user_id, _)) = find_user_by_email(email) {
                    relationships.push(UserOrganization {
                        id: DieselUlid::new(),
                        user_id,
                        organization_id: org_id,
                        organization_position_id: *pos_id,
                        is_active: true,
                        started_at: now - chrono::Duration::days(2600),
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

        // Operations Officers
        if let Some(org_id) = find_org_by_code("OPS-DIV") {
            if let Some(pos_id) = positions.get(3) {
                for email in &["charles.ops", "patricia.intel", "joseph.logistics"] {
                    if let Some((user_id, _)) = find_user_by_email(email) {
                        relationships.push(UserOrganization {
                            id: DieselUlid::new(),
                            user_id,
                            organization_id: org_id,
                            organization_position_id: *pos_id,
                            is_active: true,
                            started_at: now - chrono::Duration::days(1800),
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

        // Technical Staff
        if let Some(org_id) = find_org_by_code("CYBER-OPS") {
            if let Some(pos_id) = positions.get(11) {
                for email in &["steven.cyber", "jennifer.comms", "michael.systems"] {
                    if let Some((user_id, _)) = find_user_by_email(email) {
                        relationships.push(UserOrganization {
                            id: DieselUlid::new(),
                            user_id,
                            organization_id: org_id,
                            organization_position_id: *pos_id,
                            is_active: true,
                            started_at: now - chrono::Duration::days(1400),
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

        // ==================== RELIGIOUS (REL) - St. Joseph Diocese ====================
        let diocese_id = find_org_by_code("STJ-DIOC");

        // Diocese Leadership
        if let (Some(org_id), Some(pos_id)) = (diocese_id, positions.first()) {
            for email in &["francis.bishop", "anthony.vicar", "joseph.chancellor"] {
                if let Some((user_id, _)) = find_user_by_email(email) {
                    relationships.push(UserOrganization {
                        id: DieselUlid::new(),
                        user_id,
                        organization_id: org_id,
                        organization_position_id: *pos_id,
                        is_active: true,
                        started_at: now - chrono::Duration::days(3500),
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

        // Parish Priests
        let parishes = vec![
            ("STM", vec!["michael.parish", "patrick.mission"]),
            ("STP", vec!["thomas.community"]),
        ];

        for (code, priests) in parishes {
            if let (Some(org_id), Some(pos_id)) = (find_org_by_code(code), positions.get(3)) {
                for priest in priests {
                    if let Some((user_id, _)) = find_user_by_email(priest) {
                        relationships.push(UserOrganization {
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
        }

        // Religious Staff and Ministers
        if let Some(org_id) = find_org_by_code("YOUTH") {
            if let Some(pos_id) = positions.get(11) {
                for email in &["paul.youth", "mary.education", "catherine.charity"] {
                    if let Some((user_id, _)) = find_user_by_email(email) {
                        relationships.push(UserOrganization {
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

        // Administrative Staff
        if let (Some(org_id), Some(pos_id)) = (diocese_id, positions.get(12)) {
            for email in &["martha.admin", "john.finance"] {
                if let Some((user_id, _)) = find_user_by_email(email) {
                    relationships.push(UserOrganization {
                        id: DieselUlid::new(),
                        user_id,
                        organization_id: org_id,
                        organization_position_id: *pos_id,
                        is_active: true,
                        started_at: now - chrono::Duration::days(1800),
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

        // Create some historical relationships (people who changed positions)
        // Example: Some people had previous roles
        if let (Some(user), Some(org_id), Some(pos_id)) = (
            find_user_by_email("ashley.backend"),
            find_org_by_code("FRONT-DEV"),
            positions.get(5),
        ) {
            relationships.push(UserOrganization {
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
