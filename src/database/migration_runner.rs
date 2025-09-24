use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;
use diesel::prelude::*;
use diesel::sql_query;
use crate::database::{DbPool};
use crate::app::models::migration::Migration;
use crate::schema::migrations;

#[derive(QueryableByName)]
struct TableName {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub tablename: String,
}

#[derive(QueryableByName)]
struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub count: i64,
}

pub struct MigrationRunner {
    pool: DbPool,
    migrations_path: String,
}

#[derive(Debug)]
pub struct MigrationFile {
    pub filename: String,
    pub name: String,
    pub up_sql: String,
    pub down_sql: Option<String>,
}

impl MigrationRunner {
    pub fn new(pool: DbPool, migrations_path: String) -> Self {
        Self {
            pool,
            migrations_path,
        }
    }

    pub fn ensure_migrations_table(&self) -> Result<()> {
        let mut conn = self.pool.get()?;

        // Create table first
        let create_table = r#"
            CREATE TABLE IF NOT EXISTS migrations (
                id SERIAL PRIMARY KEY,
                migration VARCHAR(255) NOT NULL,
                batch INTEGER NOT NULL,
                executed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            )
        "#;
        sql_query(create_table).execute(&mut conn)?;

        // Create index separately
        let create_index = "CREATE INDEX IF NOT EXISTS idx_migrations_batch ON migrations(batch)";
        sql_query(create_index).execute(&mut conn)?;

        Ok(())
    }

    pub fn get_executed_migrations(&self) -> Result<Vec<Migration>> {
        let mut conn = self.pool.get()?;

        let migrations = migrations::table
            .order(migrations::id)
            .load::<Migration>(&mut conn)?;

        Ok(migrations)
    }

    pub fn get_pending_migrations(&self) -> Result<Vec<MigrationFile>> {
        let executed = self.get_executed_migrations()?;
        let executed_names: std::collections::HashSet<String> =
            executed.into_iter().map(|m| m.migration).collect();

        let all_migrations = self.load_migration_files()?;
        let pending = all_migrations
            .into_iter()
            .filter(|m| !executed_names.contains(&m.name))
            .collect();

        Ok(pending)
    }

    pub fn get_next_batch_number(&self) -> Result<i32> {
        let mut conn = self.pool.get()?;

        let result: Option<Option<i32>> = migrations::table
            .select(diesel::dsl::max(migrations::batch))
            .first(&mut conn)
            .optional()?;

        Ok(result.flatten().unwrap_or(0) + 1)
    }

    pub fn get_last_batch_number(&self) -> Result<Option<i32>> {
        let mut conn = self.pool.get()?;

        let result: Option<Option<i32>> = migrations::table
            .select(diesel::dsl::max(migrations::batch))
            .first(&mut conn)
            .optional()?;

        Ok(result.flatten())
    }

    pub fn run_migrations(&self) -> Result<()> {
        self.ensure_migrations_table()?;

        let pending = self.get_pending_migrations()?;
        if pending.is_empty() {
            println!("No pending migrations found.");
            return Ok(());
        }

        let batch = self.get_next_batch_number()?;
        println!("Running {} migrations in batch {}...", pending.len(), batch);

        for migration in pending {
            println!("Migrating: {}", migration.name);

            // Execute the migration SQL (split by semicolons and execute each statement)
            self.execute_sql_statements(&migration.up_sql, &migration.name)?;

            // Record the migration
            let mut conn = self.pool.get()?;
            diesel::insert_into(migrations::table)
                .values((
                    migrations::migration.eq(&migration.name),
                    migrations::batch.eq(batch),
                ))
                .execute(&mut conn)?;

            println!("Migrated: {}", migration.name);
        }

        println!("✅ All migrations completed successfully!");
        Ok(())
    }

    pub fn rollback_migrations(&self, steps: Option<i32>) -> Result<()> {
        self.ensure_migrations_table()?;

        let steps = steps.unwrap_or(1);
        let last_batch = self.get_last_batch_number()?;

        if last_batch.is_none() {
            println!("No migrations to rollback.");
            return Ok(());
        }

        let last_batch = last_batch.unwrap();
        let target_batch = std::cmp::max(0, last_batch - steps + 1);

        let mut conn = self.pool.get()?;
        let migrations_to_rollback: Vec<String> = migrations::table
            .select(migrations::migration)
            .filter(migrations::batch.ge(target_batch))
            .order(migrations::id.desc())
            .load(&mut conn)?;

        if migrations_to_rollback.is_empty() {
            println!("No migrations to rollback.");
            return Ok(());
        }

        println!("Rolling back {} migrations...", migrations_to_rollback.len());

        for migration_name in migrations_to_rollback {
            println!("Rolling back: {}", migration_name);

            // Load migration file to get down SQL
            if let Some(migration_file) = self.load_migration_file(&migration_name)? {
                if let Some(down_sql) = migration_file.down_sql {
                    // Execute rollback SQL
                    self.execute_sql_statements(&down_sql, &migration_name)
                        .map_err(|e| anyhow!("Failed to rollback migration {}: {}", migration_name, e))?;

                    // Remove from migrations table
                    let mut conn = self.pool.get()?;
                    diesel::delete(migrations::table.filter(migrations::migration.eq(&migration_name)))
                        .execute(&mut conn)?;

                    println!("Rolled back: {}", migration_name);
                } else {
                    println!("⚠️  No rollback SQL found for: {}", migration_name);
                    println!("   Remove manually or provide down migration SQL");
                }
            } else {
                println!("⚠️  Migration file not found for: {}", migration_name);
            }
        }

        println!("✅ Rollback completed!");
        Ok(())
    }

    pub fn reset_migrations(&self) -> Result<()> {
        self.ensure_migrations_table()?;

        println!("Resetting all migrations...");

        let mut conn = self.pool.get()?;

        // Get all table names from the database (excluding system tables)
        let tables: Vec<String> = sql_query(
            r#"
            SELECT tablename
            FROM pg_tables
            WHERE schemaname = 'public'
            AND tablename != 'migrations'
            ORDER BY tablename
            "#
        )
        .load::<TableName>(&mut conn)?
        .into_iter()
        .map(|t| t.tablename)
        .collect();

        if !tables.is_empty() {
            // Drop all tables with CASCADE to handle foreign key dependencies
            let table_names: Vec<String> = tables.iter()
                .map(|t| format!("\"{}\"", t))
                .collect();

            let drop_sql = format!("DROP TABLE IF EXISTS {} CASCADE", table_names.join(", "));
            println!("Dropping all tables...");

            sql_query(&drop_sql)
                .execute(&mut conn)
                .map_err(|e| anyhow!("Failed to drop tables: {}", e))?;
        }

        // Clear migrations table
        diesel::delete(migrations::table).execute(&mut conn)?;
        println!("✅ All migrations reset!");
        Ok(())
    }

    pub fn refresh_migrations(&self) -> Result<()> {
        println!("Refreshing migrations (reset + migrate)...");
        self.reset_migrations()?;
        self.run_migrations()?;
        Ok(())
    }

    pub fn show_status(&self) -> Result<()> {
        self.ensure_migrations_table()?;

        let executed = self.get_executed_migrations()?;
        let pending = self.get_pending_migrations()?;

        println!("\n📊 Migration Status");
        println!("==================");

        if !executed.is_empty() {
            println!("\n✅ Executed Migrations:");
            for migration in &executed {
                println!("  [Batch {}] {} ({})",
                    migration.batch,
                    migration.migration,
                    migration.executed_at.format("%Y-%m-%d %H:%M:%S")
                );
            }
        }

        if !pending.is_empty() {
            println!("\n⏳ Pending Migrations:");
            for migration in &pending {
                println!("  {}", migration.name);
            }
        }

        if executed.is_empty() && pending.is_empty() {
            println!("\nNo migrations found.");
        }

        println!("\nSummary: {} executed, {} pending", executed.len(), pending.len());
        Ok(())
    }

    fn load_migration_files(&self) -> Result<Vec<MigrationFile>> {
        let mut migrations = std::collections::HashMap::new();
        let path = Path::new(&self.migrations_path);

        if !path.exists() {
            return Ok(Vec::new());
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if let Some(extension) = file_path.extension() {
                if extension == "sql" {
                    if let Some(filename) = file_path.file_name() {
                        let filename_str = filename.to_string_lossy().to_string();

                        if filename_str.ends_with(".up.sql") {
                            let name = filename_str.trim_end_matches(".up.sql").to_string();
                            let content = fs::read_to_string(&file_path)?;

                            let migration = migrations.entry(name.clone()).or_insert_with(|| MigrationFile {
                                filename: format!("{}.up.sql", name),
                                name: name.clone(),
                                up_sql: String::new(),
                                down_sql: None,
                            });
                            migration.up_sql = content.trim().to_string();
                        } else if filename_str.ends_with(".down.sql") {
                            let name = filename_str.trim_end_matches(".down.sql").to_string();
                            let content = fs::read_to_string(&file_path)?;

                            let migration = migrations.entry(name.clone()).or_insert_with(|| MigrationFile {
                                filename: format!("{}.up.sql", name),
                                name: name.clone(),
                                up_sql: String::new(),
                                down_sql: None,
                            });
                            migration.down_sql = Some(content.trim().to_string());
                        }
                    }
                }
            }
        }

        let mut migration_list: Vec<MigrationFile> = migrations.into_values()
            .filter(|m| !m.up_sql.is_empty()) // Only include migrations that have up SQL
            .collect();

        // Sort migrations by name to ensure proper order
        migration_list.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(migration_list)
    }

    fn load_migration_file(&self, migration_name: &str) -> Result<Option<MigrationFile>> {
        let migrations = self.load_migration_files()?;
        Ok(migrations.into_iter().find(|m| m.name == migration_name))
    }


    fn execute_sql_statements(&self, sql: &str, migration_name: &str) -> Result<()> {
        let mut conn = self.pool.get()?;

        // Use a transaction to ensure all statements in a migration are executed atomically
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            // Parse SQL statements properly, handling functions that contain semicolons
            let statements = self.parse_sql_statements(sql)
                .map_err(|e| diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::Unknown,
                    Box::new(e.to_string())
                ))?;

            for statement in statements {
                if !statement.is_empty() {
                    sql_query(&statement)
                        .execute(conn)
                        .map_err(|e| {
                            diesel::result::Error::DatabaseError(
                                diesel::result::DatabaseErrorKind::Unknown,
                                Box::new(format!("Failed to execute SQL statement in migration {}: {}\nStatement: {}", migration_name, e, statement))
                            )
                        })?;
                }
            }
            Ok(())
        }).map_err(|e| anyhow!("Transaction failed: {}", e))?;

        Ok(())
    }

    fn parse_sql_statements(&self, sql: &str) -> Result<Vec<String>> {
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        let mut in_dollar_quote = false;
        let mut dollar_tag = String::new();

        // Remove comments and split into lines
        let lines: Vec<&str> = sql.lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.starts_with("--") && !trimmed.is_empty()
            })
            .collect();

        let full_text = lines.join("\n");
        let chars: Vec<char> = full_text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if ch == '$' && !in_dollar_quote {
                // Look for start of dollar quote
                let mut tag = String::new();
                let mut j = i + 1;

                // Extract the tag between $$
                while j < chars.len() && chars[j] != '$' {
                    tag.push(chars[j]);
                    j += 1;
                }

                if j < chars.len() && chars[j] == '$' {
                    // Found complete dollar quote start
                    in_dollar_quote = true;
                    dollar_tag = tag;
                    current_statement.push_str(&format!("${}$", dollar_tag));
                    i = j;
                } else {
                    current_statement.push(ch);
                }
            } else if ch == '$' && in_dollar_quote {
                // Look for end of dollar quote
                let mut tag = String::new();
                let mut j = i + 1;

                // Extract the tag between $$
                while j < chars.len() && chars[j] != '$' {
                    tag.push(chars[j]);
                    j += 1;
                }

                if j < chars.len() && chars[j] == '$' && tag == dollar_tag {
                    // Found matching dollar quote end
                    in_dollar_quote = false;
                    current_statement.push_str(&format!("${}$", tag));
                    dollar_tag.clear();
                    i = j;
                } else {
                    current_statement.push(ch);
                }
            } else if ch == ';' && !in_dollar_quote {
                // End of statement
                let statement = current_statement.trim().to_string();
                if !statement.is_empty() {
                    statements.push(statement);
                }
                current_statement.clear();
            } else {
                current_statement.push(ch);
            }

            i += 1;
        }

        // Add any remaining statement
        let statement = current_statement.trim().to_string();
        if !statement.is_empty() {
            statements.push(statement);
        }

        Ok(statements)
    }

    /// Get the count of applied migrations
    pub fn count_applied_migrations(&self) -> Result<i64> {
        self.ensure_migrations_table()?;

        let mut conn = self.pool.get()?;
        let result: CountResult = sql_query("SELECT COUNT(*) as count FROM migrations")
            .get_result(&mut conn)?;

        Ok(result.count)
    }
}