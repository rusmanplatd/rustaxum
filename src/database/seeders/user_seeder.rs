use crate::database::DbPool;
use anyhow::Result;
use ulid::Ulid;
use chrono::{Utc};
use crate::database::seeder::Seeder;
use crate::app::services::auth_service::AuthService;
use diesel::prelude::*;
use crate::schema::sys_users;

pub struct UserSeeder;

impl Seeder for UserSeeder {
    fn class_name(&self) -> &'static str {
        "UserSeeder"
    }

    fn description(&self) -> Option<&'static str> {
        Some("Seed default sys_users for the application")
    }

    fn run(&self, pool: &DbPool) -> Result<()> {
        println!("🌱 Seeding sys_users...");

        let mut conn = pool.get()?;
        let now = Utc::now().naive_utc();
        let password = AuthService::hash_password("password")?;

        // Enhanced user data with more diversity and realistic profiles
        let users = vec![
            // Executive Leadership
            ("Jane CEO", "jane.ceo@techcorp.com", Some("jceo"), true, Some("1975-03-15"), Some("+1-555-1001"), Some("America/New_York"), Some("en-US")),
            ("John CFO", "john.cfo@techcorp.com", Some("jcfo"), true, Some("1978-07-22"), Some("+1-555-1002"), Some("America/New_York"), Some("en-US")),
            ("Mary CTO", "mary.cto@techcorp.com", Some("mcto"), true, Some("1980-11-10"), Some("+1-555-1003"), Some("America/New_York"), Some("en-US")),
            ("Bob COO", "bob.coo@techcorp.com", Some("bcoo"), true, Some("1976-05-18"), Some("+1-555-1004"), Some("America/New_York"), Some("en-US")),

            // Senior Management
            ("Sarah VP Engineering", "sarah.vp@techcorp.com", Some("svp"), true, Some("1982-09-05"), Some("+1-555-2001"), Some("America/Chicago"), Some("en-US")),
            ("Michael VP Product", "michael.vp@techcorp.com", Some("mvp"), true, Some("1983-04-12"), Some("+1-555-2002"), Some("America/Chicago"), Some("en-US")),
            ("Emily VP Operations", "emily.vp@techcorp.com", Some("evp"), true, Some("1981-12-28"), Some("+1-555-2003"), Some("America/Denver"), Some("en-US")),
            ("David VP Sales", "david.vp@techcorp.com", Some("dvp"), true, Some("1979-08-14"), Some("+1-555-2004"), Some("America/Los_Angeles"), Some("en-US")),

            // Engineering Directors
            ("Lisa Backend Director", "lisa.backend@techcorp.com", Some("lbackend"), true, Some("1985-02-20"), Some("+1-555-3001"), Some("America/New_York"), Some("en-US")),
            ("Robert Frontend Director", "robert.frontend@techcorp.com", Some("rfrontend"), true, Some("1986-06-15"), Some("+1-555-3002"), Some("America/Chicago"), Some("en-US")),
            ("Jennifer Mobile Director", "jennifer.mobile@techcorp.com", Some("jmobile"), true, Some("1987-10-08"), Some("+1-555-3003"), Some("America/Los_Angeles"), Some("en-US")),
            ("William QA Director", "william.qa@techcorp.com", Some("wqa"), true, Some("1984-03-25"), Some("+1-555-3004"), Some("America/Denver"), Some("en-US")),
            ("Jessica DevOps Director", "jessica.devops@techcorp.com", Some("jdevops"), true, Some("1988-11-30"), Some("+1-555-3005"), Some("America/Phoenix"), Some("en-US")),

            // Product & Design
            ("Christopher Product Manager", "christopher.pm@techcorp.com", Some("cpm"), true, Some("1989-01-17"), Some("+1-555-4001"), Some("America/New_York"), Some("en-US")),
            ("Amanda UX Director", "amanda.ux@techcorp.com", Some("aux"), true, Some("1990-05-22"), Some("+1-555-4002"), Some("America/Los_Angeles"), Some("en-US")),
            ("Matthew Designer", "matthew.design@techcorp.com", Some("mdesign"), true, Some("1991-09-12"), Some("+1-555-4003"), Some("America/Chicago"), Some("en-US")),

            // Senior Engineers
            ("Ashley Senior Backend", "ashley.backend@techcorp.com", Some("abackend"), true, Some("1988-07-08"), Some("+1-555-5001"), Some("America/New_York"), Some("en-US")),
            ("Daniel Senior Frontend", "daniel.frontend@techcorp.com", Some("dfrontend"), true, Some("1989-12-14"), Some("+1-555-5002"), Some("America/Chicago"), Some("en-US")),
            ("Stephanie Senior Mobile", "stephanie.mobile@techcorp.com", Some("smobile"), true, Some("1990-04-19"), Some("+1-555-5003"), Some("America/Los_Angeles"), Some("en-US")),
            ("Joshua Senior DevOps", "joshua.devops@techcorp.com", Some("jdevops"), true, Some("1987-08-23"), Some("+1-555-5004"), Some("America/Denver"), Some("en-US")),

            // Mid-Level Engineers
            ("Michelle Backend Dev", "michelle.backend@techcorp.com", Some("mbackend"), true, Some("1992-02-11"), Some("+1-555-6001"), Some("America/New_York"), Some("en-US")),
            ("Andrew Frontend Dev", "andrew.frontend@techcorp.com", Some("afrontend"), true, Some("1993-06-27"), Some("+1-555-6002"), Some("America/Chicago"), Some("en-US")),
            ("Elizabeth iOS Dev", "elizabeth.ios@techcorp.com", Some("eios"), true, Some("1991-10-15"), Some("+1-555-6003"), Some("America/Los_Angeles"), Some("en-US")),
            ("Ryan Android Dev", "ryan.android@techcorp.com", Some("randroid"), true, Some("1992-03-08"), Some("+1-555-6004"), Some("America/Denver"), Some("en-US")),
            ("Nicole QA Engineer", "nicole.qa@techcorp.com", Some("nqa"), true, Some("1993-11-22"), Some("+1-555-6005"), Some("America/Phoenix"), Some("en-US")),

            // Junior Engineers
            ("Brandon Junior Backend", "brandon.junior@techcorp.com", Some("bjunior"), true, Some("1995-01-14"), Some("+1-555-7001"), Some("America/New_York"), Some("en-US")),
            ("Samantha Junior Frontend", "samantha.junior@techcorp.com", Some("sjunior"), true, Some("1996-05-09"), Some("+1-555-7002"), Some("America/Chicago"), Some("en-US")),
            ("Kevin Junior Mobile", "kevin.junior@techcorp.com", Some("kjunior"), true, Some("1994-09-18"), Some("+1-555-7003"), Some("America/Los_Angeles"), Some("en-US")),
            ("Rachel Junior QA", "rachel.junior@techcorp.com", Some("rjunior"), true, Some("1997-02-26"), Some("+1-555-7004"), Some("America/Denver"), Some("en-US")),
            ("Tyler Junior DevOps", "tyler.junior@techcorp.com", Some("tjunior"), true, Some("1995-07-03"), Some("+1-555-7005"), Some("America/Phoenix"), Some("en-US")),

            // Interns
            ("Madison Intern Backend", "madison.intern@techcorp.com", Some("mintern"), true, Some("1999-03-21"), Some("+1-555-8001"), Some("America/New_York"), Some("en-US")),
            ("Jordan Intern Frontend", "jordan.intern@techcorp.com", Some("jintern"), true, Some("2000-08-16"), Some("+1-555-8002"), Some("America/Chicago"), Some("en-US")),
            ("Taylor Intern Mobile", "taylor.intern@techcorp.com", Some("tintern"), true, Some("1998-12-05"), Some("+1-555-8003"), Some("America/Los_Angeles"), Some("en-US")),
            ("Alex Intern QA", "alex.intern@techcorp.com", Some("aintern"), true, Some("2001-04-11"), Some("+1-555-8004"), Some("America/Denver"), Some("en-US")),
            ("Morgan Intern DevOps", "morgan.intern@techcorp.com", Some("mointern"), true, Some("1999-10-28"), Some("+1-555-8005"), Some("America/Phoenix"), Some("en-US")),

            // Sales & Marketing
            ("Brian Sales Manager", "brian.sales@techcorp.com", Some("bsales"), true, Some("1986-04-07"), Some("+1-555-9001"), Some("America/New_York"), Some("en-US")),
            ("Melissa Marketing Manager", "melissa.marketing@techcorp.com", Some("mmarketing"), true, Some("1987-11-19"), Some("+1-555-9002"), Some("America/Los_Angeles"), Some("en-US")),
            ("Patrick Account Manager", "patrick.account@techcorp.com", Some("paccount"), true, Some("1988-06-24"), Some("+1-555-9003"), Some("America/Chicago"), Some("en-US")),
            ("Rebecca Business Dev", "rebecca.bizdev@techcorp.com", Some("rbizdev"), true, Some("1989-02-13"), Some("+1-555-9004"), Some("America/Denver"), Some("en-US")),

            // Operations & Support
            ("Steven IT Support", "steven.it@techcorp.com", Some("sit"), true, Some("1990-08-30"), Some("+1-555-9101"), Some("America/Phoenix"), Some("en-US")),
            ("Christina Network Admin", "christina.network@techcorp.com", Some("cnetwork"), true, Some("1991-12-17"), Some("+1-555-9102"), Some("America/New_York"), Some("en-US")),
            ("James Security Analyst", "james.security@techcorp.com", Some("jsecurity"), true, Some("1988-05-06"), Some("+1-555-9103"), Some("America/Chicago"), Some("en-US")),
            ("Kimberly Data Analyst", "kimberly.data@techcorp.com", Some("kdata"), true, Some("1992-09-21"), Some("+1-555-9104"), Some("America/Los_Angeles"), Some("en-US")),

            // Consultants
            ("Thomas Enterprise Consultant", "thomas.consultant@techcorp.com", Some("tconsultant"), true, Some("1983-07-12"), Some("+1-555-9201"), Some("America/Denver"), Some("en-US")),
            ("Angela Technical Consultant", "angela.tech@techcorp.com", Some("atech"), true, Some("1985-11-27"), Some("+1-555-9202"), Some("America/Phoenix"), Some("en-US")),

            // Cloud & Infrastructure
            ("Richard Cloud Architect", "richard.cloud@techcorp.com", Some("rcloud"), true, Some("1984-03-18"), Some("+1-555-9301"), Some("America/New_York"), Some("en-US")),
            ("Karen SRE Lead", "karen.sre@techcorp.com", Some("ksre"), true, Some("1986-09-04"), Some("+1-555-9302"), Some("America/Chicago"), Some("en-US")),
            ("Charles Platform Engineer", "charles.platform@techcorp.com", Some("cplatform"), true, Some("1989-01-29"), Some("+1-555-9303"), Some("America/Los_Angeles"), Some("en-US")),

            // Security Team
            ("Nancy SOC Manager", "nancy.soc@techcorp.com", Some("nsoc"), true, Some("1987-06-10"), Some("+1-555-9401"), Some("America/Denver"), Some("en-US")),
            ("Jason Incident Response", "jason.incident@techcorp.com", Some("jincident"), true, Some("1990-10-23"), Some("+1-555-9402"), Some("America/Phoenix"), Some("en-US")),
            ("Laura Threat Analyst", "laura.threat@techcorp.com", Some("lthreat"), true, Some("1991-04-16"), Some("+1-555-9403"), Some("America/New_York"), Some("en-US")),
        ];

        for (i, (name, email, username, verified, birthdate, phone, zoneinfo, locale)) in users.iter().enumerate() {
            let user_id = Ulid::new().to_string();
            let birthdate_parsed = birthdate.as_ref().map(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").unwrap());
            let phone_verified = if *verified && phone.is_some() { Some(now) } else { None };
            let last_login = if *verified { Some(now - chrono::Duration::days(i as i64 % 30)) } else { None };

            diesel::insert_into(sys_users::table)
                .values((
                    sys_users::id.eq(&user_id),
                    sys_users::name.eq(*name),
                    sys_users::email.eq(*email),
                    sys_users::username.eq(*username),
                    sys_users::password.eq(&password),
                    sys_users::email_verified_at.eq(if *verified { Some(now) } else { None }),
                    sys_users::phone_number.eq(*phone),
                    sys_users::phone_verified_at.eq(phone_verified),
                    sys_users::birthdate.eq(birthdate_parsed),
                    sys_users::zoneinfo.eq(*zoneinfo),
                    sys_users::locale.eq(*locale),
                    sys_users::last_login_at.eq(last_login),
                    sys_users::last_seen_at.eq(now - chrono::Duration::hours(i as i64 % 24)),
                    sys_users::failed_login_attempts.eq(0),
                    sys_users::avatar.eq(Some(format!("https://ui-avatars.com/api/?name={}&background=random", name.replace(" ", "+")))),
                    sys_users::google_id.eq::<Option<String>>(None),
                    sys_users::remember_token.eq::<Option<String>>(None),
                    sys_users::password_reset_token.eq::<Option<String>>(None),
                    sys_users::password_reset_expires_at.eq::<Option<chrono::DateTime<Utc>>>(None),
                    sys_users::refresh_token.eq::<Option<String>>(None),
                    sys_users::refresh_token_expires_at.eq::<Option<chrono::DateTime<Utc>>>(None),
                    sys_users::locked_until.eq::<Option<chrono::DateTime<Utc>>>(None),
                    sys_users::created_at.eq(now),
                    sys_users::updated_at.eq(now),
                    sys_users::deleted_at.eq::<Option<chrono::DateTime<Utc>>>(None),
                    sys_users::created_by_id.eq::<Option<String>>(None),
                    sys_users::updated_by_id.eq::<Option<String>>(None),
                    sys_users::deleted_by_id.eq::<Option<String>>(None),
                ))
                .on_conflict(sys_users::email)
                .do_nothing()
                .execute(&mut conn)?;
        }

        println!("✅ {} Users seeded successfully!", users.len());
        Ok(())
    }
}
