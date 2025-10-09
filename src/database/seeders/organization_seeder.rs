use crate::database::seeder::Seeder;
use crate::database::DbPool;
use anyhow::Result;
use crate::app::models::{DieselUlid, organization::{NewOrganization, CreateOrganization}};
use diesel::prelude::*;
use crate::schema::{organization_domains, organization_types, organizations};
use std::collections::HashMap;

pub struct OrganizationSeeder;

impl Seeder for OrganizationSeeder {
    fn class_name(&self) -> &'static str {
        "OrganizationSeeder"
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("🌱 Seeding organizations...");
        let mut conn = pool.get()?;

        // Get domain and type IDs from seeded data
        let pvt_domain_id: DieselUlid = organization_domains::table
            .filter(organization_domains::code.eq("PVT"))
            .select(organization_domains::id)
            .first(&mut conn)?;

        // Get all private sector types
        let type_map: HashMap<String, DieselUlid> = organization_types::table
            .filter(organization_types::domain_id.eq(pvt_domain_id.to_string()))
            .select((organization_types::code, organization_types::id))
            .load::<(Option<String>, DieselUlid)>(&mut conn)?
            .into_iter()
            .filter_map(|(code, id)| code.map(|c| (c, id)))
            .collect();

        let hold_type_id = type_map.get("HOLD").or_else(|| type_map.get("CORP")).unwrap();
        let corp_type_id = type_map.get("CORP").unwrap();
        let comp_type_id = type_map.get("COMP").unwrap();
        let div_type_id = type_map.get("DIV").unwrap();
        let dept_type_id = type_map.get("DEPT").unwrap();
        let team_type_id = type_map.get("TEAM").unwrap();

        // Level 1: Create main holding company
        let holding_org = CreateOrganization {
            domain_id: pvt_domain_id,
            type_id: *hold_type_id,
            name: "TechCorp Holdings".to_string(),
            parent_id: None,
            code: Some("TECH-HOLD".to_string()),
            address: Some("123 Corporate Blvd, Tech City, TC 12345".to_string()),
            authorized_capital: Some(crate::app::models::DecimalWrapper::from(50000000)),
            business_activities: Some("Technology investment and management, software development, cloud services, consulting".to_string()),
            contact_persons: Some(serde_json::json!([
                {"name": "Jane CEO", "title": "Chief Executive Officer", "email": "ceo@techcorp.com", "phone": "+1-555-0101"},
                {"name": "John CFO", "title": "Chief Financial Officer", "email": "cfo@techcorp.com", "phone": "+1-555-0102"}
            ])),
            description: Some("Main holding company for technology businesses with focus on innovative software solutions".to_string()),
            email: Some("info@techcorp.com".to_string()),
            establishment_date: Some(chrono::NaiveDate::from_ymd_opt(2010, 1, 15).unwrap()),
            governance_structure: Some(serde_json::json!({
                "board_of_directors": ["Jane CEO", "John CFO", "Mary CTO", "Bob COO"],
                "committees": ["Audit Committee", "Compensation Committee", "Governance Committee"],
                "structure": "Corporation with Board of Directors"
            })),
            legal_status: Some("Corporation".to_string()),
            paid_capital: Some(crate::app::models::DecimalWrapper::from(25000000)),
            path: Some("/techcorp".to_string()),
            phone: Some("+1-555-0100".to_string()),
            registration_number: Some("REG-001".to_string()),
            tax_number: Some("TAX-001".to_string()),
            website: Some("https://techcorp.com".to_string()),
        };

        let new_org = NewOrganization::new(holding_org, None);
        let holding_id = new_org.id;
        diesel::insert_into(organizations::table)
            .values(&new_org)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        println!("   ✓ Created Level 1: TechCorp Holdings");

        // Level 2: Create subsidiary companies
        let subsidiaries = vec![
            ("TechCorp Software", "TECH-SOFT", "Software development and engineering services"),
            ("TechCorp Consulting", "TECH-CONS", "Technology consulting and advisory services"),
            ("TechCorp Cloud", "TECH-CLOUD", "Cloud infrastructure and SaaS solutions"),
            ("TechCorp Security", "TECH-SEC", "Cybersecurity products and services"),
        ];

        let mut subsidiary_ids = Vec::new();
        for (name, code, desc) in subsidiaries {
            let org = CreateOrganization {
                domain_id: pvt_domain_id,
                type_id: *corp_type_id,
                name: name.to_string(),
                parent_id: Some(holding_id),
                code: Some(code.to_string()),
                address: Some(format!("TechCorp Campus, {} Wing", code)),
                authorized_capital: Some(crate::app::models::DecimalWrapper::from(10000000)),
                business_activities: Some(desc.to_string()),
                contact_persons: Some(serde_json::json!([
                    {"name": format!("{} Director", name), "title": "Managing Director", "email": format!("{}@techcorp.com", code.to_lowercase()), "phone": "+1-555-0200"}
                ])),
                description: Some(desc.to_string()),
                email: Some(format!("{}@techcorp.com", code.to_lowercase())),
                establishment_date: Some(chrono::NaiveDate::from_ymd_opt(2015, 1, 1).unwrap()),
                governance_structure: Some(serde_json::json!({"structure": "Subsidiary", "reporting_to": "Board of Directors"})),
                legal_status: Some("Subsidiary".to_string()),
                paid_capital: Some(crate::app::models::DecimalWrapper::from(5000000)),
                path: Some(format!("/techcorp/{}", code.to_lowercase())),
                phone: Some("+1-555-0200".to_string()),
                registration_number: Some(format!("REG-{}", code)),
                tax_number: Some(format!("TAX-{}", code)),
                website: Some(format!("https://{}.techcorp.com", code.to_lowercase())),
            };

            let new_org = NewOrganization::new(org, None);
            let sub_id = new_org.id;
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            subsidiary_ids.push((sub_id, name, code));
            println!("   ✓ Created Level 2: {}", name);
        }

        // Level 3: Create divisions
        let divisions = vec![
            ("Engineering Division", "ENG-DIV", "Software engineering and development", 0),
            ("Product Division", "PROD-DIV", "Product management and strategy", 0),
            ("Operations Division", "OPS-DIV", "Operations and infrastructure management", 0),
            ("Sales Division", "SALES-DIV", "Sales and business development", 1),
            ("Marketing Division", "MKTG-DIV", "Marketing and brand management", 1),
            ("Consulting Services", "CONS-SERV", "Professional consulting services", 1),
            ("Cloud Operations", "CLOUD-OPS", "Cloud platform operations", 2),
            ("Infrastructure Services", "INFRA-SERV", "Infrastructure management services", 2),
            ("Security Operations", "SEC-OPS", "Security operations center", 3),
            ("Threat Intelligence", "THREAT-INT", "Cybersecurity threat intelligence", 3),
        ];

        let mut division_ids = Vec::new();
        for (name, code, desc, parent_idx) in divisions {
            let parent = subsidiary_ids[parent_idx];
            let org = CreateOrganization {
                domain_id: pvt_domain_id,
                type_id: *div_type_id,
                name: name.to_string(),
                parent_id: Some(parent.0),
                code: Some(code.to_string()),
                address: Some(format!("TechCorp Campus, {} - {}", parent.2, code)),
                authorized_capital: None,
                business_activities: Some(desc.to_string()),
                contact_persons: Some(serde_json::json!([
                    {"name": format!("{} VP", name), "title": "Vice President", "email": format!("{}@techcorp.com", code.to_lowercase().replace("-", ".")), "phone": "+1-555-0300"}
                ])),
                description: Some(desc.to_string()),
                email: Some(format!("{}@techcorp.com", code.to_lowercase().replace("-", "."))),
                establishment_date: Some(chrono::NaiveDate::from_ymd_opt(2016, 6, 1).unwrap()),
                governance_structure: Some(serde_json::json!({"structure": "Division", "reporting_to": "VP Level"})),
                legal_status: None,
                paid_capital: None,
                path: Some(format!("/techcorp/{}/{}", parent.2.to_lowercase(), code.to_lowercase())),
                phone: Some("+1-555-0300".to_string()),
                registration_number: None,
                tax_number: None,
                website: None,
            };

            let new_org = NewOrganization::new(org, None);
            let div_id = new_org.id;
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            division_ids.push((div_id, name, code));
            println!("   ✓ Created Level 3: {}", name);
        }

        // Level 4: Create departments
        let departments = vec![
            ("Backend Development", "BACK-DEV", "Backend systems and API development", 0),
            ("Frontend Development", "FRONT-DEV", "Frontend and UI development", 0),
            ("Mobile Development", "MOBILE-DEV", "Mobile application development", 0),
            ("Quality Assurance", "QA-DEPT", "Software testing and quality assurance", 0),
            ("DevOps Engineering", "DEVOPS", "DevOps and CI/CD", 0),
            ("Product Management", "PROD-MGT", "Product strategy and planning", 1),
            ("User Experience", "UX-DEPT", "User experience and design", 1),
            ("Data Engineering", "DATA-ENG", "Data platform and analytics", 2),
            ("IT Support", "IT-SUPP", "Internal IT support", 2),
            ("Network Operations", "NET-OPS", "Network management", 2),
            ("Business Development", "BIZ-DEV", "New business development", 3),
            ("Account Management", "ACCT-MGT", "Customer account management", 3),
            ("Digital Marketing", "DIG-MKTG", "Digital marketing campaigns", 4),
            ("Brand Management", "BRAND-MGT", "Brand strategy and management", 4),
            ("Enterprise Consulting", "ENT-CONS", "Enterprise consulting services", 5),
            ("Technical Consulting", "TECH-CONS-DEPT", "Technical consulting", 5),
            ("Platform Engineering", "PLAT-ENG", "Cloud platform engineering", 6),
            ("Site Reliability", "SRE", "Site reliability engineering", 6),
            ("SOC Operations", "SOC-OPS", "Security operations center", 8),
            ("Incident Response", "INC-RESP", "Incident response team", 8),
        ];

        let mut dept_ids = Vec::new();
        for (name, code, desc, parent_idx) in departments {
            let parent = division_ids[parent_idx];
            let org = CreateOrganization {
                domain_id: pvt_domain_id,
                type_id: *dept_type_id,
                name: name.to_string(),
                parent_id: Some(parent.0),
                code: Some(code.to_string()),
                address: Some(format!("TechCorp Campus, {} - {}", parent.2, code)),
                authorized_capital: None,
                business_activities: Some(desc.to_string()),
                contact_persons: Some(serde_json::json!([
                    {"name": format!("{} Director", name), "title": "Director", "email": format!("{}@techcorp.com", code.to_lowercase().replace("-", ".")), "phone": "+1-555-0400"}
                ])),
                description: Some(desc.to_string()),
                email: Some(format!("{}@techcorp.com", code.to_lowercase().replace("-", "."))),
                establishment_date: Some(chrono::NaiveDate::from_ymd_opt(2017, 1, 1).unwrap()),
                governance_structure: Some(serde_json::json!({"structure": "Department", "reporting_to": "Director Level"})),
                legal_status: None,
                paid_capital: None,
                path: Some(format!("/techcorp/departments/{}", code.to_lowercase())),
                phone: Some("+1-555-0400".to_string()),
                registration_number: None,
                tax_number: None,
                website: None,
            };

            let new_org = NewOrganization::new(org, None);
            let dept_id = new_org.id;
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            dept_ids.push((dept_id, name, code));
            println!("   ✓ Created Level 4: {}", name);
        }

        // Level 5: Create teams
        let teams = vec![
            ("API Development Team", "API-TEAM", "REST API development", 0),
            ("Microservices Team", "MICRO-TEAM", "Microservices architecture", 0),
            ("React Development Team", "REACT-TEAM", "React frontend development", 1),
            ("Vue Development Team", "VUE-TEAM", "Vue.js frontend development", 1),
            ("iOS Team", "IOS-TEAM", "iOS application development", 2),
            ("Android Team", "ANDROID-TEAM", "Android application development", 2),
            ("Automation Testing Team", "AUTO-TEST", "Test automation", 3),
            ("Manual Testing Team", "MANUAL-TEST", "Manual testing", 3),
            ("Infrastructure Team", "INFRA-TEAM", "Infrastructure automation", 4),
            ("Release Engineering Team", "REL-ENG", "Release management", 4),
        ];

        for (name, code, desc, parent_idx) in teams {
            let parent = dept_ids[parent_idx];
            let org = CreateOrganization {
                domain_id: pvt_domain_id,
                type_id: *team_type_id,
                name: name.to_string(),
                parent_id: Some(parent.0),
                code: Some(code.to_string()),
                address: Some(format!("TechCorp Campus, {} - {}", parent.2, code)),
                authorized_capital: None,
                business_activities: Some(desc.to_string()),
                contact_persons: Some(serde_json::json!([
                    {"name": format!("{} Lead", name), "title": "Team Lead", "email": format!("{}@techcorp.com", code.to_lowercase().replace("-", ".")), "phone": "+1-555-0500"}
                ])),
                description: Some(desc.to_string()),
                email: Some(format!("{}@techcorp.com", code.to_lowercase().replace("-", "."))),
                establishment_date: Some(chrono::NaiveDate::from_ymd_opt(2018, 1, 1).unwrap()),
                governance_structure: Some(serde_json::json!({"structure": "Team", "reporting_to": "Manager Level"})),
                legal_status: None,
                paid_capital: None,
                path: Some(format!("/techcorp/teams/{}", code.to_lowercase())),
                phone: Some("+1-555-0500".to_string()),
                registration_number: None,
                tax_number: None,
                website: None,
            };

            let new_org = NewOrganization::new(org, None);
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            println!("   ✓ Created Level 5: {}", name);
        }

        println!("✅ {} Organizations seeded successfully!", 1 + 4 + 10 + 20 + 10);
        Ok(())
    }
}
