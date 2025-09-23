use crate::database::seeder::Seeder;
use crate::database::DbPool;
use anyhow::Result;
use crate::app::models::{DieselUlid, organization::{NewOrganization, CreateOrganization}};
use diesel::prelude::*;
use crate::schema::organizations;

pub struct OrganizationSeeder;

impl Seeder for OrganizationSeeder {
    fn class_name(&self) -> &'static str {
        "OrganizationSeeder"
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("🌱 Seeding organizations...");
        let mut conn = pool.get()?;

        // Create main holding company first
        let holding_org = CreateOrganization {
            name: "TechCorp Holdings".to_string(),
            organization_type: "company".to_string(),
            parent_id: None,
            code: Some("TECH-HOLD".to_string()),
            level: Some(0),
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

        let holding_id = {
            let new_org = NewOrganization::new(holding_org, None);
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .returning(organizations::id)
                .get_result::<DieselUlid>(&mut conn)
                .unwrap_or(new_org.id)
        };

        // Create subsidiaries and departments
        let organizations_data = vec![
            // Level 1: Subsidiaries
            ("TechCorp Software", "company", Some(holding_id), Some("TECH-SOFT"), 1, "Software development subsidiary"),
            ("TechCorp Consulting", "company", Some(holding_id), Some("TECH-CONS"), 1, "Technology consulting subsidiary"),
            ("TechCorp Cloud", "company", Some(holding_id), Some("TECH-CLOUD"), 1, "Cloud services subsidiary"),

            // Level 2: Divisions
            ("Engineering Division", "division", Some(holding_id), Some("ENG-DIV"), 2, "Software engineering division"),
            ("Product Division", "division", Some(holding_id), Some("PROD-DIV"), 2, "Product development division"),
            ("Operations Division", "division", Some(holding_id), Some("OPS-DIV"), 2, "Operations and infrastructure"),
            ("Sales Division", "division", Some(holding_id), Some("SALES-DIV"), 2, "Sales and business development"),

            // Level 3: Departments
            ("Backend Development", "department", Some(holding_id), Some("BACK-DEV"), 3, "Backend systems development"),
            ("Frontend Development", "department", Some(holding_id), Some("FRONT-DEV"), 3, "Frontend and UI development"),
            ("Mobile Development", "department", Some(holding_id), Some("MOBILE-DEV"), 3, "Mobile application development"),
            ("Quality Assurance", "department", Some(holding_id), Some("QA-DEPT"), 3, "Software testing and quality"),
            ("DevOps & Infrastructure", "department", Some(holding_id), Some("DEVOPS"), 3, "DevOps and infrastructure"),
            ("Product Management", "department", Some(holding_id), Some("PROD-MGT"), 3, "Product strategy and management"),
            ("User Experience", "department", Some(holding_id), Some("UX-DEPT"), 3, "User experience and design"),
            ("Data Engineering", "department", Some(holding_id), Some("DATA-ENG"), 3, "Data platform and analytics"),
            ("Security", "department", Some(holding_id), Some("SEC-DEPT"), 3, "Information security"),
            ("IT Support", "department", Some(holding_id), Some("IT-SUPP"), 3, "Internal IT support"),

            // Level 4: Branches
            ("API Development", "branch", Some(holding_id), Some("API-BR"), 4, "REST API development"),
            ("Microservices", "branch", Some(holding_id), Some("MICRO-BR"), 4, "Microservices architecture"),
            ("Web Applications", "branch", Some(holding_id), Some("WEB-BR"), 4, "Web application development"),
            ("iOS Development", "branch", Some(holding_id), Some("IOS-BR"), 4, "iOS application development"),
            ("Android Development", "branch", Some(holding_id), Some("ANDROID-BR"), 4, "Android application development"),
        ];

        for (i, (name, org_type, parent_id, code, level, description)) in organizations_data.iter().enumerate() {
            let establishment_date = Some(chrono::NaiveDate::from_ymd_opt(2015, 1, 1).unwrap() + chrono::Duration::days(i as i64 * 30));
            let contact_email = format!("{}@techcorp.com", code.unwrap_or("info"));

            let contact_persons = Some(serde_json::json!([
                {"name": format!("{} Manager", name), "title": "Department Manager", "email": contact_email, "phone": format!("+1-555-{:04}", 200 + i)}
            ]));

            let governance_structure = match *org_type {
                "company" => Some(serde_json::json!({"structure": "Subsidiary", "reporting_to": "Board of Directors"})),
                "division" => Some(serde_json::json!({"structure": "Division", "reporting_to": "VP Level"})),
                "department" => Some(serde_json::json!({"structure": "Department", "reporting_to": "Director Level"})),
                "branch" => Some(serde_json::json!({"structure": "Branch", "reporting_to": "Manager Level"})),
                _ => None,
            };

            let (authorized_capital, paid_capital) = match *org_type {
                "company" => (Some(crate::app::models::DecimalWrapper::from(10000000)), Some(crate::app::models::DecimalWrapper::from(5000000))),
                _ => (None, None),
            };

            let org = CreateOrganization {
                name: name.to_string(),
                organization_type: org_type.to_string(),
                parent_id: *parent_id,
                code: code.map(|c| c.to_string()),
                level: Some(*level),
                address: Some(format!("Tech Campus, Innovation Way, Floor {}, Suite {}", level, i + 100)),
                authorized_capital,
                business_activities: Some(format!("{} operations and related activities", description)),
                contact_persons,
                description: Some(description.to_string()),
                email: Some(contact_email),
                establishment_date,
                governance_structure,
                legal_status: if *org_type == "company" { Some("Subsidiary".to_string()) } else { None },
                paid_capital,
                path: Some(format!("/techcorp/{}", code.unwrap_or("org"))),
                phone: Some(format!("+1-555-{:04}", 100 + i)),
                registration_number: if *org_type == "company" { Some(format!("REG-{:03}", i + 2)) } else { None },
                tax_number: if *org_type == "company" { Some(format!("TAX-{:03}", i + 2)) } else { None },
                website: if *org_type == "company" { Some(format!("https://{}.techcorp.com", code.unwrap_or("info"))) } else { None },
            };

            let new_org = NewOrganization::new(org, None);
            diesel::insert_into(organizations::table)
                .values(&new_org)
                .on_conflict_do_nothing()
                .execute(&mut conn)?;
        }

        println!("✅ 25 Organizations seeded successfully!");
        Ok(())
    }
}