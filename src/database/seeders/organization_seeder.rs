use crate::database::seeder::Seeder;
use crate::database::DbPool;
use anyhow::Result;
use crate::app::models::{DieselUlid, organization::{CreateOrganization, Organization}};
use diesel::prelude::*;
use crate::schema::{organization_domains, organization_types, organizations, sys_users};
use std::collections::HashMap;

pub struct OrganizationSeeder;

impl Seeder for OrganizationSeeder {
    fn class_name(&self) -> &'static str {
        "OrganizationSeeder"
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("🌱 Seeding organizations...");
        let mut conn = pool.get()?;

        // Get system user for audit fields
        let system_user_id: DieselUlid = sys_users::table
            .filter(sys_users::email.eq("system@seeder.internal"))
            .select(sys_users::id)
            .first(&mut conn)?;

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
        let _comp_type_id = type_map.get("COMP").unwrap();
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

        let new_org = Organization::new(holding_org, system_user_id);
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

            let new_org = Organization::new(org, system_user_id);
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

            let new_org = Organization::new(org, system_user_id);
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

            let new_org = Organization::new(org, system_user_id);
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

            let new_org = Organization::new(org, system_user_id);
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            println!("   ✓ Created Level 5: {}", name);
        }

        let mut total_orgs = 1 + 4 + 10 + 20 + 10; // TechCorp total

        // ==================== GOVERNMENT (GOV) - Ministry of Digital Affairs ====================
        let gov_domain_id: DieselUlid = organization_domains::table
            .filter(organization_domains::code.eq("GOV"))
            .select(organization_domains::id)
            .first(&mut conn)?;

        let gov_type_map: HashMap<String, DieselUlid> = organization_types::table
            .filter(organization_types::domain_id.eq(gov_domain_id.to_string()))
            .select((organization_types::code, organization_types::id))
            .load::<(Option<String>, DieselUlid)>(&mut conn)?
            .into_iter()
            .filter_map(|(code, id)| code.map(|c| (c, id)))
            .collect();

        // Ministry (Level 1)
        let ministry_type = gov_type_map.get("MIN").unwrap();
        let agency_type = gov_type_map.get("AGN").unwrap();
        let dept_type = gov_type_map.get("DEPT").unwrap();

        let ministry_org = CreateOrganization {
            domain_id: gov_domain_id,
            type_id: *ministry_type,
            name: "Ministry of Digital Affairs".to_string(),
            parent_id: None,
            code: Some("MDA".to_string()),
            address: Some("Jl. Merdeka No. 123, Jakarta 10110".to_string()),
            authorized_capital: None,
            business_activities: Some("Digital transformation, cybersecurity, data governance, IT infrastructure".to_string()),
            contact_persons: Some(serde_json::json!([
                {"name": "Dr. Sarah Minister", "title": "Minister", "email": "sarah.minister@gov.id", "phone": "+62-21-5001"}
            ])),
            description: Some("Government ministry responsible for digital transformation and technology policy".to_string()),
            email: Some("info@digital.gov.id".to_string()),
            establishment_date: Some(chrono::NaiveDate::from_ymd_opt(2015, 10, 20).unwrap()),
            governance_structure: Some(serde_json::json!({"structure": "Government Ministry", "reporting_to": "President"})),
            legal_status: Some("Government Ministry".to_string()),
            paid_capital: None,
            path: Some("/gov/mda".to_string()),
            phone: Some("+62-21-5000".to_string()),
            registration_number: Some("MIN-001".to_string()),
            tax_number: None,
            website: Some("https://digital.gov.id".to_string()),
        };

        let new_org = Organization::new(ministry_org, system_user_id);
        let ministry_id = new_org.id;
        diesel::insert_into(organizations::table)
            .values(&new_org)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        total_orgs += 1;

        // Agencies (Level 2)
        let agencies = vec![
            ("IT Infrastructure Agency", "ITA", "National IT infrastructure management"),
            ("Data Governance Agency", "DGA", "National data governance and protection"),
            ("Cyber Security Agency", "CSA", "National cybersecurity operations"),
        ];

        let mut agency_ids = Vec::new();
        for (name, code, desc) in agencies {
            let org = CreateOrganization {
                domain_id: gov_domain_id,
                type_id: *agency_type,
                name: name.to_string(),
                parent_id: Some(ministry_id),
                code: Some(code.to_string()),
                address: Some("Ministry Complex, Jakarta".to_string()),
                authorized_capital: None,
                business_activities: Some(desc.to_string()),
                contact_persons: Some(serde_json::json!([{"name": format!("{} Director", name), "title": "Director", "email": format!("{}@gov.id", code.to_lowercase())}])),
                description: Some(desc.to_string()),
                email: Some(format!("{}@gov.id", code.to_lowercase())),
                establishment_date: Some(chrono::NaiveDate::from_ymd_opt(2016, 1, 1).unwrap()),
                governance_structure: Some(serde_json::json!({"structure": "Agency", "reporting_to": "Minister"})),
                legal_status: Some("Government Agency".to_string()),
                paid_capital: None,
                path: Some(format!("/gov/mda/{}", code.to_lowercase())),
                phone: Some("+62-21-5100".to_string()),
                registration_number: Some(format!("AGN-{}", code)),
                tax_number: None,
                website: None,
            };

            let new_org = Organization::new(org, system_user_id);
            let agency_id = new_org.id;
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            agency_ids.push((agency_id, code));
            total_orgs += 1;
        }

        // Departments (Level 3)
        let gov_departments = vec![
            ("Infrastructure Development", "INFRA-DEV", "IT infrastructure development", 0),
            ("Digital Services", "DIG-SERV", "Public digital services", 1),
            ("Security Operations", "SEC-OPS", "Cybersecurity operations", 2),
        ];

        for (name, code, desc, parent_idx) in gov_departments {
            let parent = agency_ids[parent_idx];
            let org = CreateOrganization {
                domain_id: gov_domain_id,
                type_id: *dept_type,
                name: name.to_string(),
                parent_id: Some(parent.0),
                code: Some(code.to_string()),
                address: Some(format!("Agency Complex, {}", parent.1)),
                authorized_capital: None,
                business_activities: Some(desc.to_string()),
                contact_persons: None,
                description: Some(desc.to_string()),
                email: Some(format!("{}@gov.id", code.to_lowercase().replace("-", "."))),
                establishment_date: Some(chrono::NaiveDate::from_ymd_opt(2017, 1, 1).unwrap()),
                governance_structure: None,
                legal_status: None,
                paid_capital: None,
                path: Some(format!("/gov/mda/dept/{}", code.to_lowercase())),
                phone: None,
                registration_number: None,
                tax_number: None,
                website: None,
            };

            let new_org = Organization::new(org, system_user_id);
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            total_orgs += 1;
        }

        // ==================== EDUCATION (EDU) - National University ====================
        let edu_domain_id: DieselUlid = organization_domains::table
            .filter(organization_domains::code.eq("EDU"))
            .select(organization_domains::id)
            .first(&mut conn)?;

        let edu_type_map: HashMap<String, DieselUlid> = organization_types::table
            .filter(organization_types::domain_id.eq(edu_domain_id.to_string()))
            .select((organization_types::code, organization_types::id))
            .load::<(Option<String>, DieselUlid)>(&mut conn)?
            .into_iter()
            .filter_map(|(code, id)| code.map(|c| (c, id)))
            .collect();

        let univ_type = edu_type_map.get("UNIV").unwrap();
        let college_type = edu_type_map.get("COLL").unwrap();
        let edu_dept_type = edu_type_map.get("DEPT").unwrap();

        // University (Level 1)
        let university_org = CreateOrganization {
            domain_id: edu_domain_id,
            type_id: *univ_type,
            name: "National University".to_string(),
            parent_id: None,
            code: Some("NATUNIV".to_string()),
            address: Some("University Campus, Academic Way 456, City".to_string()),
            authorized_capital: None,
            business_activities: Some("Higher education, research, academic programs".to_string()),
            contact_persons: Some(serde_json::json!([
                {"name": "Prof. David Rector", "title": "Rector", "email": "david.rector@natuniv.edu", "phone": "+1-555-6001"}
            ])),
            description: Some("Premier national university for technology and science education".to_string()),
            email: Some("info@natuniv.edu".to_string()),
            establishment_date: Some(chrono::NaiveDate::from_ymd_opt(1950, 9, 1).unwrap()),
            governance_structure: Some(serde_json::json!({"structure": "University", "board": "Board of Trustees"})),
            legal_status: Some("Public University".to_string()),
            paid_capital: None,
            path: Some("/edu/natuniv".to_string()),
            phone: Some("+1-555-6000".to_string()),
            registration_number: Some("UNIV-001".to_string()),
            tax_number: None,
            website: Some("https://www.natuniv.edu".to_string()),
        };

        let new_org = Organization::new(university_org, system_user_id);
        let university_id = new_org.id;
        diesel::insert_into(organizations::table)
            .values(&new_org)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        total_orgs += 1;

        // Colleges (Level 2)
        let colleges = vec![
            ("College of Computer Science", "CCS", "Computer science education and research"),
            ("College of Engineering", "CE", "Engineering education and research"),
        ];

        let mut college_ids = Vec::new();
        for (name, code, desc) in colleges {
            let org = CreateOrganization {
                domain_id: edu_domain_id,
                type_id: *college_type,
                name: name.to_string(),
                parent_id: Some(university_id),
                code: Some(code.to_string()),
                address: Some("University Campus".to_string()),
                authorized_capital: None,
                business_activities: Some(desc.to_string()),
                contact_persons: None,
                description: Some(desc.to_string()),
                email: Some(format!("{}@natuniv.edu", code.to_lowercase())),
                establishment_date: Some(chrono::NaiveDate::from_ymd_opt(1960, 1, 1).unwrap()),
                governance_structure: None,
                legal_status: None,
                paid_capital: None,
                path: Some(format!("/edu/natuniv/{}", code.to_lowercase())),
                phone: None,
                registration_number: None,
                tax_number: None,
                website: None,
            };

            let new_org = Organization::new(org, system_user_id);
            let college_id = new_org.id;
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            college_ids.push((college_id, code));
            total_orgs += 1;
        }

        // Departments (Level 3)
        let edu_departments = vec![
            ("Computer Science Department", "CS-DEPT", "CS programs and research", 0),
            ("AI Research Lab", "AI-LAB", "AI and machine learning research", 0),
        ];

        for (name, code, desc, parent_idx) in edu_departments {
            let parent = college_ids[parent_idx];
            let org = CreateOrganization {
                domain_id: edu_domain_id,
                type_id: *edu_dept_type,
                name: name.to_string(),
                parent_id: Some(parent.0),
                code: Some(code.to_string()),
                address: Some("University Campus".to_string()),
                authorized_capital: None,
                business_activities: Some(desc.to_string()),
                contact_persons: None,
                description: Some(desc.to_string()),
                email: Some(format!("{}@natuniv.edu", code.to_lowercase().replace("-", "."))),
                establishment_date: Some(chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                governance_structure: None,
                legal_status: None,
                paid_capital: None,
                path: Some(format!("/edu/natuniv/dept/{}", code.to_lowercase())),
                phone: None,
                registration_number: None,
                tax_number: None,
                website: None,
            };

            let new_org = Organization::new(org, system_user_id);
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            total_orgs += 1;
        }

        // ==================== HEALTHCARE (HEA) - Regional Medical Center ====================
        let hea_domain_id: DieselUlid = organization_domains::table
            .filter(organization_domains::code.eq("HEA"))
            .select(organization_domains::id)
            .first(&mut conn)?;

        let hea_type_map: HashMap<String, DieselUlid> = organization_types::table
            .filter(organization_types::domain_id.eq(hea_domain_id.to_string()))
            .select((organization_types::code, organization_types::id))
            .load::<(Option<String>, DieselUlid)>(&mut conn)?
            .into_iter()
            .filter_map(|(code, id)| code.map(|c| (c, id)))
            .collect();

        let hosp_type = hea_type_map.get("HOSP").unwrap();
        let hea_dept_type = hea_type_map.get("DEPT").unwrap();
        let unit_type = hea_type_map.get("UNIT").unwrap();

        // Hospital (Level 1)
        let hospital_org = CreateOrganization {
            domain_id: hea_domain_id,
            type_id: *hosp_type,
            name: "Regional Medical Center".to_string(),
            parent_id: None,
            code: Some("RMC".to_string()),
            address: Some("789 Healthcare Drive, Medical District".to_string()),
            authorized_capital: None,
            business_activities: Some("Healthcare services, emergency care, surgery, diagnostics".to_string()),
            contact_persons: Some(serde_json::json!([
                {"name": "Dr. Elizabeth Director", "title": "Hospital Director", "email": "elizabeth.director@rmc.health", "phone": "+1-555-7001"}
            ])),
            description: Some("Comprehensive regional medical center providing advanced healthcare services".to_string()),
            email: Some("info@rmc.health".to_string()),
            establishment_date: Some(chrono::NaiveDate::from_ymd_opt(1985, 3, 15).unwrap()),
            governance_structure: Some(serde_json::json!({"structure": "Hospital", "board": "Hospital Board"})),
            legal_status: Some("Medical Institution".to_string()),
            paid_capital: None,
            path: Some("/health/rmc".to_string()),
            phone: Some("+1-555-7000".to_string()),
            registration_number: Some("HOSP-001".to_string()),
            tax_number: None,
            website: Some("https://www.rmc.health".to_string()),
        };

        let new_org = Organization::new(hospital_org, system_user_id);
        let hospital_id = new_org.id;
        diesel::insert_into(organizations::table)
            .values(&new_org)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        total_orgs += 1;

        // Departments (Level 2)
        let hospital_departments = vec![
            ("Emergency Department", "ER", "Emergency and trauma care"),
            ("Surgery Department", "SURG", "Surgical services"),
            ("Radiology Department", "RAD", "Imaging and diagnostics"),
        ];

        let mut hospital_dept_ids = Vec::new();
        for (name, code, desc) in hospital_departments {
            let org = CreateOrganization {
                domain_id: hea_domain_id,
                type_id: *hea_dept_type,
                name: name.to_string(),
                parent_id: Some(hospital_id),
                code: Some(code.to_string()),
                address: Some("Regional Medical Center".to_string()),
                authorized_capital: None,
                business_activities: Some(desc.to_string()),
                contact_persons: None,
                description: Some(desc.to_string()),
                email: Some(format!("{}@rmc.health", code.to_lowercase())),
                establishment_date: Some(chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
                governance_structure: None,
                legal_status: None,
                paid_capital: None,
                path: Some(format!("/health/rmc/{}", code.to_lowercase())),
                phone: None,
                registration_number: None,
                tax_number: None,
                website: None,
            };

            let new_org = Organization::new(org, system_user_id);
            let dept_id = new_org.id;
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            hospital_dept_ids.push((dept_id, code));
            total_orgs += 1;
        }

        // Units (Level 3)
        let hospital_units = vec![
            ("ICU", "ICU", "Intensive care unit", 0),
            ("Operating Room", "OR", "Surgical operating rooms", 1),
        ];

        for (name, code, desc, parent_idx) in hospital_units {
            let parent = hospital_dept_ids[parent_idx];
            let org = CreateOrganization {
                domain_id: hea_domain_id,
                type_id: *unit_type,
                name: name.to_string(),
                parent_id: Some(parent.0),
                code: Some(code.to_string()),
                address: Some("Regional Medical Center".to_string()),
                authorized_capital: None,
                business_activities: Some(desc.to_string()),
                contact_persons: None,
                description: Some(desc.to_string()),
                email: None,
                establishment_date: Some(chrono::NaiveDate::from_ymd_opt(1995, 1, 1).unwrap()),
                governance_structure: None,
                legal_status: None,
                paid_capital: None,
                path: Some(format!("/health/rmc/unit/{}", code.to_lowercase())),
                phone: None,
                registration_number: None,
                tax_number: None,
                website: None,
            };

            let new_org = Organization::new(org, system_user_id);
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            total_orgs += 1;
        }

        // ==================== NGO - Global Aid Foundation ====================
        let ngo_domain_id: DieselUlid = organization_domains::table
            .filter(organization_domains::code.eq("NGO"))
            .select(organization_domains::id)
            .first(&mut conn)?;

        let ngo_type_map: HashMap<String, DieselUlid> = organization_types::table
            .filter(organization_types::domain_id.eq(ngo_domain_id.to_string()))
            .select((organization_types::code, organization_types::id))
            .load::<(Option<String>, DieselUlid)>(&mut conn)?
            .into_iter()
            .filter_map(|(code, id)| code.map(|c| (c, id)))
            .collect();

        let foun_type = ngo_type_map.get("FOUN").unwrap();
        let prog_type = ngo_type_map.get("PROG").unwrap();
        let proj_type = ngo_type_map.get("PROJ").unwrap();

        // Foundation (Level 1)
        let foundation_org = CreateOrganization {
            domain_id: ngo_domain_id,
            type_id: *foun_type,
            name: "Global Aid Foundation".to_string(),
            parent_id: None,
            code: Some("GAF".to_string()),
            address: Some("Global House, London WC2N 5DU, UK".to_string()),
            authorized_capital: None,
            business_activities: Some("Humanitarian aid, development programs, disaster relief".to_string()),
            contact_persons: Some(serde_json::json!([
                {"name": "Catherine Executive Dir", "title": "Executive Director", "email": "catherine.ed@globalaid.org", "phone": "+44-20-8001"}
            ])),
            description: Some("International NGO providing humanitarian assistance and development programs worldwide".to_string()),
            email: Some("info@globalaid.org".to_string()),
            establishment_date: Some(chrono::NaiveDate::from_ymd_opt(2000, 6, 1).unwrap()),
            governance_structure: Some(serde_json::json!({"structure": "Foundation", "board": "Board of Trustees"})),
            legal_status: Some("Registered Charity".to_string()),
            paid_capital: None,
            path: Some("/ngo/gaf".to_string()),
            phone: Some("+44-20-8000".to_string()),
            registration_number: Some("NGO-001".to_string()),
            tax_number: None,
            website: Some("https://www.globalaid.org".to_string()),
        };

        let new_org = Organization::new(foundation_org, system_user_id);
        let foundation_id = new_org.id;
        diesel::insert_into(organizations::table)
            .values(&new_org)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        total_orgs += 1;

        // Programs (Level 2)
        let programs = vec![
            ("Health Program", "HEALTH", "Healthcare and medical assistance"),
            ("Education Program", "EDU", "Educational development"),
        ];

        let mut program_ids = Vec::new();
        for (name, code, desc) in programs {
            let org = CreateOrganization {
                domain_id: ngo_domain_id,
                type_id: *prog_type,
                name: name.to_string(),
                parent_id: Some(foundation_id),
                code: Some(code.to_string()),
                address: Some("Global House, London".to_string()),
                authorized_capital: None,
                business_activities: Some(desc.to_string()),
                contact_persons: None,
                description: Some(desc.to_string()),
                email: Some(format!("{}@globalaid.org", code.to_lowercase())),
                establishment_date: Some(chrono::NaiveDate::from_ymd_opt(2005, 1, 1).unwrap()),
                governance_structure: None,
                legal_status: None,
                paid_capital: None,
                path: Some(format!("/ngo/gaf/{}", code.to_lowercase())),
                phone: None,
                registration_number: None,
                tax_number: None,
                website: None,
            };

            let new_org = Organization::new(org, system_user_id);
            let prog_id = new_org.id;
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            program_ids.push((prog_id, code));
            total_orgs += 1;
        }

        // Projects (Level 3)
        let projects = vec![
            ("Africa Health Initiative", "AHI", "Health programs in Africa", 0),
            ("Rural Education Project", "REP", "Education in rural areas", 1),
        ];

        for (name, code, desc, parent_idx) in projects {
            let parent = program_ids[parent_idx];
            let org = CreateOrganization {
                domain_id: ngo_domain_id,
                type_id: *proj_type,
                name: name.to_string(),
                parent_id: Some(parent.0),
                code: Some(code.to_string()),
                address: Some("Field Office".to_string()),
                authorized_capital: None,
                business_activities: Some(desc.to_string()),
                contact_persons: None,
                description: Some(desc.to_string()),
                email: Some(format!("{}@globalaid.org", code.to_lowercase())),
                establishment_date: Some(chrono::NaiveDate::from_ymd_opt(2010, 1, 1).unwrap()),
                governance_structure: None,
                legal_status: None,
                paid_capital: None,
                path: Some(format!("/ngo/gaf/proj/{}", code.to_lowercase())),
                phone: None,
                registration_number: None,
                tax_number: None,
                website: None,
            };

            let new_org = Organization::new(org, system_user_id);
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            total_orgs += 1;
        }

        // ==================== MILITARY (MIL) - Defense Command ====================
        let mil_domain_id: DieselUlid = organization_domains::table
            .filter(organization_domains::code.eq("MIL"))
            .select(organization_domains::id)
            .first(&mut conn)?;

        let mil_type_map: HashMap<String, DieselUlid> = organization_types::table
            .filter(organization_types::domain_id.eq(mil_domain_id.to_string()))
            .select((organization_types::code, organization_types::id))
            .load::<(Option<String>, DieselUlid)>(&mut conn)?
            .into_iter()
            .filter_map(|(code, id)| code.map(|c| (c, id)))
            .collect();

        let command_type = mil_type_map.get("COMMAND").unwrap();
        let division_type = mil_type_map.get("DIVISION").unwrap();
        let battalion_type = mil_type_map.get("BATTALION").unwrap();

        // Command (Level 1)
        let command_org = CreateOrganization {
            domain_id: mil_domain_id,
            type_id: *command_type,
            name: "Cyber Defense Command".to_string(),
            parent_id: None,
            code: Some("CYBERCOM".to_string()),
            address: Some("Defense Complex, Fort Security".to_string()),
            authorized_capital: None,
            business_activities: Some("Cyber defense operations, military IT security, information warfare".to_string()),
            contact_persons: Some(serde_json::json!([
                {"name": "Gen. Robert Commander", "title": "Commander", "email": "robert.commander@defense.mil", "phone": "+1-555-9001"}
            ])),
            description: Some("Military cyber defense command responsible for protecting critical infrastructure".to_string()),
            email: Some("info@defense.mil".to_string()),
            establishment_date: Some(chrono::NaiveDate::from_ymd_opt(2010, 5, 1).unwrap()),
            governance_structure: Some(serde_json::json!({"structure": "Military Command", "chain_of_command": "Joint Chiefs of Staff"})),
            legal_status: Some("Military Organization".to_string()),
            paid_capital: None,
            path: Some("/mil/cybercom".to_string()),
            phone: Some("+1-555-9000".to_string()),
            registration_number: Some("MIL-001".to_string()),
            tax_number: None,
            website: None,
        };

        let new_org = Organization::new(command_org, system_user_id);
        let command_id = new_org.id;
        diesel::insert_into(organizations::table)
            .values(&new_org)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        total_orgs += 1;

        // Divisions (Level 2)
        let mil_divisions = vec![
            ("Operations Division", "OPS-DIV", "Cyber operations"),
            ("Intelligence Division", "INT-DIV", "Cyber intelligence"),
        ];

        let mut mil_div_ids = Vec::new();
        for (name, code, desc) in mil_divisions {
            let org = CreateOrganization {
                domain_id: mil_domain_id,
                type_id: *division_type,
                name: name.to_string(),
                parent_id: Some(command_id),
                code: Some(code.to_string()),
                address: Some("Defense Complex".to_string()),
                authorized_capital: None,
                business_activities: Some(desc.to_string()),
                contact_persons: None,
                description: Some(desc.to_string()),
                email: None,
                establishment_date: Some(chrono::NaiveDate::from_ymd_opt(2012, 1, 1).unwrap()),
                governance_structure: None,
                legal_status: None,
                paid_capital: None,
                path: Some(format!("/mil/cybercom/{}", code.to_lowercase())),
                phone: None,
                registration_number: None,
                tax_number: None,
                website: None,
            };

            let new_org = Organization::new(org, system_user_id);
            let div_id = new_org.id;
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            mil_div_ids.push((div_id, code));
            total_orgs += 1;
        }

        // Battalions (Level 3)
        let battalions = vec![
            ("Cyber Operations Battalion", "CYBER-OPS", "Cyber operations unit", 0),
        ];

        for (name, code, desc, parent_idx) in battalions {
            let parent = mil_div_ids[parent_idx];
            let org = CreateOrganization {
                domain_id: mil_domain_id,
                type_id: *battalion_type,
                name: name.to_string(),
                parent_id: Some(parent.0),
                code: Some(code.to_string()),
                address: Some("Defense Complex".to_string()),
                authorized_capital: None,
                business_activities: Some(desc.to_string()),
                contact_persons: None,
                description: Some(desc.to_string()),
                email: None,
                establishment_date: Some(chrono::NaiveDate::from_ymd_opt(2015, 1, 1).unwrap()),
                governance_structure: None,
                legal_status: None,
                paid_capital: None,
                path: Some(format!("/mil/cybercom/bn/{}", code.to_lowercase())),
                phone: None,
                registration_number: None,
                tax_number: None,
                website: None,
            };

            let new_org = Organization::new(org, system_user_id);
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            total_orgs += 1;
        }

        // ==================== RELIGIOUS (REL) - St. Joseph Diocese ====================
        let rel_domain_id: DieselUlid = organization_domains::table
            .filter(organization_domains::code.eq("REL"))
            .select(organization_domains::id)
            .first(&mut conn)?;

        let rel_type_map: HashMap<String, DieselUlid> = organization_types::table
            .filter(organization_types::domain_id.eq(rel_domain_id.to_string()))
            .select((organization_types::code, organization_types::id))
            .load::<(Option<String>, DieselUlid)>(&mut conn)?
            .into_iter()
            .filter_map(|(code, id)| code.map(|c| (c, id)))
            .collect();

        let dioc_type = rel_type_map.get("DIOC").unwrap();
        let parish_type = rel_type_map.get("PARISH").unwrap();
        let comm_type = rel_type_map.get("COMM").unwrap();

        // Diocese (Level 1)
        let diocese_org = CreateOrganization {
            domain_id: rel_domain_id,
            type_id: *dioc_type,
            name: "Diocese of St. Joseph".to_string(),
            parent_id: None,
            code: Some("STJ-DIOC".to_string()),
            address: Some("Cathedral Square, 123 Faith Avenue".to_string()),
            authorized_capital: None,
            business_activities: Some("Religious services, education, charity, community outreach".to_string()),
            contact_persons: Some(serde_json::json!([
                {"name": "Bishop Francis Leader", "title": "Bishop", "email": "francis.bishop@stjoseph.church", "phone": "+1-555-5001"}
            ])),
            description: Some("Catholic diocese providing religious services and community support".to_string()),
            email: Some("info@stjoseph.church".to_string()),
            establishment_date: Some(chrono::NaiveDate::from_ymd_opt(1920, 12, 25).unwrap()),
            governance_structure: Some(serde_json::json!({"structure": "Diocese", "hierarchy": "Catholic Church"})),
            legal_status: Some("Religious Organization".to_string()),
            paid_capital: None,
            path: Some("/rel/stjoseph".to_string()),
            phone: Some("+1-555-5000".to_string()),
            registration_number: Some("REL-001".to_string()),
            tax_number: None,
            website: Some("https://www.stjoseph.church".to_string()),
        };

        let new_org = Organization::new(diocese_org, system_user_id);
        let diocese_id = new_org.id;
        diesel::insert_into(organizations::table)
            .values(&new_org)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        total_orgs += 1;

        // Parishes (Level 2)
        let parishes = vec![
            ("St. Mary Parish", "STM", "Parish services and community"),
            ("St. Peter Parish", "STP", "Parish services and missions"),
        ];

        let mut parish_ids = Vec::new();
        for (name, code, desc) in parishes {
            let org = CreateOrganization {
                domain_id: rel_domain_id,
                type_id: *parish_type,
                name: name.to_string(),
                parent_id: Some(diocese_id),
                code: Some(code.to_string()),
                address: Some("Parish Center".to_string()),
                authorized_capital: None,
                business_activities: Some(desc.to_string()),
                contact_persons: None,
                description: Some(desc.to_string()),
                email: Some(format!("{}@stjoseph.church", code.to_lowercase())),
                establishment_date: Some(chrono::NaiveDate::from_ymd_opt(1950, 1, 1).unwrap()),
                governance_structure: None,
                legal_status: None,
                paid_capital: None,
                path: Some(format!("/rel/stjoseph/{}", code.to_lowercase())),
                phone: None,
                registration_number: None,
                tax_number: None,
                website: None,
            };

            let new_org = Organization::new(org, system_user_id);
            let parish_id = new_org.id;
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            parish_ids.push((parish_id, code));
            total_orgs += 1;
        }

        // Communities (Level 3)
        let communities = vec![
            ("Youth Ministry", "YOUTH", "Youth programs and activities", 0),
            ("Education Ministry", "EDU-MIN", "Religious education", 0),
        ];

        for (name, code, desc, parent_idx) in communities {
            let parent = parish_ids[parent_idx];
            let org = CreateOrganization {
                domain_id: rel_domain_id,
                type_id: *comm_type,
                name: name.to_string(),
                parent_id: Some(parent.0),
                code: Some(code.to_string()),
                address: Some("Parish Center".to_string()),
                authorized_capital: None,
                business_activities: Some(desc.to_string()),
                contact_persons: None,
                description: Some(desc.to_string()),
                email: None,
                establishment_date: Some(chrono::NaiveDate::from_ymd_opt(1980, 1, 1).unwrap()),
                governance_structure: None,
                legal_status: None,
                paid_capital: None,
                path: Some(format!("/rel/stjoseph/comm/{}", code.to_lowercase())),
                phone: None,
                registration_number: None,
                tax_number: None,
                website: None,
            };

            let new_org = Organization::new(org, system_user_id);
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
            total_orgs += 1;
        }

        println!("✅ {} Organizations seeded successfully!", total_orgs);
        Ok(())
    }
}
