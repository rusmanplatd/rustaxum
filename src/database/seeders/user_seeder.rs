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

        let users = vec![
            ("Admin User", "admin@example.com", Some("admin"), true, Some("1985-01-15"), Some("+1-555-0001"), Some("America/New_York"), Some("en-US")),
            ("John Smith", "john.smith@example.com", Some("jsmith"), true, Some("1990-03-22"), Some("+1-555-0002"), Some("America/New_York"), Some("en-US")),
            ("Sarah Johnson", "sarah.johnson@example.com", Some("sjohnson"), true, Some("1988-07-10"), Some("+1-555-0003"), Some("America/Chicago"), Some("en-US")),
            ("Michael Brown", "michael.brown@example.com", Some("mbrown"), true, Some("1987-11-05"), Some("+1-555-0004"), Some("America/Denver"), Some("en-US")),
            ("Emily Davis", "emily.davis@example.com", Some("edavis"), true, Some("1992-04-18"), Some("+1-555-0005"), Some("America/Los_Angeles"), Some("en-US")),
            ("David Wilson", "david.wilson@example.com", Some("dwilson"), true, Some("1986-09-12"), Some("+1-555-0006"), Some("America/New_York"), Some("en-US")),
            ("Lisa Miller", "lisa.miller@example.com", Some("lmiller"), true, Some("1991-02-28"), Some("+1-555-0007"), Some("America/Chicago"), Some("en-US")),
            ("Robert Garcia", "robert.garcia@example.com", Some("rgarcia"), true, Some("1989-06-14"), Some("+1-555-0008"), Some("America/Phoenix"), Some("es-US")),
            ("Jennifer Martinez", "jennifer.martinez@example.com", Some("jmartinez"), true, Some("1993-08-03"), Some("+1-555-0009"), Some("America/Los_Angeles"), Some("es-US")),
            ("William Anderson", "william.anderson@example.com", Some("wanderson"), true, Some("1984-12-20"), Some("+1-555-0010"), Some("America/New_York"), Some("en-US")),
            ("Jessica Taylor", "jessica.taylor@example.com", Some("jtaylor"), true, Some("1990-05-07"), Some("+1-555-0011"), Some("America/Chicago"), Some("en-US")),
            ("Christopher Thomas", "christopher.thomas@example.com", Some("cthomas"), true, Some("1987-10-25"), Some("+1-555-0012"), Some("America/Denver"), Some("en-US")),
            ("Amanda Jackson", "amanda.jackson@example.com", Some("ajackson"), true, Some("1994-01-16"), Some("+1-555-0013"), Some("America/Los_Angeles"), Some("en-US")),
            ("Matthew White", "matthew.white@example.com", Some("mwhite"), true, Some("1988-03-09"), Some("+1-555-0014"), Some("America/New_York"), Some("en-US")),
            ("Ashley Harris", "ashley.harris@example.com", Some("aharris"), true, Some("1991-07-23"), Some("+1-555-0015"), Some("America/Chicago"), Some("en-US")),
            ("Daniel Martin", "daniel.martin@example.com", Some("dmartin"), true, Some("1986-11-11"), Some("+1-555-0016"), Some("America/Phoenix"), Some("en-US")),
            ("Stephanie Thompson", "stephanie.thompson@example.com", Some("sthompson"), true, Some("1992-09-04"), Some("+1-555-0017"), Some("America/Los_Angeles"), Some("en-US")),
            ("Joshua Garcia", "joshua.garcia@example.com", Some("jgarcia"), true, Some("1989-12-18"), Some("+1-555-0018"), Some("America/Denver"), Some("es-US")),
            ("Michelle Rodriguez", "michelle.rodriguez@example.com", Some("mrodriguez"), true, Some("1993-04-27"), Some("+1-555-0019"), Some("America/Los_Angeles"), Some("es-US")),
            ("Andrew Lewis", "andrew.lewis@example.com", Some("alewis"), true, Some("1985-08-13"), Some("+1-555-0020"), Some("America/New_York"), Some("en-US")),
            ("Elizabeth Lee", "elizabeth.lee@example.com", Some("elee"), true, Some("1990-10-31"), Some("+1-555-0021"), Some("America/Chicago"), Some("en-US")),
            ("Ryan Walker", "ryan.walker@example.com", Some("rwalker"), true, Some("1987-06-06"), Some("+1-555-0022"), Some("America/Denver"), Some("en-US")),
            ("Nicole Hall", "nicole.hall@example.com", Some("nhall"), true, Some("1994-02-14"), Some("+1-555-0023"), Some("America/Los_Angeles"), Some("en-US")),
            ("Brandon Allen", "brandon.allen@example.com", Some("ballen"), true, Some("1988-05-19"), Some("+1-555-0024"), Some("America/Phoenix"), Some("en-US")),
            ("Samantha Young", "samantha.young@example.com", Some("syoung"), true, Some("1991-09-08"), Some("+1-555-0025"), Some("America/New_York"), Some("en-US")),
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

        println!("✅ 25 Users seeded successfully!");
        Ok(())
    }
}
