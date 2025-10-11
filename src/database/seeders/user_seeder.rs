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

        // First, create the system user that will be used as the creator for all other users
        // Generate a proper ULID for the system user
        let system_user_id = Ulid::new().to_string();

        // Insert system user first with self-referencing audit fields
        diesel::insert_into(sys_users::table)
            .values((
                sys_users::id.eq(&system_user_id),
                sys_users::name.eq("System"),
                sys_users::email.eq("system@seeder.internal"),
                sys_users::username.eq(Some("system")),
                sys_users::password.eq(&password),
                sys_users::email_verified_at.eq(Some(now)),
                sys_users::phone_number.eq::<Option<&str>>(None),
                sys_users::phone_verified_at.eq::<Option<chrono::NaiveDateTime>>(None),
                sys_users::birthdate.eq::<Option<chrono::NaiveDate>>(None),
                sys_users::zoneinfo.eq(Some("UTC")),
                sys_users::locale.eq(Some("en-US")),
                sys_users::last_login_at.eq(Some(now)),
                sys_users::last_seen_at.eq(now),
                sys_users::failed_login_attempts.eq(0),
                sys_users::avatar.eq::<Option<String>>(None),
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
                sys_users::created_by_id.eq(&system_user_id),
                sys_users::updated_by_id.eq(&system_user_id),
                sys_users::deleted_by_id.eq::<Option<String>>(None),
            ))
            .on_conflict(sys_users::email)
            .do_nothing()
            .execute(&mut conn)?;

        println!("   ✓ Created system user: {}", system_user_id);

        // Create register test user
        let register_user_id = Ulid::new().to_string();
        diesel::insert_into(sys_users::table)
            .values((
                sys_users::id.eq(&register_user_id),
                sys_users::name.eq("Register Test"),
                sys_users::email.eq("register@seeder.internal"),
                sys_users::username.eq(Some("register")),
                sys_users::password.eq(&password),
                sys_users::email_verified_at.eq(Some(now)),
                sys_users::phone_number.eq::<Option<&str>>(None),
                sys_users::phone_verified_at.eq::<Option<chrono::NaiveDateTime>>(None),
                sys_users::birthdate.eq::<Option<chrono::NaiveDate>>(None),
                sys_users::zoneinfo.eq(Some("UTC")),
                sys_users::locale.eq(Some("en-US")),
                sys_users::last_login_at.eq(Some(now)),
                sys_users::last_seen_at.eq(now),
                sys_users::failed_login_attempts.eq(0),
                sys_users::avatar.eq::<Option<String>>(None),
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
                sys_users::created_by_id.eq(&system_user_id),
                sys_users::updated_by_id.eq(&system_user_id),
                sys_users::deleted_by_id.eq::<Option<String>>(None),
            ))
            .on_conflict(sys_users::email)
            .do_nothing()
            .execute(&mut conn)?;

        println!("   ✓ Created register test user: {}", register_user_id);

        // Alter table to make created_by_id and updated_by_id NOT NULL
        diesel::sql_query("ALTER TABLE sys_users ALTER COLUMN created_by_id SET NOT NULL")
            .execute(&mut conn)?;
        diesel::sql_query("ALTER TABLE sys_users ALTER COLUMN updated_by_id SET NOT NULL")
            .execute(&mut conn)?;

        println!("   ✓ Altered sys_users table: created_by_id, updated_by_id set to NOT NULL");

        // Enhanced user data with more diversity and realistic profiles
        let users = vec![
            // ==================== PRIVATE SECTOR (PVT) - TechCorp ====================
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
            ("Jessica DevOps Director", "jessica.devops@techcorp.com", Some("jessdevops"), true, Some("1988-11-30"), Some("+1-555-3005"), Some("America/Phoenix"), Some("en-US")),

            // Product & Design
            ("Christopher Product Manager", "christopher.pm@techcorp.com", Some("cpm"), true, Some("1989-01-17"), Some("+1-555-4001"), Some("America/New_York"), Some("en-US")),
            ("Amanda UX Director", "amanda.ux@techcorp.com", Some("aux"), true, Some("1990-05-22"), Some("+1-555-4002"), Some("America/Los_Angeles"), Some("en-US")),
            ("Matthew Designer", "matthew.design@techcorp.com", Some("mdesign"), true, Some("1991-09-12"), Some("+1-555-4003"), Some("America/Chicago"), Some("en-US")),

            // Senior Engineers
            ("Ashley Senior Backend", "ashley.backend@techcorp.com", Some("abackend"), true, Some("1988-07-08"), Some("+1-555-5001"), Some("America/New_York"), Some("en-US")),
            ("Daniel Senior Frontend", "daniel.frontend@techcorp.com", Some("dfrontend"), true, Some("1989-12-14"), Some("+1-555-5002"), Some("America/Chicago"), Some("en-US")),
            ("Stephanie Senior Mobile", "stephanie.mobile@techcorp.com", Some("smobile"), true, Some("1990-04-19"), Some("+1-555-5003"), Some("America/Los_Angeles"), Some("en-US")),
            ("Joshua Senior DevOps", "joshua.devops@techcorp.com", Some("joshuadevops"), true, Some("1987-08-23"), Some("+1-555-5004"), Some("America/Denver"), Some("en-US")),

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

            // ==================== GOVERNMENT (GOV) - Ministry of Digital Affairs ====================
            // Ministry Leadership
            ("Dr. Sarah Minister", "sarah.minister@gov.id", Some("sminister"), true, Some("1970-03-20"), Some("+62-21-5001"), Some("Asia/Jakarta"), Some("id-ID")),
            ("Ahmad Secretary General", "ahmad.secgen@gov.id", Some("asecgen"), true, Some("1972-08-15"), Some("+62-21-5002"), Some("Asia/Jakarta"), Some("id-ID")),
            ("Dewi Director General", "dewi.dirgen@gov.id", Some("ddirgen"), true, Some("1975-11-28"), Some("+62-21-5003"), Some("Asia/Jakarta"), Some("id-ID")),

            // Agency Directors
            ("Budi IT Director", "budi.it@gov.id", Some("bitdir"), true, Some("1978-05-12"), Some("+62-21-5101"), Some("Asia/Jakarta"), Some("id-ID")),
            ("Siti Data Director", "siti.data@gov.id", Some("sdatadir"), true, Some("1980-09-22"), Some("+62-21-5102"), Some("Asia/Jakarta"), Some("id-ID")),
            ("Rudi Cyber Director", "rudi.cyber@gov.id", Some("rcyberdir"), true, Some("1979-12-08"), Some("+62-21-5103"), Some("Asia/Jakarta"), Some("id-ID")),

            // Department Heads
            ("Rina Infrastructure Head", "rina.infra@gov.id", Some("rinfra"), true, Some("1982-04-15"), Some("+62-21-5201"), Some("Asia/Jakarta"), Some("id-ID")),
            ("Agus Security Head", "agus.security@gov.id", Some("asecurity"), true, Some("1983-07-20"), Some("+62-21-5202"), Some("Asia/Jakarta"), Some("id-ID")),
            ("Fitri Digital Services", "fitri.services@gov.id", Some("fservices"), true, Some("1985-10-11"), Some("+62-21-5203"), Some("Asia/Jakarta"), Some("id-ID")),

            // Staff & Analysts
            ("Hendra Policy Analyst", "hendra.policy@gov.id", Some("hpolicy"), true, Some("1988-02-18"), Some("+62-21-5301"), Some("Asia/Jakarta"), Some("id-ID")),
            ("Maya Data Analyst", "maya.analyst@gov.id", Some("manalyst"), true, Some("1990-06-25"), Some("+62-21-5302"), Some("Asia/Jakarta"), Some("id-ID")),
            ("Doni IT Support", "doni.support@gov.id", Some("dsupport"), true, Some("1992-11-30"), Some("+62-21-5303"), Some("Asia/Jakarta"), Some("id-ID")),

            // ==================== EDUCATION (EDU) - National University ====================
            // University Leadership
            ("Prof. David Rector", "david.rector@natuniv.edu", Some("drector"), true, Some("1965-04-10"), Some("+1-555-6001"), Some("America/New_York"), Some("en-US")),
            ("Dr. Lisa Vice Rector", "lisa.vice@natuniv.edu", Some("lvice"), true, Some("1970-08-22"), Some("+1-555-6002"), Some("America/New_York"), Some("en-US")),
            ("Prof. Michael Dean CS", "michael.dean@natuniv.edu", Some("mdean"), true, Some("1972-12-15"), Some("+1-555-6003"), Some("America/New_York"), Some("en-US")),

            // Faculty Professors
            ("Prof. Jennifer AI", "jennifer.ai@natuniv.edu", Some("jenai"), true, Some("1975-05-18"), Some("+1-555-6101"), Some("America/New_York"), Some("en-US")),
            ("Dr. Robert Networks", "robert.networks@natuniv.edu", Some("rnetworks"), true, Some("1978-09-12"), Some("+1-555-6102"), Some("America/New_York"), Some("en-US")),
            ("Dr. Patricia Database", "patricia.db@natuniv.edu", Some("pdb"), true, Some("1980-01-25"), Some("+1-555-6103"), Some("America/New_York"), Some("en-US")),

            // Research Staff
            ("Dr. William Research", "william.research@natuniv.edu", Some("wresearch"), true, Some("1982-07-08"), Some("+1-555-6201"), Some("America/New_York"), Some("en-US")),
            ("Maria Lab Manager", "maria.lab@natuniv.edu", Some("mlab"), true, Some("1985-11-14"), Some("+1-555-6202"), Some("America/New_York"), Some("en-US")),
            ("Thomas Researcher", "thomas.research@natuniv.edu", Some("tresearch"), true, Some("1987-03-22"), Some("+1-555-6203"), Some("America/New_York"), Some("en-US")),

            // Administrative Staff
            ("Susan Registrar", "susan.registrar@natuniv.edu", Some("sregistrar"), true, Some("1980-06-17"), Some("+1-555-6301"), Some("America/New_York"), Some("en-US")),
            ("George IT Admin", "george.admin@natuniv.edu", Some("gadmin"), true, Some("1985-10-29"), Some("+1-555-6302"), Some("America/New_York"), Some("en-US")),

            // ==================== HEALTHCARE (HEA) - Regional Medical Center ====================
            // Hospital Leadership
            ("Dr. Elizabeth Director", "elizabeth.director@rmc.health", Some("edirector"), true, Some("1968-03-12"), Some("+1-555-7001"), Some("America/Chicago"), Some("en-US")),
            ("Dr. Richard CMO", "richard.cmo@rmc.health", Some("rcmo"), true, Some("1970-07-25"), Some("+1-555-7002"), Some("America/Chicago"), Some("en-US")),
            ("Margaret CNO", "margaret.cno@rmc.health", Some("mcno"), true, Some("1972-11-08"), Some("+1-555-7003"), Some("America/Chicago"), Some("en-US")),

            // Department Heads
            ("Dr. James Emergency", "james.emergency@rmc.health", Some("jemergency"), true, Some("1975-02-14"), Some("+1-555-7101"), Some("America/Chicago"), Some("en-US")),
            ("Dr. Linda Surgery", "linda.surgery@rmc.health", Some("lsurgery"), true, Some("1977-06-20"), Some("+1-555-7102"), Some("America/Chicago"), Some("en-US")),
            ("Dr. Mark Radiology", "mark.radiology@rmc.health", Some("mradiology"), true, Some("1979-10-05"), Some("+1-555-7103"), Some("America/Chicago"), Some("en-US")),

            // Medical Staff
            ("Dr. Karen Physician", "karen.physician@rmc.health", Some("kphysician"), true, Some("1982-01-18"), Some("+1-555-7201"), Some("America/Chicago"), Some("en-US")),
            ("Dr. Paul Surgeon", "paul.surgeon@rmc.health", Some("psurgeon"), true, Some("1984-05-30"), Some("+1-555-7202"), Some("America/Chicago"), Some("en-US")),
            ("Jessica Nurse Lead", "jessica.nurse@rmc.health", Some("jnurse"), true, Some("1986-09-22"), Some("+1-555-7203"), Some("America/Chicago"), Some("en-US")),

            // Support Staff
            ("Angela Lab Tech", "angela.lab@rmc.health", Some("alab"), true, Some("1988-12-11"), Some("+1-555-7301"), Some("America/Chicago"), Some("en-US")),
            ("Daniel Pharmacy", "daniel.pharmacy@rmc.health", Some("dpharmacy"), true, Some("1990-04-28"), Some("+1-555-7302"), Some("America/Chicago"), Some("en-US")),
            ("Rachel IT Health", "rachel.it@rmc.health", Some("rit"), true, Some("1992-08-15"), Some("+1-555-7303"), Some("America/Chicago"), Some("en-US")),

            // ==================== NON-GOVERNMENTAL (NGO) - Global Aid Foundation ====================
            // NGO Leadership
            ("Catherine Executive Dir", "catherine.ed@globalaid.org", Some("ced"), true, Some("1973-05-22"), Some("+1-555-8001"), Some("Europe/London"), Some("en-GB")),
            ("Mohammed Program Dir", "mohammed.pd@globalaid.org", Some("mpd"), true, Some("1975-09-18"), Some("+44-20-8002"), Some("Europe/London"), Some("en-GB")),
            ("Fatima Operations Dir", "fatima.od@globalaid.org", Some("fod"), true, Some("1977-01-30"), Some("+44-20-8003"), Some("Europe/London"), Some("en-GB")),

            // Program Managers
            ("Ahmed Field Manager", "ahmed.field@globalaid.org", Some("afield"), true, Some("1980-03-15"), Some("+44-20-8101"), Some("Africa/Nairobi"), Some("en-GB")),
            ("Aisha Health Program", "aisha.health@globalaid.org", Some("ahealth"), true, Some("1982-07-28"), Some("+44-20-8102"), Some("Africa/Nairobi"), Some("en-GB")),
            ("Hassan Education Lead", "hassan.edu@globalaid.org", Some("hedu"), true, Some("1984-11-12"), Some("+44-20-8103"), Some("Asia/Dubai"), Some("en-GB")),

            // Project Officers
            ("Leila Project Officer", "leila.project@globalaid.org", Some("lproject"), true, Some("1987-02-20"), Some("+44-20-8201"), Some("Europe/London"), Some("en-GB")),
            ("Omar Monitoring", "omar.monitoring@globalaid.org", Some("omonitoring"), true, Some("1989-06-14"), Some("+44-20-8202"), Some("Africa/Nairobi"), Some("en-GB")),
            ("Zainab Evaluation", "zainab.eval@globalaid.org", Some("zeval"), true, Some("1991-10-08"), Some("+44-20-8203"), Some("Asia/Dubai"), Some("en-GB")),

            // Support Staff
            ("Yusuf Logistics", "yusuf.logistics@globalaid.org", Some("ylogistics"), true, Some("1985-04-25"), Some("+44-20-8301"), Some("Europe/London"), Some("en-GB")),
            ("Mariam Fundraising", "mariam.fundraising@globalaid.org", Some("mfundraising"), true, Some("1988-08-19"), Some("+44-20-8302"), Some("Europe/London"), Some("en-GB")),

            // ==================== MILITARY (MIL) - Defense Command ====================
            // Command Leadership
            ("Gen. Robert Commander", "robert.commander@defense.mil", Some("rcommander"), true, Some("1965-07-04"), Some("+1-555-9001"), Some("America/New_York"), Some("en-US")),
            ("Col. William Deputy", "william.deputy@defense.mil", Some("wdeputy"), true, Some("1970-11-20"), Some("+1-555-9002"), Some("America/New_York"), Some("en-US")),
            ("Col. Barbara Chief Staff", "barbara.chief@defense.mil", Some("bchief"), true, Some("1972-03-15"), Some("+1-555-9003"), Some("America/New_York"), Some("en-US")),

            // Operations Officers
            ("Maj. Charles Operations", "charles.ops@defense.mil", Some("cops"), true, Some("1975-08-22"), Some("+1-555-9101"), Some("America/New_York"), Some("en-US")),
            ("Maj. Patricia Intelligence", "patricia.intel@defense.mil", Some("pintel"), true, Some("1977-12-10"), Some("+1-555-9102"), Some("America/New_York"), Some("en-US")),
            ("Capt. Joseph Logistics", "joseph.logistics@defense.mil", Some("jlogistics"), true, Some("1980-04-18"), Some("+1-555-9103"), Some("America/New_York"), Some("en-US")),

            // Technical Staff
            ("Capt. Steven Cyber Ops", "steven.cyber@defense.mil", Some("scyber"), true, Some("1982-09-25"), Some("+1-555-9201"), Some("America/New_York"), Some("en-US")),
            ("Lt. Jennifer Comms", "jennifer.comms@defense.mil", Some("jcomms"), true, Some("1985-01-30"), Some("+1-555-9202"), Some("America/New_York"), Some("en-US")),
            ("Lt. Michael Systems", "michael.systems@defense.mil", Some("msystems"), true, Some("1987-06-14"), Some("+1-555-9203"), Some("America/New_York"), Some("en-US")),

            // ==================== RELIGIOUS (REL) - St. Joseph Diocese ====================
            // Diocese Leadership
            ("Bishop Francis Leader", "francis.bishop@stjoseph.church", Some("fbishop"), true, Some("1960-12-25"), Some("+1-555-5001"), Some("America/Chicago"), Some("en-US")),
            ("Fr. Anthony Vicar", "anthony.vicar@stjoseph.church", Some("avicar"), true, Some("1968-05-10"), Some("+1-555-5002"), Some("America/Chicago"), Some("en-US")),
            ("Fr. Joseph Chancellor", "joseph.chancellor@stjoseph.church", Some("jchancellor"), true, Some("1970-09-22"), Some("+1-555-5003"), Some("America/Chicago"), Some("en-US")),

            // Parish Priests
            ("Fr. Michael Parish", "michael.parish@stjoseph.church", Some("mparish"), true, Some("1975-03-18"), Some("+1-555-5101"), Some("America/Chicago"), Some("en-US")),
            ("Fr. Patrick Mission", "patrick.mission@stjoseph.church", Some("pmission"), true, Some("1978-07-25"), Some("+1-555-5102"), Some("America/Chicago"), Some("en-US")),
            ("Fr. Thomas Community", "thomas.community@stjoseph.church", Some("tcommunity"), true, Some("1980-11-08"), Some("+1-555-5103"), Some("America/Chicago"), Some("en-US")),

            // Religious Staff
            ("Sr. Mary Education", "mary.education@stjoseph.church", Some("meducation"), true, Some("1972-02-14"), Some("+1-555-5201"), Some("America/Chicago"), Some("en-US")),
            ("Sr. Catherine Charity", "catherine.charity@stjoseph.church", Some("ccharity"), true, Some("1975-06-20"), Some("+1-555-5202"), Some("America/Chicago"), Some("en-US")),
            ("Deacon Paul Youth", "paul.youth@stjoseph.church", Some("pyouth"), true, Some("1982-10-12"), Some("+1-555-5203"), Some("America/Chicago"), Some("en-US")),

            // Administrative Staff
            ("Martha Administrator", "martha.admin@stjoseph.church", Some("madmin"), true, Some("1978-04-28"), Some("+1-555-5301"), Some("America/Chicago"), Some("en-US")),
            ("John Finance", "john.finance@stjoseph.church", Some("jfinance"), true, Some("1980-08-15"), Some("+1-555-5302"), Some("America/Chicago"), Some("en-US")),
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
                    sys_users::created_by_id.eq(&system_user_id),
                    sys_users::updated_by_id.eq(&system_user_id),
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
